//! `StaticFile` segment implementations and utilities.

mod transactions;
pub use transactions::Transactions;

mod headers;
pub use headers::Headers;

mod receipts;
pub use receipts::Receipts;

use alloy_primitives::BlockNumber;
use reth_provider::StaticFileProviderFactory;
use reth_static_file_types::StaticFileSegment;
use reth_storage_errors::provider::ProviderResult;
use std::ops::RangeInclusive;

/// A segment represents moving some portion of the data to static files.
// LESSON 11: The Segment Trait
// Each segment type (Headers, Transactions, Receipts) implements this trait.
// The segment is responsible for:
// 1. Reading data from the database
// 2. Writing it to static files in the correct format
// 3. Maintaining consistency during the migration
pub trait Segment<Provider: StaticFileProviderFactory>: Send + Sync {
    /// Returns the [`StaticFileSegment`].
    fn segment(&self) -> StaticFileSegment;

    /// Move data to static files for the provided block range.
    /// [`StaticFileProvider`](reth_provider::providers::StaticFileProvider) will handle
    /// the management of and writing to files.
    fn copy_to_static_files(
        &self,
        provider: Provider,
        block_range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<()>;
}
