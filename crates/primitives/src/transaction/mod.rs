//! Transaction types.
//! 
//! LESSON 3: The Transaction Module - Where State Changes Begin
//! This module contains everything related to transactions - the fundamental unit
//! of state change in Ethereum. Every action on Ethereum starts as a transaction:
//! sending ETH, deploying contracts, calling functions, etc.
//!
//! Think of transactions as signed messages that say:
//! "I, the holder of this private key, want to do X and I'm willing to pay Y for it"

use crate::Recovered;
pub use alloy_consensus::transaction::PooledTransaction;
use once_cell as _;
#[expect(deprecated)]
pub use pooled::PooledTransactionsElementEcRecovered;

// LESSON 3: Traits and Type Organization
// Notice how types are organized by functionality:
// - error types for what can go wrong
// - signed types for transactions with signatures
// - traits like FillTxEnv for EVM integration
pub use reth_primitives_traits::{
    sync::{LazyLock, OnceLock},  // For lazy initialization (caching)
    transaction::{
        error::{
            InvalidTransactionError, TransactionConversionError, TryFromRecoveredTransactionError,
        },
        signed::SignedTransaction,
    },
    FillTxEnv, WithEncoded,
};

// LESSON 3: Signature Recovery
// These functions recover the sender's address from a transaction signature.
// This is computationally expensive, so we cache the result!
pub use signature::{recover_signer, recover_signer_unchecked};
pub use tx_type::TxType;

/// Handling transaction signature operations, including signature recovery,
/// applying chain IDs, and EIP-2 validation.
// LESSON 3: Modular Code Organization
// Each submodule handles a specific aspect:
// - signature: cryptographic operations
// - util: helper functions
// - pooled: mempool-specific types
// - tx_type: transaction type identification
pub mod signature;
pub mod util;

mod pooled;
mod tx_type;

/// Signed transaction.
// LESSON 3: The Core Transaction Types
// Transaction: The enum containing all transaction variants (Legacy, EIP-1559, EIP-4844)
// TransactionSigned: A transaction with its signature attached
// These are defined in reth_ethereum_primitives because they're Ethereum-specific
pub use reth_ethereum_primitives::{Transaction, TransactionSigned};

/// Type alias kept for backward compatibility.
// LESSON 3: Graceful API Evolution
// When APIs change, we keep old names as deprecated aliases.
// This gives users time to migrate without breaking their code immediately.
// The Recovered<T> type wraps a transaction with its recovered sender address.
#[deprecated(note = "Use `Recovered` instead")]
pub type TransactionSignedEcRecovered<T = TransactionSigned> = Recovered<T>;
