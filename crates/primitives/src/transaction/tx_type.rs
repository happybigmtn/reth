/// Transaction Type
///
/// LESSON 3: Transaction Type Identifiers
/// Ethereum uses a single byte to identify transaction types:
/// - 0x00 (or missing): Legacy transaction
/// - 0x01: EIP-2930 (access list)
/// - 0x02: EIP-1559 (dynamic fees)
/// - 0x03: EIP-4844 (blob transactions)
///
/// This is brilliant! Old nodes that don't understand new types will just skip them.
/// It's like how email evolved - old clients ignore new features they don't understand.
///
/// Currently being used as 2-bit type when encoding it to `reth_codecs::Compact` on
/// [`crate::TransactionSigned`]. Adding more transaction types will break the codec and
/// database format.
///
/// LESSON 3: Database Format Considerations
/// The comment about "2-bit type" is important! Reth optimizes storage by using only
/// 2 bits for the type (supporting 4 types: 00, 01, 10, 11). This saves space but
/// means adding a 5th type requires a database migration. Trade-offs everywhere!
///
/// Other required changes when adding a new type can be seen on [PR#3953](https://github.com/paradigmxyz/reth/pull/3953/files).
pub use alloy_consensus::TxType;
