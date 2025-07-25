use crate::{segment::PrunePurpose, PruneSegment, PruneSegmentError};
use alloy_primitives::BlockNumber;

/// Prune mode.
///
/// LESSON 21: Pruning Modes - Different Strategies for Data Retention
/// These modes control how much historical data to keep. 'Distance' keeps
/// the last N blocks, 'Before' keeps everything before a specific block,
/// and 'Full' removes everything possible while maintaining node functionality.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[cfg_attr(any(test, feature = "test-utils"), derive(arbitrary::Arbitrary))]
#[cfg_attr(any(test, feature = "reth-codec"), derive(reth_codecs::Compact))]
#[cfg_attr(any(test, feature = "reth-codec"), reth_codecs::add_arbitrary_tests(compact))]
#[cfg_attr(any(test, feature = "serde"), derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(any(test, feature = "serde"), serde(rename_all = "lowercase"))]
pub enum PruneMode {
    /// Prune all blocks.
    Full,
    /// Prune blocks before the `head-N` block number. In other words, keep last N + 1 blocks.
    Distance(u64),
    /// Prune blocks before the specified block number. The specified block number is not pruned.
    Before(BlockNumber),
}

#[cfg(any(test, feature = "test-utils"))]
impl Default for PruneMode {
    fn default() -> Self {
        Self::Full
    }
}

impl PruneMode {
    /// Prune blocks up to the specified block number. The specified block number is also pruned.
    ///
    /// This acts as `PruneMode::Before(block_number + 1)`.
    pub const fn before_inclusive(block_number: BlockNumber) -> Self {
        Self::Before(block_number + 1)
    }

    /// Returns block up to which variant pruning needs to be done, inclusive, according to the
    /// provided tip.
    pub fn prune_target_block(
        &self,
        tip: BlockNumber,
        segment: PruneSegment,
        purpose: PrunePurpose,
    ) -> Result<Option<(BlockNumber, Self)>, PruneSegmentError> {
        let result = match self {
            Self::Full if segment.min_blocks(purpose) == 0 => Some((tip, *self)),
            Self::Distance(distance) if *distance > tip => None, // Nothing to prune yet
            Self::Distance(distance) if *distance >= segment.min_blocks(purpose) => {
                Some((tip - distance, *self))
            }
            Self::Before(n) if *n == tip + 1 && purpose.is_static_file() => Some((tip, *self)),
            Self::Before(n) if *n > tip => None, // Nothing to prune yet
            Self::Before(n) => {
                (tip - n >= segment.min_blocks(purpose)).then(|| ((*n).saturating_sub(1), *self))
            }
            _ => return Err(PruneSegmentError::Configuration(segment)),
        };
        Ok(result)
    }

    /// Check if target block should be pruned according to the provided prune mode and tip.
    pub const fn should_prune(&self, block: BlockNumber, tip: BlockNumber) -> bool {
        match self {
            Self::Full => true,
            Self::Distance(distance) => {
                if *distance > tip {
                    return false
                }
                block < tip - *distance
            }
            Self::Before(n) => *n > block,
        }
    }

    /// Returns true if the prune mode is [`PruneMode::Full`].
    pub const fn is_full(&self) -> bool {
        matches!(self, Self::Full)
    }

    /// Returns true if the prune mode is [`PruneMode::Distance`].
    pub const fn is_distance(&self) -> bool {
        matches!(self, Self::Distance(_))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        PruneMode, PrunePurpose, PruneSegment, PruneSegmentError, MINIMUM_PRUNING_DISTANCE,
    };
    use assert_matches::assert_matches;
    use serde::Deserialize;

    #[test]
    fn test_prune_target_block() {
        let tip = 20000;
        let segment = PruneSegment::Receipts;

        let tests = vec![
            // MINIMUM_PRUNING_DISTANCE makes this impossible
            (PruneMode::Full, Err(PruneSegmentError::Configuration(segment))),
            // Nothing to prune
            (PruneMode::Distance(tip + 1), Ok(None)),
            (
                PruneMode::Distance(segment.min_blocks(PrunePurpose::User) + 1),
                Ok(Some(tip - (segment.min_blocks(PrunePurpose::User) + 1))),
            ),
            // Nothing to prune
            (PruneMode::Before(tip + 1), Ok(None)),
            (
                PruneMode::Before(tip - MINIMUM_PRUNING_DISTANCE),
                Ok(Some(tip - MINIMUM_PRUNING_DISTANCE - 1)),
            ),
            (
                PruneMode::Before(tip - MINIMUM_PRUNING_DISTANCE - 1),
                Ok(Some(tip - MINIMUM_PRUNING_DISTANCE - 2)),
            ),
            // Nothing to prune
            (PruneMode::Before(tip - 1), Ok(None)),
        ];

        for (index, (mode, expected_result)) in tests.into_iter().enumerate() {
            assert_eq!(
                mode.prune_target_block(tip, segment, PrunePurpose::User),
                expected_result.map(|r| r.map(|b| (b, mode))),
                "Test {} failed",
                index + 1,
            );
        }

        // Test for a scenario where there are no minimum blocks and Full can be used
        assert_eq!(
            PruneMode::Full.prune_target_block(tip, PruneSegment::Transactions, PrunePurpose::User),
            Ok(Some((tip, PruneMode::Full))),
        );
    }

    #[test]
    fn test_should_prune() {
        let tip = 20000;
        let should_prune = true;

        let tests = vec![
            (PruneMode::Distance(tip + 1), 1, !should_prune),
            (
                PruneMode::Distance(MINIMUM_PRUNING_DISTANCE + 1),
                tip - MINIMUM_PRUNING_DISTANCE - 1,
                !should_prune,
            ),
            (
                PruneMode::Distance(MINIMUM_PRUNING_DISTANCE + 1),
                tip - MINIMUM_PRUNING_DISTANCE - 2,
                should_prune,
            ),
            (PruneMode::Before(tip + 1), 1, should_prune),
            (PruneMode::Before(tip + 1), tip + 1, !should_prune),
        ];

        for (index, (mode, block, expected_result)) in tests.into_iter().enumerate() {
            assert_eq!(mode.should_prune(block, tip), expected_result, "Test {} failed", index + 1,);
        }
    }

    #[test]
    fn prune_mode_deserialize() {
        #[derive(Debug, Deserialize)]
        struct Config {
            a: Option<PruneMode>,
            b: Option<PruneMode>,
            c: Option<PruneMode>,
            d: Option<PruneMode>,
        }

        let toml_str = r#"
        a = "full"
        b = { distance = 10 }
        c = { before = 20 }
    "#;

        assert_matches!(
            toml::from_str(toml_str),
            Ok(Config {
                a: Some(PruneMode::Full),
                b: Some(PruneMode::Distance(10)),
                c: Some(PruneMode::Before(20)),
                d: None
            })
        );
    }
}
