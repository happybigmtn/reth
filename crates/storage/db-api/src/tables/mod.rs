//! Tables and data models.
//!
//! # Overview
//!
//! This module defines the tables in reth, as well as some table-related abstractions:
//!
//! - [`codecs`] integrates different codecs into [`Encode`] and [`Decode`]
//! - [`models`](crate::models) defines the values written to tables
//!
//! LESSON 7: Database Schema Definition
//! This is where all of Reth's database tables are defined. Each table has:
//! - A unique name (used as the table identifier in MDBX)
//! - A key type (what we search by)
//! - A value type (what we store)
//! - Optional subkey for DupSort tables (tables with duplicate keys)
//! 
//! The schema is carefully designed for:
//! - Efficient queries (keys are ordered for range scans)
//! - Space efficiency (deduplication, compression)
//! - Type safety (can't put wrong data in tables)

pub mod codecs;

mod raw;
pub use raw::{RawDupSort, RawKey, RawTable, RawValue, TableRawRow};

use crate::{
    models::{
        accounts::BlockNumberAddress,
        blocks::{HeaderHash, StoredBlockOmmers},
        storage_sharded_key::StorageShardedKey,
        AccountBeforeTx, ClientVersion, CompactU256, IntegerList, ShardedKey,
        StoredBlockBodyIndices, StoredBlockWithdrawals,
    },
    table::{Decode, DupSort, Encode, Table, TableInfo},
};
use alloy_consensus::Header;
use alloy_primitives::{Address, BlockHash, BlockNumber, TxHash, TxNumber, B256};
use reth_ethereum_primitives::{Receipt, TransactionSigned};
use reth_primitives_traits::{Account, Bytecode, StorageEntry};
use reth_prune_types::{PruneCheckpoint, PruneSegment};
use reth_stages_types::StageCheckpoint;
use reth_trie_common::{BranchNodeCompact, StorageTrieEntry, StoredNibbles, StoredNibblesSubKey};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Enum for the types of tables present in libmdbx.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TableType {
    /// key value table
    Table,
    /// Duplicate key value table
    DupSort,
}

/// The general purpose of this is to use with a combination of Tables enum,
/// by implementing a `TableViewer` trait you can operate on db tables in an abstract way.
///
/// # Example
///
/// ```
/// use reth_db_api::{
///     table::{DupSort, Table},
///     TableViewer, Tables,
/// };
///
/// struct MyTableViewer;
///
/// impl TableViewer<()> for MyTableViewer {
///     type Error = &'static str;
///
///     fn view<T: Table>(&self) -> Result<(), Self::Error> {
///         // operate on table in a generic way
///         Ok(())
///     }
///
///     fn view_dupsort<T: DupSort>(&self) -> Result<(), Self::Error> {
///         // operate on a dupsort table in a generic way
///         Ok(())
///     }
/// }
///
/// let viewer = MyTableViewer {};
///
/// let _ = Tables::Headers.view(&viewer);
/// let _ = Tables::Transactions.view(&viewer);
/// ```
pub trait TableViewer<R> {
    /// The error type returned by the viewer.
    type Error;

    /// Calls `view` with the correct table type.
    fn view_rt(&self, table: Tables) -> Result<R, Self::Error> {
        table.view(self)
    }

    /// Operate on the table in a generic way.
    fn view<T: Table>(&self) -> Result<R, Self::Error>;

    /// Operate on the dupsort table in a generic way.
    ///
    /// By default, the `view` function is invoked unless overridden.
    fn view_dupsort<T: DupSort>(&self) -> Result<R, Self::Error> {
        self.view::<T>()
    }
}

/// General trait for defining the set of tables
/// Used to initialize database
pub trait TableSet {
    /// Returns an iterator over the tables
    fn tables() -> Box<dyn Iterator<Item = Box<dyn TableInfo>>>;
}

