// LESSON 4: The Block Module - Containers of History
// This module defines block types - the fundamental building blocks of the blockchain.
// Each block contains a header (metadata) and a body (transactions).

use alloy_consensus::Header;
use reth_ethereum_primitives::TransactionSigned;
#[cfg(any(test, feature = "arbitrary"))]
pub use reth_primitives_traits::test_utils::{generate_valid_header, valid_header_strategy};

/// Ethereum full block.
///
/// LESSON 4: Type Aliases for Flexibility
/// Notice how Block is a type alias with default type parameters:
/// - T defaults to TransactionSigned (the transaction type)
/// - H defaults to Header (the header type)
/// This allows different chains (like Optimism) to use their own transaction/header types
/// while sharing the same block structure. It's like having a standard shipping container
/// that can hold different types of cargo!
///
/// Withdrawals can be optionally included at the end of the RLP encoded message.
pub type Block<T = TransactionSigned, H = Header> = alloy_consensus::Block<T, H>;

/// A response to `GetBlockBodies`, containing bodies if any bodies were found.
///
/// LESSON 4: Network Protocol Types
/// This type is specifically for the P2P protocol. When a node asks for block bodies
/// (transactions without headers), this is what gets sent back. Separating headers
/// and bodies allows efficient syncing - you can download headers first to verify
/// the chain, then fetch bodies later.
///
/// Withdrawals can be optionally included at the end of the RLP encoded message.
pub type BlockBody<T = TransactionSigned, H = Header> = alloy_consensus::BlockBody<T, H>;

/// Ethereum sealed block type
// LESSON 4: The Sealed Block Pattern
// A "sealed" block has its hash computed and cached. This is brilliant because:
// 1. Computing a block hash (Keccak-256 of RLP-encoded header) is expensive
// 2. We need the hash frequently (for lookups, references, etc.)
// 3. Blocks are immutable once created
// So we compute once, cache forever! The type system ensures we can't accidentally
// use an unsealed block where we need the hash.
pub type SealedBlock<B = Block> = reth_primitives_traits::block::SealedBlock<B>;

/// Helper type for constructing the block
// LESSON 4: API Evolution Through Deprecation
// These deprecated types show how Reth evolved its API. Previously, blocks with
// recovered senders were called "BlockWithSenders" or "SealedBlockWithSenders".
// Now they're uniformly called "RecoveredBlock". The old names remain as aliases
// to avoid breaking existing code - a thoughtful approach to API design!
#[deprecated(note = "Use `RecoveredBlock` instead")]
pub type SealedBlockFor<B = Block> = reth_primitives_traits::block::SealedBlock<B>;

/// Ethereum recovered block
#[deprecated(note = "Use `RecoveredBlock` instead")]
pub type BlockWithSenders<B = Block> = reth_primitives_traits::block::RecoveredBlock<B>;

/// Ethereum recovered block
// LESSON 4: The Recovered Block Type
// A "recovered" block has all transaction senders computed from signatures.
// This is expensive (ECDSA recovery for each transaction) but necessary for
// execution. The type system tracks whether this work has been done, preventing
// us from accidentally trying to execute a block without known senders.
#[deprecated(note = "Use `RecoveredBlock` instead")]
pub type SealedBlockWithSenders<B = Block> = reth_primitives_traits::block::RecoveredBlock<B>;
