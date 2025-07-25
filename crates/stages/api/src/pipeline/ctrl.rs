use alloy_eips::eip1898::BlockWithParent;
use alloy_primitives::BlockNumber;

/// Determines the control flow during pipeline execution.
///
/// See [`Pipeline::run_loop`](crate::Pipeline::run_loop) for more information.
///
/// LESSON 20: Pipeline Control Flow - Handling Reorgs in Staged Sync
/// This enum controls how the pipeline responds to different situations,
/// especially when reorgs are detected. The Unwind variant triggers
/// the staged sync unwinding process to handle chain reorganizations!
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ControlFlow {
    /// An unwind was requested and must be performed before continuing.
    Unwind {
        /// The block to unwind to.
        ///
        /// This marks the highest block to which the stage should unwind to.
        /// For example, unwinding to block 10, should remove all data for blocks above 10 (>=11).
        target: BlockNumber,
        /// The block that caused the unwind.
        bad_block: Box<BlockWithParent>,
    },
    /// The pipeline made progress.
    Continue {
        /// Block number reached by the stage.
        block_number: BlockNumber,
    },
    /// Pipeline made no progress
    NoProgress {
        /// Block number reached by the stage.
        block_number: Option<BlockNumber>,
    },
}

impl ControlFlow {
    /// Whether the pipeline should continue executing stages.
    pub const fn should_continue(&self) -> bool {
        matches!(self, Self::Continue { .. } | Self::NoProgress { .. })
    }

    /// Returns true if the control flow is unwind.
    pub const fn is_unwind(&self) -> bool {
        matches!(self, Self::Unwind { .. })
    }

    /// Returns the pipeline block number the stage reached, if the state is not `Unwind`.
    pub const fn block_number(&self) -> Option<BlockNumber> {
        match self {
            Self::Unwind { .. } => None,
            Self::Continue { block_number } => Some(*block_number),
            Self::NoProgress { block_number } => *block_number,
        }
    }
}