/// Defines all the tables in the database.
#[macro_export]
macro_rules! tables {
    (@bool) => { false };
    (@bool $($t:tt)+) => { true };

    (@view $name:ident $v:ident) => { $v.view::<$name>() };
    (@view $name:ident $v:ident $_subkey:ty) => { $v.view_dupsort::<$name>() };

    (@value_doc $key:ty, $value:ty) => {
        concat!("[`", stringify!($value), "`]")
    };
    // Don't generate links if we have generics
    (@value_doc $key:ty, $value:ty, $($generic:ident),*) => {
        concat!("`", stringify!($value), "`")
    };

    ($($(#[$attr:meta])* table $name:ident$(<$($generic:ident $(= $default:ty)?),*>)? { type Key = $key:ty; type Value = $value:ty; $(type SubKey = $subkey:ty;)? } )*) => {
        // Table marker types.
        $(
            $(#[$attr])*
            ///
            #[doc = concat!("Marker type representing a database table mapping [`", stringify!($key), "`] to ", tables!(@value_doc $key, $value, $($($generic),*)?), ".")]
            $(
                #[doc = concat!("\n\nThis table's `DUPSORT` subkey is [`", stringify!($subkey), "`].")]
            )?
            pub struct $name$(<$($generic $( = $default)?),*>)? {
                _private: std::marker::PhantomData<($($($generic,)*)?)>,
            }

            // Ideally this implementation wouldn't exist, but it is necessary to derive `Debug`
            // when a type is generic over `T: Table`. See: https://github.com/rust-lang/rust/issues/26925
            impl$(<$($generic),*>)? fmt::Debug for $name$(<$($generic),*>)? {
                fn fmt(&self, _: &mut fmt::Formatter<'_>) -> fmt::Result {
                    unreachable!("this type cannot be instantiated")
                }
            }

            impl$(<$($generic),*>)? $crate::table::Table for $name$(<$($generic),*>)?
            where
                $value: $crate::table::Value + 'static
                $($(,$generic: Send + Sync)*)?
            {
                const NAME: &'static str = table_names::$name;
                const DUPSORT: bool = tables!(@bool $($subkey)?);

                type Key = $key;
                type Value = $value;
            }

            $(
                impl DupSort for $name {
                    type SubKey = $subkey;
                }
            )?
        )*

        // Tables enum.

        /// A table in the database.
        #[derive(Clone, Copy, PartialEq, Eq, Hash)]
        pub enum Tables {
            $(
                #[doc = concat!("The [`", stringify!($name), "`] database table.")]
                $name,
            )*
        }

        impl Tables {
            /// All the tables in the database.
            pub const ALL: &'static [Self] = &[$(Self::$name,)*];

            /// The number of tables in the database.
            pub const COUNT: usize = Self::ALL.len();

            /// Returns the name of the table as a string.
            pub const fn name(&self) -> &'static str {
                match self {
                    $(
                        Self::$name => table_names::$name,
                    )*
                }
            }

            /// Returns `true` if the table is a `DUPSORT` table.
            pub const fn is_dupsort(&self) -> bool {
                match self {
                    $(
                        Self::$name => tables!(@bool $($subkey)?),
                    )*
                }
            }

            /// The type of the given table in database.
            pub const fn table_type(&self) -> TableType {
                if self.is_dupsort() {
                    TableType::DupSort
                } else {
                    TableType::Table
                }
            }

            /// Allows to operate on specific table type
            pub fn view<T, R>(&self, visitor: &T) -> Result<R, T::Error>
            where
                T: ?Sized + TableViewer<R>,
            {
                match self {
                    $(
                        Self::$name => tables!(@view $name visitor $($subkey)?),
                    )*
                }
            }
        }

        impl fmt::Debug for Tables {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(self.name())
            }
        }

        impl fmt::Display for Tables {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                self.name().fmt(f)
            }
        }

        impl std::str::FromStr for Tables {
            type Err = String;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    $(
                        table_names::$name => Ok(Self::$name),
                    )*
                    s => Err(format!("unknown table: {s:?}")),
                }
            }
        }

        impl TableInfo for Tables {
            fn name(&self) -> &'static str {
                self.name()
            }

            fn is_dupsort(&self) -> bool {
                self.is_dupsort()
            }
        }

        impl TableSet for Tables {
            fn tables() -> Box<dyn Iterator<Item = Box<dyn TableInfo>>> {
                Box::new(Self::ALL.iter().map(|table| Box::new(*table) as Box<dyn TableInfo>))
            }
        }

        // Need constants to match on in the `FromStr` implementation.
        #[expect(non_upper_case_globals)]
        mod table_names {
            $(
                pub(super) const $name: &'static str = stringify!($name);
            )*
        }

        /// Maps a run-time [`Tables`] enum value to its corresponding compile-time [`Table`] type.
        ///
        /// This is a simpler alternative to [`TableViewer`].
        ///
        /// # Examples
        ///
        /// ```
        /// use reth_db_api::{table::Table, Tables, tables_to_generic};
        ///
        /// let table = Tables::Headers;
        /// let result = tables_to_generic!(table, |GenericTable| <GenericTable as Table>::NAME);
        /// assert_eq!(result, table.name());
        /// ```
        #[macro_export]
        macro_rules! tables_to_generic {
            ($table:expr, |$generic_name:ident| $e:expr) => {
                match $table {
                    $(
                        Tables::$name => {
                            use $crate::tables::$name as $generic_name;
                            $e
                        },
                    )*
                }
            };
        }
    };
}

