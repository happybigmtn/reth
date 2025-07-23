//! Commonly used types in Reth.
//!
//! This crate contains Ethereum primitive types and helper functions.
//!
//! LESSON 2: The Primitives Crate - Your Type System Toolbox
//! This is where all the fundamental types live. Think of it as the "standard library"
//! for Ethereum types. Every other part of Reth builds on these primitives.
//! 
//! Just like how in physics we have fundamental particles (electrons, protons, neutrons),
//! in Ethereum we have fundamental types:
//! - Addresses (20 bytes) - where accounts live
//! - Hashes (32 bytes) - unique identifiers for data
//! - Numbers (U256) - for balances, nonces, gas
//! - Transactions - the messages that change state
//! - Blocks - containers of transactions
//! - Receipts - proof that transactions were executed
//!
//! ## Feature Flags
//!
//! LESSON 2: Rust's Feature Flags - Conditional Compilation
//! Feature flags let us include or exclude code at compile time.
//! It's like having different versions of a recipe - basic or gourmet!
//! 
//! - `arbitrary`: Adds `proptest` and `arbitrary` support for primitive types.
//! - `test-utils`: Export utilities for testing
//! - `reth-codec`: Enables db codec support for reth types including zstd compression for certain
//!   types.

#![doc(
    html_logo_url = "https://raw.githubusercontent.com/paradigmxyz/reth/main/assets/reth-docs.png",
    html_favicon_url = "https://avatars0.githubusercontent.com/u/97369466?s=256",
    issue_tracker_base_url = "https://github.com/paradigmxyz/reth/issues/"
)]
#![cfg_attr(not(test), warn(unused_crate_dependencies))]
#![cfg_attr(docsrs, feature(doc_cfg, doc_auto_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]

mod block;
mod receipt;
pub use reth_static_file_types as static_file;
pub mod transaction;
#[cfg(any(test, feature = "arbitrary"))]
pub use block::{generate_valid_header, valid_header_strategy};
pub use block::{Block, BlockBody, SealedBlock};
#[expect(deprecated)]
pub use block::{BlockWithSenders, SealedBlockFor, SealedBlockWithSenders};

pub use receipt::{gas_spent_by_transactions, Receipt};
// LESSON 2: Re-exports - The Power of a Unified Interface
// Rust lets us re-export types from other crates. This is brilliant!
// Users only need to import from reth_primitives, not remember dozens of crate names.
// It's like having one phone number that connects you to all departments.
pub use reth_primitives_traits::{
    logs_bloom, Account, BlockTy, BodyTy, Bytecode, GotExpected, GotExpectedBoxed, Header,
    HeaderTy, Log, LogData, NodePrimitives, ReceiptTy, RecoveredBlock, SealedHeader, StorageEntry,
    TxTy,
};
pub use static_file::StaticFileSegment;

// LESSON 2: Alloy - The Shared Foundation
// Alloy is like the "standard library" for Ethereum in Rust.
// Multiple projects (Reth, Foundry, etc.) use these same types.
// This ensures compatibility - like how all electrical plugs follow a standard!
pub use alloy_consensus::{
    transaction::{PooledTransaction, Recovered, TransactionMeta},
    ReceiptWithBloom,
};

/// Recovered transaction
// LESSON 2: Deprecation in Rust
// The #[deprecated] attribute is like putting a "Please use the new entrance" sign.
// It warns developers at compile time, but doesn't break existing code.
// This is how libraries evolve without breaking their users' code!
#[deprecated(note = "use `Recovered` instead")]
pub type RecoveredTx<T> = Recovered<T>;

pub use transaction::{
    util::secp256k1::{public_key_to_address, recover_signer_unchecked, sign_message},
    InvalidTransactionError, Transaction, TransactionSigned, TxType,
};
#[expect(deprecated)]
pub use transaction::{PooledTransactionsElementEcRecovered, TransactionSignedEcRecovered};

// Re-exports
pub use reth_ethereum_forks::*;

#[cfg(any(test, feature = "arbitrary"))]
pub use arbitrary;

#[cfg(feature = "c-kzg")]
pub use c_kzg as kzg;

/// Bincode-compatible serde implementations for commonly used types in Reth.
///
/// `bincode` crate doesn't work with optionally serializable serde fields, but some of the
/// Reth types require optional serialization for RPC compatibility. This module makes so that
/// all fields are serialized.
///
/// Read more: <https://github.com/bincode-org/bincode/issues/326>
#[cfg(feature = "serde-bincode-compat")]
pub mod serde_bincode_compat {
    pub use reth_primitives_traits::serde_bincode_compat::*;
}

// Re-export of `EthPrimitives`
pub use reth_ethereum_primitives::EthPrimitives;
