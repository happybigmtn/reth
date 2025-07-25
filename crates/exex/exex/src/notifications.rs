use crate::{BackfillJobFactory, ExExNotification, StreamBackfillJob, WalHandle};
use alloy_consensus::BlockHeader;
use alloy_eips::BlockNumHash;
use futures::{Stream, StreamExt};
use reth_ethereum_primitives::EthPrimitives;
use reth_evm::ConfigureEvm;
use reth_exex_types::ExExHead;
use reth_node_api::NodePrimitives;
use reth_provider::{BlockReader, Chain, HeaderProvider, StateProviderFactory};
use reth_tracing::tracing::debug;
use std::{
    fmt::Debug,
    pin::Pin,
    sync::Arc,
    task::{ready, Context, Poll},
};
use tokio::sync::mpsc::Receiver;

/// A stream of [`ExExNotification`]s. The stream will emit notifications for all blocks. If the
/// stream is configured with a head via [`ExExNotifications::set_with_head`] or
/// [`ExExNotifications::with_head`], it will run backfill jobs to catch up to the node head.
#[derive(Debug)]
pub struct ExExNotifications<P, E>
where
    E: ConfigureEvm,
{
    inner: ExExNotificationsInner<P, E>,
}

/// A trait, that represents a stream of [`ExExNotification`]s. The stream will emit notifications
/// for all blocks. If the stream is configured with a head via [`ExExNotifications::set_with_head`]
/// or [`ExExNotifications::with_head`], it will run backfill jobs to catch up to the node head.
pub trait ExExNotificationsStream<N: NodePrimitives = EthPrimitives>:
    Stream<Item = eyre::Result<ExExNotification<N>>> + Unpin
{
    /// Sets [`ExExNotificationsStream`] to a stream of [`ExExNotification`]s without a head.
    ///
    /// It's a no-op if the stream has already been configured without a head.
    ///
    /// See the documentation of [`ExExNotificationsWithoutHead`] for more details.
    fn set_without_head(&mut self);

    /// Sets [`ExExNotificationsStream`] to a stream of [`ExExNotification`]s with the provided
    /// head.
    ///
    /// It's a no-op if the stream has already been configured with a head.
    ///
    /// See the documentation of [`ExExNotificationsWithHead`] for more details.
    fn set_with_head(&mut self, exex_head: ExExHead);

    /// Returns a new [`ExExNotificationsStream`] without a head.
    ///
    /// See the documentation of [`ExExNotificationsWithoutHead`] for more details.
    fn without_head(self) -> Self
    where
        Self: Sized;

    /// Returns a new [`ExExNotificationsStream`] with the provided head.
    ///
    /// See the documentation of [`ExExNotificationsWithHead`] for more details.
    fn with_head(self, exex_head: ExExHead) -> Self
    where
        Self: Sized;
}

#[derive(Debug)]
enum ExExNotificationsInner<P, E>
where
    E: ConfigureEvm,
{
    /// A stream of [`ExExNotification`]s. The stream will emit notifications for all blocks.
    WithoutHead(ExExNotificationsWithoutHead<P, E>),
    /// A stream of [`ExExNotification`]s. The stream will only emit notifications for blocks that
    /// are committed or reverted after the given head.
    WithHead(Box<ExExNotificationsWithHead<P, E>>),
    /// Internal state used when transitioning between [`ExExNotificationsInner::WithoutHead`] and
    /// [`ExExNotificationsInner::WithHead`].
    Invalid,
}

impl<P, E> ExExNotifications<P, E>
where
    E: ConfigureEvm,
{
    /// Creates a new stream of [`ExExNotifications`] without a head.
    pub const fn new(
        node_head: BlockNumHash,
        provider: P,
        evm_config: E,
        notifications: Receiver<ExExNotification<E::Primitives>>,
        wal_handle: WalHandle<E::Primitives>,
    ) -> Self {
        Self {
            inner: ExExNotificationsInner::WithoutHead(ExExNotificationsWithoutHead::new(
                node_head,
                provider,
                evm_config,
                notifications,
                wal_handle,
            )),
        }
    }
}