tables! {
    /// Stores the header hashes belonging to the canonical chain.
    // LESSON 7: Canonical Chain Tracking
    // This table maps block number → hash for the canonical (main) chain.
    // When reorgs happen, this table is updated to point to the new canonical blocks.
    table CanonicalHeaders {
        type Key = BlockNumber;
        type Value = HeaderHash;
    }

    /// Stores the total difficulty from a block header.
    // LESSON 7: Terminal Difficulty for The Merge
    // This tracked total difficulty until The Merge. Post-merge, difficulty is always 0.
    // Kept for historical blocks and consensus verification.
    table HeaderTerminalDifficulties {
        type Key = BlockNumber;
        type Value = CompactU256;
    }

    /// Stores the block number corresponding to a header.
    // LESSON 7: Hash to Number Mapping
    // Reverse lookup: given a block hash, find its height.
    // Essential for RPC methods like eth_getBlockByHash.
    table HeaderNumbers {
        type Key = BlockHash;
        type Value = BlockNumber;
    }

    /// Stores header bodies.
    // LESSON 7: Generic Header Storage
    // Generic over H to support different header types (Ethereum, Optimism, etc.)
    // Headers are accessed frequently, so they're stored separately from bodies.
    table Headers<H = Header> {
        type Key = BlockNumber;
        type Value = H;
    }

    /// Stores block indices that contains indexes of transaction and the count of them.
    ///
    /// More information about stored indices can be found in the [`StoredBlockBodyIndices`] struct.
    // LESSON 7: Block Body Indices
    // Instead of storing full bodies, we store indices pointing to transactions.
    // This enables deduplication - transactions are stored once and referenced.
    table BlockBodyIndices {
        type Key = BlockNumber;
        type Value = StoredBlockBodyIndices;
    }

    /// Stores the uncles/ommers of the block.
    // LESSON 7: Uncle/Ommer Storage
    // Pre-merge, miners could include uncle blocks for partial rewards.
    // Post-merge, this is always empty but kept for historical data.
    table BlockOmmers<H = Header> {
        type Key = BlockNumber;
        type Value = StoredBlockOmmers<H>;
    }

    /// Stores the block withdrawals.
    // LESSON 7: Staking Withdrawals (Post-Shanghai)
    // Validators can withdraw staking rewards and principal.
    // Each block can contain multiple withdrawals.
    table BlockWithdrawals {
        type Key = BlockNumber;
        type Value = StoredBlockWithdrawals;
    }

    /// Canonical only Stores the transaction body for canonical transactions.
    // LESSON 7: Transaction Storage with Global Numbering
    // Transactions get a global TxNumber for efficient indexing.
    // This allows deduplication and fast range queries.
    table Transactions<T = TransactionSigned> {
        type Key = TxNumber;
        type Value = T;
    }

    /// Stores the mapping of the transaction hash to the transaction number.
    // LESSON 7: Transaction Hash Index
    // Two-step lookup: Hash → TxNumber → Transaction
    // Saves space since hash is stored only once.
    table TransactionHashNumbers {
        type Key = TxHash;
        type Value = TxNumber;
    }

    /// Stores the mapping of transaction number to the blocks number.
    ///
    /// The key is the highest transaction ID in the block.
    // LESSON 7: Transaction to Block Mapping
    // Key insight: We store the LAST tx number in each block.
    // To find a tx's block: seek to the first key >= tx_number.
    table TransactionBlocks {
        type Key = TxNumber;
        type Value = BlockNumber;
    }

    /// Canonical only Stores transaction receipts.
    // LESSON 7: Receipt Storage
    // Receipts contain execution results: status, gas used, logs.
    // Stored by TxNumber for consistency with transactions.
    table Receipts<R = Receipt> {
        type Key = TxNumber;
        type Value = R;
    }

    /// Stores all smart contract bytecodes.
    /// There will be multiple accounts that have same bytecode
    /// So we would need to introduce reference counter.
    /// This will be small optimization on state.
    // LESSON 7: Bytecode Deduplication
    // Many contracts share bytecode (e.g., proxy contracts, token contracts).
    // We store each unique bytecode once, keyed by its hash.
    table Bytecodes {
        type Key = B256;
        type Value = Bytecode;
    }

    /// Stores the current state of an [`Account`].
    // LESSON 7: Current Account State
    // "Plain" means unhashed addresses (vs HashedAccountState for tries).
    // This is the latest state - what eth_getBalance returns.
    table PlainAccountState {
        type Key = Address;
        type Value = Account;
    }

    /// Stores the current value of a storage key.
    // LESSON 7: Contract Storage - DupSort Table
    // This is a DupSort table: multiple storage slots per address.
    // Key = Address, SubKey = StorageKey (B256)
    // Enables efficient "get all storage for address" queries.
    table PlainStorageState {
        type Key = Address;
        type Value = StorageEntry;
        type SubKey = B256;
    }

    /// Stores pointers to block changeset with changes for each account key.
    ///
    /// Last shard key of the storage will contain `u64::MAX` `BlockNumber`,
    /// this would allows us small optimization on db access when change is in plain state.
    ///
    /// Imagine having shards as:
    /// * `Address | 100`
    /// * `Address | u64::MAX`
    ///
    /// What we need to find is number that is one greater than N. Db `seek` function allows us to fetch
    /// the shard that equal or more than asked. For example:
    /// * For N=50 we would get first shard.
    /// * for N=150 we would get second shard.
    /// * If max block number is 200 and we ask for N=250 we would fetch last shard and know that needed entry is in `AccountPlainState`.
    /// * If there were no shard we would get `None` entry or entry of different storage key.
    ///
    /// Code example can be found in `reth_provider::HistoricalStateProviderRef`
    // LESSON 7: Sharded History Index
    // ShardedKey = (Address, HighestBlockInShard)
    // Value = List of blocks where this account changed
    // Sharding improves query performance for "what did account X look like at block Y?"
    table AccountsHistory {
        type Key = ShardedKey<Address>;
        type Value = BlockNumberList;
    }

    /// Stores pointers to block number changeset with changes for each storage key.
    ///
    /// Last shard key of the storage will contain `u64::MAX` `BlockNumber`,
    /// this would allows us small optimization on db access when change is in plain state.
    ///
    /// Imagine having shards as:
    /// * `Address | StorageKey | 100`
    /// * `Address | StorageKey | u64::MAX`
    ///
    /// What we need to find is number that is one greater than N. Db `seek` function allows us to fetch
    /// the shard that equal or more than asked. For example:
    /// * For N=50 we would get first shard.
    /// * for N=150 we would get second shard.
    /// * If max block number is 200 and we ask for N=250 we would fetch last shard and know that needed entry is in `StoragePlainState`.
    /// * If there were no shard we would get `None` entry or entry of different storage key.
    ///
    /// Code example can be found in `reth_provider::HistoricalStateProviderRef`
    table StoragesHistory {
        type Key = StorageShardedKey;
        type Value = BlockNumberList;
    }

    /// Stores the state of an account before a certain transaction changed it.
    /// Change on state can be: account is created, selfdestructed, touched while empty
    /// or changed balance,nonce.
    table AccountChangeSets {
        type Key = BlockNumber;
        type Value = AccountBeforeTx;
        type SubKey = Address;
    }

    /// Stores the state of a storage key before a certain transaction changed it.
    /// If [`StorageEntry::value`] is zero, this means storage was not existing
    /// and needs to be removed.
    table StorageChangeSets {
        type Key = BlockNumberAddress;
        type Value = StorageEntry;
        type SubKey = B256;
    }

    /// Stores the current state of an [`Account`] indexed with `keccak256Address`
    /// This table is in preparation for merklization and calculation of state root.
    /// We are saving whole account data as it is needed for partial update when
    /// part of storage is changed. Benefit for merklization is that hashed addresses are sorted.
    table HashedAccounts {
        type Key = B256;
        type Value = Account;
    }

    /// Stores the current storage values indexed with `keccak256Address` and
    /// hash of storage key `keccak256key`.
    /// This table is in preparation for merklization and calculation of state root.
    /// Benefit for merklization is that hashed addresses/keys are sorted.
    table HashedStorages {
        type Key = B256;
        type Value = StorageEntry;
        type SubKey = B256;
    }

    /// Stores the current state's Merkle Patricia Tree.
    table AccountsTrie {
        type Key = StoredNibbles;
        type Value = BranchNodeCompact;
    }

    /// From `HashedAddress` => `NibblesSubKey` => Intermediate value
    table StoragesTrie {
        type Key = B256;
        type Value = StorageTrieEntry;
        type SubKey = StoredNibblesSubKey;
    }

    /// Stores the transaction sender for each canonical transaction.
    /// It is needed to speed up execution stage and allows fetching signer without doing
    /// transaction signed recovery
    table TransactionSenders {
        type Key = TxNumber;
        type Value = Address;
    }

    /// Stores the highest synced block number and stage-specific checkpoint of each stage.
    table StageCheckpoints {
        type Key = StageId;
        type Value = StageCheckpoint;
    }

    /// Stores arbitrary data to keep track of a stage first-sync progress.
    table StageCheckpointProgresses {
        type Key = StageId;
        type Value = Vec<u8>;
    }

    /// Stores the highest pruned block number and prune mode of each prune segment.
    table PruneCheckpoints {
        type Key = PruneSegment;
        type Value = PruneCheckpoint;
    }

    /// Stores the history of client versions that have accessed the database with write privileges by unix timestamp in seconds.
    table VersionHistory {
        type Key = u64;
        type Value = ClientVersion;
    }

    /// Stores generic chain state info, like the last finalized block.
    table ChainState {
        type Key = ChainStateKey;
        type Value = BlockNumber;
    }
}

