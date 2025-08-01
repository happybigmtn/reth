use crate::segments::Segment;
use alloy_primitives::BlockNumber;
use reth_codecs::Compact;
use reth_db_api::{cursor::DbCursorRO, table::Value, tables, transaction::DbTx};
use reth_primitives_traits::NodePrimitives;
use reth_provider::{providers::StaticFileWriter, DBProvider, StaticFileProviderFactory};
use reth_static_file_types::StaticFileSegment;
use reth_storage_errors::provider::ProviderResult;
use std::ops::RangeInclusive;

/// Static File segment responsible for [`StaticFileSegment::Headers`] part of data.
#[derive(Debug, Default)]
pub struct Headers;

impl<Provider> Segment<Provider> for Headers
where
    Provider: StaticFileProviderFactory<Primitives: NodePrimitives<BlockHeader: Compact + Value>>
        + DBProvider,
{
    fn segment(&self) -> StaticFileSegment {
        StaticFileSegment::Headers
    }

    fn copy_to_static_files(
        &self,
        provider: Provider,
        block_range: RangeInclusive<BlockNumber>,
    ) -> ProviderResult<()> {
        let static_file_provider = provider.static_file_provider();
        let mut static_file_writer =
            static_file_provider.get_writer(*block_range.start(), StaticFileSegment::Headers)?;

        // LESSON 11: Moving Headers to Static Files
        // We need to read from three tables to get complete header data:
        // 1. Headers - The actual header data
        // 2. HeaderTerminalDifficulties - Total difficulty (for pre-merge)
        // 3. CanonicalHeaders - Which headers are canonical
        let mut headers_cursor = provider
            .tx_ref()
            .cursor_read::<tables::Headers<<Provider::Primitives as NodePrimitives>::BlockHeader>>(
            )?;
        let headers_walker = headers_cursor.walk_range(block_range.clone())?;

        let mut header_td_cursor =
            provider.tx_ref().cursor_read::<tables::HeaderTerminalDifficulties>()?;
        let header_td_walker = header_td_cursor.walk_range(block_range.clone())?;

        let mut canonical_headers_cursor =
            provider.tx_ref().cursor_read::<tables::CanonicalHeaders>()?;
        let canonical_headers_walker = canonical_headers_cursor.walk_range(block_range)?;

        for ((header_entry, header_td_entry), canonical_header_entry) in
            headers_walker.zip(header_td_walker).zip(canonical_headers_walker)
        {
            let (header_block, header) = header_entry?;
            let (header_td_block, header_td) = header_td_entry?;
            let (canonical_header_block, canonical_header) = canonical_header_entry?;

            debug_assert_eq!(header_block, header_td_block);
            debug_assert_eq!(header_td_block, canonical_header_block);

            static_file_writer.append_header(&header, header_td.0, &canonical_header)?;
        }

        Ok(())
    }
}