impl<P, E> ExExNotificationsStream<E::Primitives> for ExExNotifications<P, E>
where
    P: BlockReader + HeaderProvider + StateProviderFactory + Clone + Unpin + 'static,
    E: ConfigureEvm<Primitives: NodePrimitives<Block = P::Block>> + Clone + Unpin + 'static,
{
    fn set_without_head(&mut self) {
        let current = std::mem::replace(&mut self.inner, ExExNotificationsInner::Invalid);
        self.inner = ExExNotificationsInner::WithoutHead(match current {
            ExExNotificationsInner::WithoutHead(notifications) => notifications,
            ExExNotificationsInner::WithHead(notifications) => ExExNotificationsWithoutHead::new(
                notifications.initial_local_head,
                notifications.provider,
                notifications.evm_config,
                notifications.notifications,
                notifications.wal_handle,
            ),
            ExExNotificationsInner::Invalid => unreachable!(),
        });
    }

    fn set_with_head(&mut self, exex_head: ExExHead) {
        let current = std::mem::replace(&mut self.inner, ExExNotificationsInner::Invalid);
        self.inner = ExExNotificationsInner::WithHead(match current {
            ExExNotificationsInner::WithoutHead(notifications) => {
                Box::new(notifications.with_head(exex_head))
            }
            ExExNotificationsInner::WithHead(notifications) => {
                Box::new(ExExNotificationsWithHead::new(
                    notifications.initial_local_head,
                    notifications.provider,
                    notifications.evm_config,
                    notifications.notifications,
                    notifications.wal_handle,
                    exex_head,
                ))
            }
            ExExNotificationsInner::Invalid => unreachable!(),
        });
    }

    fn without_head(mut self) -> Self {
        self.set_without_head();
        self
    }

    fn with_head(mut self, exex_head: ExExHead) -> Self {
        self.set_with_head(exex_head);
        self
    }
}

impl<P, E> Stream for ExExNotifications<P, E>
where
    P: BlockReader + HeaderProvider + StateProviderFactory + Clone + Unpin + 'static,
    E: ConfigureEvm<Primitives: NodePrimitives<Block = P::Block>> + 'static,
{
    type Item = eyre::Result<ExExNotification<E::Primitives>>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match &mut self.get_mut().inner {
            ExExNotificationsInner::WithoutHead(notifications) => {
                notifications.poll_next_unpin(cx).map(|result| result.map(Ok))
            }
            ExExNotificationsInner::WithHead(notifications) => notifications.poll_next_unpin(cx),
            ExExNotificationsInner::Invalid => unreachable!(),
        }
    }
}

/// A stream of [`ExExNotification`]s. The stream will emit notifications for all blocks.
pub struct ExExNotificationsWithoutHead<P, E>
where
    E: ConfigureEvm,
{
    node_head: BlockNumHash,
    provider: P,
    evm_config: E,
    notifications: Receiver<ExExNotification<E::Primitives>>,
    wal_handle: WalHandle<E::Primitives>,
}

impl<P: Debug, E> Debug for ExExNotificationsWithoutHead<P, E>
where
    E: ConfigureEvm + Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExExNotifications")
            .field("provider", &self.provider)
            .field("evm_config", &self.evm_config)
            .field("notifications", &self.notifications)
            .finish()
    }
}

impl<P, E> ExExNotificationsWithoutHead<P, E>
where
    E: ConfigureEvm,
{
    /// Creates a new instance of [`ExExNotificationsWithoutHead`].
    const fn new(
        node_head: BlockNumHash,
        provider: P,
        evm_config: E,
        notifications: Receiver<ExExNotification<E::Primitives>>,
        wal_handle: WalHandle<E::Primitives>,
    ) -> Self {
        Self { node_head, provider, evm_config, notifications, wal_handle }
    }

    /// Subscribe to notifications with the given head.
    fn with_head(self, head: ExExHead) -> ExExNotificationsWithHead<P, E> {
        ExExNotificationsWithHead::new(
            self.node_head,
            self.provider,
            self.evm_config,
            self.notifications,
            self.wal_handle,
            head,
        )
    }
}

impl<P: Unpin, E> Stream for ExExNotificationsWithoutHead<P, E>
where
    E: ConfigureEvm,
{
    type Item = ExExNotification<E::Primitives>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.get_mut().notifications.poll_recv(cx)
    }
}