/// Keys for the `ChainState` table.
#[derive(Ord, Clone, Eq, PartialOrd, PartialEq, Debug, Deserialize, Serialize, Hash)]
pub enum ChainStateKey {
    /// Last finalized block key
    LastFinalizedBlock,
    /// Last finalized block key
    LastSafeBlockBlock,
}

impl Encode for ChainStateKey {
    type Encoded = [u8; 1];

    fn encode(self) -> Self::Encoded {
        match self {
            Self::LastFinalizedBlock => [0],
            Self::LastSafeBlockBlock => [1],
        }
    }
}

impl Decode for ChainStateKey {
    fn decode(value: &[u8]) -> Result<Self, crate::DatabaseError> {
        match value {
            [0] => Ok(Self::LastFinalizedBlock),
            [1] => Ok(Self::LastSafeBlockBlock),
            _ => Err(crate::DatabaseError::Decode),
        }
    }
}

// Alias types.

/// List with transaction numbers.
pub type BlockNumberList = IntegerList;

/// Encoded stage id.
pub type StageId = String;

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn parse_table_from_str() {
        for table in Tables::ALL {
            assert_eq!(format!("{table:?}"), table.name());
            assert_eq!(table.to_string(), table.name());
            assert_eq!(Tables::from_str(table.name()).unwrap(), *table);
        }
    }
}
