//! Static file producer implementation.
//! 
//! LESSON 11: Static Files - Efficient Immutable Storage
//! Static files store finalized blockchain data that never changes.
//! Instead of keeping everything in the database, we move immutable data
//! (headers, bodies, receipts) to append-only files. Benefits:
//! - Better compression ratios
//! - Faster sequential reads  
//! - Reduced database size
//! - Memory-mapped access

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]

pub mod segments;
mod static_file_producer;

pub use static_file_producer::{
    StaticFileProducer, StaticFileProducerInner, StaticFileProducerResult,
    StaticFileProducerWithResult,
};

// Re-export for convenience.
pub use reth_static_file_types::*;