/// A stream of [`ExExNotification`]s. The stream will only emit notifications for blocks that are
/// committed or reverted after the given head. The head is the ExEx's latest view of the host
/// chain.
///
/// Notifications will be sent starting from the head, not inclusive. For example, if
/// `exex_head.number == 10`, then the first notification will be with `block.number == 11`. An
/// `exex_head.number` of 10 indicates that the ExEx has processed up to block 10, and is ready to
/// process block 11.
#[derive(Debug)]
pub struct ExExNotificationsWithHead<P, E>
where
    E: ConfigureEvm,
{
    /// The node's local head at launch.
    initial_local_head: BlockNumHash,
    provider: P,
    evm_config: E,
    notifications: Receiver<ExExNotification<E::Primitives>>,
    wal_handle: WalHandle<E::Primitives>,
    /// The exex head at launch
    initial_exex_head: ExExHead,

    /// If true, then we need to check if the ExEx head is on the canonical chain and if not,
    /// revert its head.
    pending_check_canonical: bool,
    /// If true, then we need to check if the ExEx head is behind the node head and if so, backfill
    /// the missing blocks.
    pending_check_backfill: bool,
    /// The backfill job to run before consuming any notifications.
    backfill_job: Option<StreamBackfillJob<E, P, Chain<E::Primitives>>>,
}

impl<P, E> ExExNotificationsWithHead<P, E>
where
    E: ConfigureEvm,
{
    /// Creates a new [`ExExNotificationsWithHead`].
    const fn new(
        node_head: BlockNumHash,
        provider: P,
        evm_config: E,
        notifications: Receiver<ExExNotification<E::Primitives>>,
        wal_handle: WalHandle<E::Primitives>,
        exex_head: ExExHead,
    ) -> Self {
        Self {
            initial_local_head: node_head,
            provider,
            evm_config,
            notifications,
            wal_handle,
            initial_exex_head: exex_head,
            pending_check_canonical: true,
            pending_check_backfill: true,
            backfill_job: None,
        }
    }
}

impl<P, E> ExExNotificationsWithHead<P, E>
where
    P: BlockReader + HeaderProvider + StateProviderFactory + Clone + Unpin + 'static,
    E: ConfigureEvm<Primitives: NodePrimitives<Block = P::Block>> + Clone + Unpin + 'static,
{
    /// Checks if the ExEx head is on the canonical chain.
    ///
    /// If the head block is not found in the database or it's ahead of the node head, it means
    /// we're not on the canonical chain and we need to revert the notification with the ExEx
    /// head block.
    fn check_canonical(&mut self) -> eyre::Result<Option<ExExNotification<E::Primitives>>> {
        if self.provider.is_known(&self.initial_exex_head.block.hash)? &&
            self.initial_exex_head.block.number <= self.initial_local_head.number
        {
            // we have the targeted block and that block is below the current head
            debug!(target: "exex::notifications", "ExEx head is on the canonical chain");
            return Ok(None)
        }

        // If the head block is not found in the database, it means we're not on the canonical
        // chain.

        // Get the committed notification for the head block from the WAL.
        let Some(notification) = self
            .wal_handle
            .get_committed_notification_by_block_hash(&self.initial_exex_head.block.hash)?
        else {
            // it's possible that the exex head is further ahead
            if self.initial_exex_head.block.number > self.initial_local_head.number {
                debug!(target: "exex::notifications", "ExEx head is ahead of the canonical chain");
                return Ok(None);
            }

            return Err(eyre::eyre!(
                "Could not find notification for block hash {:?} in the WAL",
                self.initial_exex_head.block.hash
            ))
        };

        // Update the head block hash to the parent hash of the first committed block.
        let committed_chain = notification.committed_chain().unwrap();
        let new_exex_head =
            (committed_chain.first().parent_hash(), committed_chain.first().number() - 1).into();
        debug!(target: "exex::notifications", old_exex_head = ?self.initial_exex_head.block, new_exex_head = ?new_exex_head, "ExEx head updated");
        self.initial_exex_head.block = new_exex_head;

        // Return an inverted notification. See the documentation for
        // `ExExNotification::into_inverted`.
        Ok(Some(notification.into_inverted()))
    }

    /// Compares the node head against the ExEx head, and backfills if needed.
    ///
    /// CAUTION: This method assumes that the ExEx head is <= the node head, and that it's on the
    /// canonical chain.
    ///
    /// Possible situations are:
    /// - ExEx is behind the node head (`node_head.number < exex_head.number`). Backfill from the
    ///   node database.
    /// - ExEx is at the same block number as the node head (`node_head.number ==
    ///   exex_head.number`). Nothing to do.
    fn check_backfill(&mut self) -> eyre::Result<()> {
        let backfill_job_factory =
            BackfillJobFactory::new(self.evm_config.clone(), self.provider.clone());
        match self.initial_exex_head.block.number.cmp(&self.initial_local_head.number) {
            std::cmp::Ordering::Less => {
                // ExEx is behind the node head, start backfill
                debug!(target: "exex::notifications", "ExEx is behind the node head and on the canonical chain, starting backfill");
                let backfill = backfill_job_factory
                    .backfill(
                        self.initial_exex_head.block.number + 1..=self.initial_local_head.number,
                    )
                    .into_stream();
                self.backfill_job = Some(backfill);
            }
            std::cmp::Ordering::Equal => {
                debug!(target: "exex::notifications", "ExEx is at the node head");
            }
            std::cmp::Ordering::Greater => {
                debug!(target: "exex::notifications", "ExEx is ahead of the node head");
            }
        };

        Ok(())
    }
}

