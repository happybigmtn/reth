//! An integration of [`reth-trie`] with [`reth-db`].
//!
//! LESSON 17: Trie Database Integration - Connecting Tries to Storage
//! This module bridges the gap between the abstract trie algorithms and
//! the actual database. It provides cursors for navigating trie nodes
//! stored in the database and methods for computing state roots.

#![cfg_attr(not(test), warn(unused_crate_dependencies))]

mod commitment;
mod hashed_cursor;
mod prefix_set;
mod proof;
mod state;
mod storage;
mod trie_cursor;
mod witness;

pub use commitment::{MerklePatriciaTrie, StateCommitment};
pub use hashed_cursor::{
    DatabaseHashedAccountCursor, DatabaseHashedCursorFactory, DatabaseHashedStorageCursor,
};
pub use prefix_set::PrefixSetLoader;
pub use proof::{DatabaseProof, DatabaseStorageProof};
pub use state::{DatabaseHashedPostState, DatabaseStateRoot};
pub use storage::{DatabaseHashedStorage, DatabaseStorageRoot};
pub use trie_cursor::{
    DatabaseAccountTrieCursor, DatabaseStorageTrieCursor, DatabaseTrieCursorFactory,
};
pub use witness::DatabaseTrieWitness;