impl<P, E> Stream for ExExNotificationsWithHead<P, E>
where
    P: BlockReader + HeaderProvider + StateProviderFactory + Clone + Unpin + 'static,
    E: ConfigureEvm<Primitives: NodePrimitives<Block = P::Block>> + Clone + Unpin + 'static,
{
    type Item = eyre::Result<ExExNotification<E::Primitives>>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let this = self.get_mut();

        // 1. Check once whether we need to retrieve a notification gap from the WAL.
        if this.pending_check_canonical {
            if let Some(canonical_notification) = this.check_canonical()? {
                return Poll::Ready(Some(Ok(canonical_notification)))
            }

            // ExEx head is on the canonical chain, we no longer need to check it
            this.pending_check_canonical = false;
        }

        // 2. Check once whether we need to trigger backfill sync
        if this.pending_check_backfill {
            this.check_backfill()?;
            this.pending_check_backfill = false;
        }

        // 3. If backfill is in progress yield new notifications
        if let Some(backfill_job) = &mut this.backfill_job {
            debug!(target: "exex::notifications", "Polling backfill job");
            if let Some(chain) = ready!(backfill_job.poll_next_unpin(cx)).transpose()? {
                debug!(target: "exex::notifications", range = ?chain.range(), "Backfill job returned a chain");
                return Poll::Ready(Some(Ok(ExExNotification::ChainCommitted {
                    new: Arc::new(chain),
                })))
            }

            // Backfill job is done, remove it
            this.backfill_job = None;
        }

        // 4. Otherwise advance the regular event stream
        loop {
            let Some(notification) = ready!(this.notifications.poll_recv(cx)) else {
                return Poll::Ready(None)
            };

            // 5. In case the exex is ahead of the new tip, we must skip it
            if let Some(committed) = notification.committed_chain() {
                // inclusive check because we should start with `exex.head + 1`
                if this.initial_exex_head.block.number >= committed.tip().number() {
                    continue
                }
            }

            return Poll::Ready(Some(Ok(notification)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Wal;
    use alloy_consensus::Header;
    use alloy_eips::BlockNumHash;
    use eyre::OptionExt;
    use futures::StreamExt;
    use reth_db_common::init::init_genesis;
    use reth_ethereum_primitives::Block;
    use reth_evm_ethereum::EthEvmConfig;
    use reth_primitives_traits::Block as _;
    use reth_provider::{
        providers::BlockchainProvider, test_utils::create_test_provider_factory, BlockWriter,
        Chain, DatabaseProviderFactory, StorageLocation,
    };
    use reth_testing_utils::generators::{self, random_block, BlockParams};
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn exex_notifications_behind_head_canonical() -> eyre::Result<()> {
        let mut rng = generators::rng();

        let temp_dir = tempfile::tempdir().unwrap();
        let wal = Wal::new(temp_dir.path()).unwrap();

        let provider_factory = create_test_provider_factory();
        let genesis_hash = init_genesis(&provider_factory)?;
        let genesis_block = provider_factory
            .block(genesis_hash.into())?
            .ok_or_else(|| eyre::eyre!("genesis block not found"))?;

        let provider = BlockchainProvider::new(provider_factory.clone())?;

        let node_head_block = random_block(
            &mut rng,
            genesis_block.number + 1,
            BlockParams { parent: Some(genesis_hash), tx_count: Some(0), ..Default::default() },
        );
        let provider_rw = provider_factory.provider_rw()?;
        provider_rw
            .insert_block(node_head_block.clone().try_recover()?, StorageLocation::Database)?;
        provider_rw.commit()?;

        let node_head = node_head_block.num_hash();
        let exex_head =
            ExExHead { block: BlockNumHash { number: genesis_block.number, hash: genesis_hash } };

        let notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![random_block(
                    &mut rng,
                    node_head.number + 1,
                    BlockParams { parent: Some(node_head.hash), ..Default::default() },
                )
                .try_recover()?],
                Default::default(),
                None,
            )),
        };

        let (notifications_tx, notifications_rx) = mpsc::channel(1);

        notifications_tx.send(notification.clone()).await?;

        let mut notifications = ExExNotificationsWithoutHead::new(
            node_head,
            provider,
            EthEvmConfig::mainnet(),
            notifications_rx,
            wal.handle(),
        )
        .with_head(exex_head);

        // First notification is the backfill of missing blocks from the canonical chain
        assert_eq!(
            notifications.next().await.transpose()?,
            Some(ExExNotification::ChainCommitted {
                new: Arc::new(
                    BackfillJobFactory::new(
                        notifications.evm_config.clone(),
                        notifications.provider.clone()
                    )
                    .backfill(1..=1)
                    .next()
                    .ok_or_eyre("failed to backfill")??
                )
            })
        );

        // Second notification is the actual notification that we sent before
        assert_eq!(notifications.next().await.transpose()?, Some(notification));

        Ok(())
    }

    #[tokio::test]
    async fn exex_notifications_same_head_canonical() -> eyre::Result<()> {
        let temp_dir = tempfile::tempdir().unwrap();
        let wal = Wal::new(temp_dir.path()).unwrap();

        let provider_factory = create_test_provider_factory();
        let genesis_hash = init_genesis(&provider_factory)?;
        let genesis_block = provider_factory
            .block(genesis_hash.into())?
            .ok_or_else(|| eyre::eyre!("genesis block not found"))?;

        let provider = BlockchainProvider::new(provider_factory)?;

        let node_head = BlockNumHash { number: genesis_block.number, hash: genesis_hash };
        let exex_head = ExExHead { block: node_head };

        let notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![Block {
                    header: Header {
                        parent_hash: node_head.hash,
                        number: node_head.number + 1,
                        ..Default::default()
                    },
                    ..Default::default()
                }
                .seal_slow()
                .try_recover()?],
                Default::default(),
                None,
            )),
        };

        let (notifications_tx, notifications_rx) = mpsc::channel(1);

        notifications_tx.send(notification.clone()).await?;

        let mut notifications = ExExNotificationsWithoutHead::new(
            node_head,
            provider,
            EthEvmConfig::mainnet(),
            notifications_rx,
            wal.handle(),
        )
        .with_head(exex_head);

        let new_notification = notifications.next().await.transpose()?;
        assert_eq!(new_notification, Some(notification));

        Ok(())
    }

    #[tokio::test]
    async fn exex_notifications_same_head_non_canonical() -> eyre::Result<()> {
        let mut rng = generators::rng();

        let temp_dir = tempfile::tempdir().unwrap();
        let wal = Wal::new(temp_dir.path()).unwrap();

        let provider_factory = create_test_provider_factory();
        let genesis_hash = init_genesis(&provider_factory)?;
        let genesis_block = provider_factory
            .block(genesis_hash.into())?
            .ok_or_else(|| eyre::eyre!("genesis block not found"))?;

        let provider = BlockchainProvider::new(provider_factory)?;

        let node_head_block = random_block(
            &mut rng,
            genesis_block.number + 1,
            BlockParams { parent: Some(genesis_hash), tx_count: Some(0), ..Default::default() },
        )
        .try_recover()?;
        let node_head = node_head_block.num_hash();
        let provider_rw = provider.database_provider_rw()?;
        provider_rw.insert_block(node_head_block, StorageLocation::Database)?;
        provider_rw.commit()?;
        let node_head_notification = ExExNotification::ChainCommitted {
            new: Arc::new(
                BackfillJobFactory::new(EthEvmConfig::mainnet(), provider.clone())
                    .backfill(node_head.number..=node_head.number)
                    .next()
                    .ok_or_else(|| eyre::eyre!("failed to backfill"))??,
            ),
        };

        let exex_head_block = random_block(
            &mut rng,
            genesis_block.number + 1,
            BlockParams { parent: Some(genesis_hash), tx_count: Some(0), ..Default::default() },
        );
        let exex_head = ExExHead { block: exex_head_block.num_hash() };
        let exex_head_notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![exex_head_block.clone().try_recover()?],
                Default::default(),
                None,
            )),
        };
        wal.commit(&exex_head_notification)?;

        let new_notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![random_block(
                    &mut rng,
                    node_head.number + 1,
                    BlockParams { parent: Some(node_head.hash), ..Default::default() },
                )
                .try_recover()?],
                Default::default(),
                None,
            )),
        };

        let (notifications_tx, notifications_rx) = mpsc::channel(1);

        notifications_tx.send(new_notification.clone()).await?;

        let mut notifications = ExExNotificationsWithoutHead::new(
            node_head,
            provider,
            EthEvmConfig::mainnet(),
            notifications_rx,
            wal.handle(),
        )
        .with_head(exex_head);

        // First notification is the revert of the ExEx head block to get back to the canonical
        // chain
        assert_eq!(
            notifications.next().await.transpose()?,
            Some(exex_head_notification.into_inverted())
        );
        // Second notification is the backfilled block from the canonical chain to get back to the
        // canonical tip
        assert_eq!(notifications.next().await.transpose()?, Some(node_head_notification));
        // Third notification is the actual notification that we sent before
        assert_eq!(notifications.next().await.transpose()?, Some(new_notification));

        Ok(())
    }

    #[tokio::test]
    async fn test_notifications_ahead_of_head() -> eyre::Result<()> {
        reth_tracing::init_test_tracing();
        let mut rng = generators::rng();

        let temp_dir = tempfile::tempdir().unwrap();
        let wal = Wal::new(temp_dir.path()).unwrap();

        let provider_factory = create_test_provider_factory();
        let genesis_hash = init_genesis(&provider_factory)?;
        let genesis_block = provider_factory
            .block(genesis_hash.into())?
            .ok_or_else(|| eyre::eyre!("genesis block not found"))?;

        let provider = BlockchainProvider::new(provider_factory)?;

        let exex_head_block = random_block(
            &mut rng,
            genesis_block.number + 1,
            BlockParams { parent: Some(genesis_hash), tx_count: Some(0), ..Default::default() },
        );
        let exex_head_notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![exex_head_block.clone().try_recover()?],
                Default::default(),
                None,
            )),
        };
        wal.commit(&exex_head_notification)?;

        let node_head = BlockNumHash { number: genesis_block.number, hash: genesis_hash };
        let exex_head = ExExHead {
            block: BlockNumHash { number: exex_head_block.number, hash: exex_head_block.hash() },
        };

        let new_notification = ExExNotification::ChainCommitted {
            new: Arc::new(Chain::new(
                vec![random_block(
                    &mut rng,
                    genesis_block.number + 1,
                    BlockParams { parent: Some(genesis_hash), ..Default::default() },
                )
                .try_recover()?],
                Default::default(),
                None,
            )),
        };

        let (notifications_tx, notifications_rx) = mpsc::channel(1);

        notifications_tx.send(new_notification.clone()).await?;

        let mut notifications = ExExNotificationsWithoutHead::new(
            node_head,
            provider,
            EthEvmConfig::mainnet(),
            notifications_rx,
            wal.handle(),
        )
        .with_head(exex_head);

        // First notification is the revert of the ExEx head block to get back to the canonical
        // chain
        assert_eq!(
            notifications.next().await.transpose()?,
            Some(exex_head_notification.into_inverted())
        );

        // Second notification is the actual notification that we sent before
        assert_eq!(notifications.next().await.transpose()?, Some(new_notification));

        Ok(())
    }
}
