# Reth Development Guide for AI Agents

This guide provides comprehensive instructions for AI agents working on the Reth codebase. It covers the architecture, development workflows, and critical guidelines for effective contributions.

## Project Overview

Reth is a high-performance Ethereum execution client written in Rust, focusing on modularity, performance, and contributor-friendliness. The codebase is organized into well-defined crates with clear boundaries and responsibilities.

## Architecture Overview

### Core Components

1. **Consensus (`crates/consensus/`)**: Validates blocks according to Ethereum consensus rules
2. **Storage (`crates/storage/`)**: Hybrid database using MDBX + static files for optimal performance
3. **Networking (`crates/net/`)**: P2P networking stack with discovery, sync, and transaction propagation
4. **RPC (`crates/rpc/`)**: JSON-RPC server supporting all standard Ethereum APIs
5. **Execution (`crates/evm/`, `crates/ethereum/`)**: Transaction execution and state transitions
6. **Pipeline (`crates/stages/`)**: Staged sync architecture for blockchain synchronization
7. **Trie (`crates/trie/`)**: Merkle Patricia Trie implementation with parallel state root computation
8. **Node Builder (`crates/node/`)**: High-level node orchestration and configuration
9  **The Consensus Engine (`crates/engine/`)**: Handles processing blocks received from the consensus layer with the Engine API (newPayload, forkchoiceUpdated)

### Key Design Principles

- **Modularity**: Each crate can be used as a standalone library
- **Performance**: Extensive use of parallelism, memory-mapped I/O, and optimized data structures
- **Extensibility**: Traits and generic types allow for different implementations (Ethereum, Optimism, etc.)
- **Type Safety**: Strong typing throughout with minimal use of dynamic dispatch

## Development Workflow

### Code Style and Standards

1. **Formatting**: Always use nightly rustfmt
   ```bash
   cargo +nightly fmt --all
   ```

2. **Linting**: Run clippy with all features
   ```bash
   RUSTFLAGS="-D warnings" cargo +nightly clippy --workspace --lib --examples --tests --benches --all-features --locked
   ```

3. **Testing**: Use nextest for faster test execution
   ```bash
   cargo nextest run --workspace
   ```

### Common Contribution Types

Based on actual recent PRs, here are typical contribution patterns:

#### 1. Small Bug Fixes (1-10 lines)
Real example: Fixing beacon block root handling ([#16767](https://github.com/paradigmxyz/reth/pull/16767))
```rust
// Changed a single line to fix logic error
- parent_beacon_block_root: parent.parent_beacon_block_root(),
+ parent_beacon_block_root: parent.parent_beacon_block_root().map(|_| B256::ZERO),
```

#### 2. Integration with Upstream Changes
Real example: Integrating revm updates ([#16752](https://github.com/paradigmxyz/reth/pull/16752))
```rust
// Update code to use new APIs from dependencies
- if self.fork_tracker.is_shanghai_activated() {
-     if let Err(err) = transaction.ensure_max_init_code_size(MAX_INIT_CODE_BYTE_SIZE) {
+ if let Some(init_code_size_limit) = self.fork_tracker.max_initcode_size() {
+     if let Err(err) = transaction.ensure_max_init_code_size(init_code_size_limit) {
```

#### 3. Adding Comprehensive Tests
Real example: ETH69 protocol tests ([#16759](https://github.com/paradigmxyz/reth/pull/16759))
```rust
#[tokio::test(flavor = "multi_thread")]
async fn test_eth69_peers_can_connect() {
    // Create test network with specific protocol versions
    let p0 = PeerConfig::with_protocols(NoopProvider::default(), Some(EthVersion::Eth69.into()));
    // Test connection and version negotiation
}
```

#### 4. Making Components Generic
Real example: Making EthEvmConfig generic over chainspec ([#16758](https://github.com/paradigmxyz/reth/pull/16758))
```rust
// Before: Hardcoded to ChainSpec
- pub struct EthEvmConfig<EvmFactory = EthEvmFactory> {
-     pub executor_factory: EthBlockExecutorFactory<RethReceiptBuilder, Arc<ChainSpec>, EvmFactory>,

// After: Generic over any chain spec type
+ pub struct EthEvmConfig<C = ChainSpec, EvmFactory = EthEvmFactory>
+ where
+     C: EthereumHardforks,
+ {
+     pub executor_factory: EthBlockExecutorFactory<RethReceiptBuilder, Arc<C>, EvmFactory>,
```

#### 5. Resource Management Improvements
Real example: ETL directory cleanup ([#16770](https://github.com/paradigmxyz/reth/pull/16770))
```rust
// Add cleanup logic on startup
+ if let Err(err) = fs::remove_dir_all(&etl_path) {
+     warn!(target: "reth::cli", ?etl_path, %err, "Failed to remove ETL path on launch");
+ }
```

#### 6. Feature Additions
Real example: Sharded mempool support ([#16756](https://github.com/paradigmxyz/reth/pull/16756))
```rust
// Add new filtering policies for transaction announcements
pub struct ShardedMempoolAnnouncementFilter<T> {
    pub inner: T,
    pub shard_bits: u8,
    pub node_id: Option<B256>,
}
```

### Testing Guidelines

1. **Unit Tests**: Test individual functions and components
2. **Integration Tests**: Test interactions between components
3. **Benchmarks**: For performance-critical code
4. **Fuzz Tests**: For parsing and serialization code
5. **Property Tests**: For checking component correctness on a wide variety of inputs

Example test structure:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_component_behavior() {
        // Arrange
        let component = Component::new();
        
        // Act
        let result = component.operation();
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

### Performance Considerations

1. **Avoid Allocations in Hot Paths**: Use references and borrowing
2. **Parallel Processing**: Use rayon for CPU-bound parallel work
3. **Async/Await**: Use tokio for I/O-bound operations
4. **File Operations**: Use `reth_fs_util` instead of `std::fs` for better error handling

### Common Pitfalls

1. **Don't Block Async Tasks**: Use `spawn_blocking` for CPU-intensive work or work with lots of blocking I/O
2. **Handle Errors Properly**: Use `?` operator and proper error types

### What to Avoid

Based on PR patterns, avoid:

1. **Large, sweeping changes**: Keep PRs focused and reviewable
2. **Mixing unrelated changes**: One logical change per PR
3. **Ignoring CI failures**: All checks must pass
4. **Incomplete implementations**: Finish features before submitting
5. **Modifying libmdbx sources**: Never modify files in `crates/storage/libmdbx-rs/mdbx-sys/libmdbx/` - this is vendored third-party code

### CI Requirements

Before submitting changes, ensure:

1. **Format Check**: `cargo +nightly fmt --all --check`
2. **Clippy**: No warnings with `RUSTFLAGS="-D warnings"`
3. **Tests Pass**: All unit and integration tests
4. **Documentation**: Update relevant docs and add doc comments with `cargo docs --document-private-items`
5. **Commit Messages**: Follow conventional format (feat:, fix:, chore:, etc.)


### Opening PRs against <https://github.com/paradigmxyz/reth>

Label PRs appropriately, first check the available labels and then apply the relevant ones:
* when changes are RPC related, add A-rpc label
* when changes are docs related, add C-docs label
* when changes are optimism related (e.g. new feature or exclusive changes to crates/optimism), add A-op-reth label
* ... and so on, check the available labels for more options.
* if being tasked to open a pr, ensure that all changes are properly formatted: `cargo +nightly fmt --all`

If changes in reth include changes to dependencies, run commands `zepter` and `make lint-toml` before finalizing the pr. Assume `zepter` binary is installed.

### Debugging Tips

1. **Logging**: Use `tracing` crate with appropriate levels
   ```rust
   tracing::debug!(target: "reth::component", ?value, "description");
   ```

2. **Metrics**: Add metrics for monitoring
   ```rust
   metrics::counter!("reth_component_operations").increment(1);
   ```

3. **Test Isolation**: Use separate test databases/directories

### Finding Where to Contribute

1. **Check Issues**: Look for issues labeled `good-first-issue` or `help-wanted`
2. **Review TODOs**: Search for `TODO` comments in the codebase
3. **Improve Tests**: Areas with low test coverage are good targets
4. **Documentation**: Improve code comments and documentation
5. **Performance**: Profile and optimize hot paths (with benchmarks)

### Common PR Patterns

#### Small, Focused Changes
Most PRs change only 1-5 files. Examples:
- Single-line bug fixes
- Adding a missing trait implementation
- Updating error messages
- Adding test cases for edge conditions

#### Integration Work
When dependencies update (especially revm), code needs updating:
- Check for breaking API changes
- Update to use new features (like EIP implementations)
- Ensure compatibility with new versions

#### Test Improvements
Tests often need expansion for:
- New protocol versions (ETH68, ETH69)
- Edge cases in state transitions
- Network behavior under specific conditions
- Concurrent operations

#### Making Code More Generic
Common refactoring pattern:
- Replace concrete types with generics
- Add trait bounds for flexibility
- Enable reuse across different chain types (Ethereum, Optimism)

### Example Contribution Workflow

Let's say you want to fix a bug where external IP resolution fails on startup:

1. **Create a branch**:
   ```bash
   git checkout -b fix-external-ip-resolution
   ```

2. **Find the relevant code**:
   ```bash
   # Search for IP resolution code
   rg "external.*ip" --type rust
   ```

3. **Reason about the problem, when the problem is identified, make the fix**:
   ```rust
   // In crates/net/discv4/src/lib.rs
   pub fn resolve_external_ip() -> Option<IpAddr> {
       // Add fallback mechanism
       nat::external_ip()
           .or_else(|| nat::external_ip_from_stun())
           .or_else(|| Some(DEFAULT_IP))
   }
   ```

4. **Add a test**:
   ```rust
   #[test]
   fn test_external_ip_fallback() {
       // Test that resolution has proper fallbacks
   }
   ```

5. **Run checks**:
   ```bash
   cargo +nightly fmt --all
   cargo clippy --all-features
   cargo test -p reth-discv4
   ```

6. **Commit with clear message**:
   ```bash
   git commit -m "fix: add fallback for external IP resolution

   Previously, node startup could fail if external IP resolution
   failed. This adds fallback mechanisms to ensure the node can
   always start with a reasonable default."
   ```

## Quick Reference

### Essential Commands

```bash
# Format code
cargo +nightly fmt --all

# Run lints
RUSTFLAGS="-D warnings" cargo +nightly clippy --workspace --all-features --locked

# Run tests
cargo nextest run --workspace

# Run specific benchmark
cargo bench --bench bench_name

# Build optimized binary
cargo build --release --features "jemalloc asm-keccak"

# Check compilation for all features
cargo check --workspace --all-features

# Check documentation
cargo docs --document-private-items 
```

# Reth Learning Curriculum - 100 Lessons by Richard Feynman

## Overview
This curriculum teaches the Reth codebase through 100 progressive lessons, explaining both Rust syntax (intermediate level) and Ethereum/EVM concepts (from beginner level). Each lesson includes a markdown file with explanations and inline comments in relevant source files.

## Curriculum Structure

### Foundation (Lessons 1-20): Understanding Ethereum and Rust Basics in Reth

**Lesson 1**: Introduction to Reth and Ethereum Architecture
- File: `bin/reth/src/main.rs`
- Topics: What is an execution client, Ethereum's architecture, Reth's place in the ecosystem
- Rust: Basic project structure, workspace organization

**Lesson 2**: The Type System - Primitives and Core Types
- File: `crates/primitives/src/lib.rs`
- Topics: Addresses, hashes (H256/B256), block numbers
- Rust: Type aliases, newtype pattern, From/Into traits

**Lesson 3**: Understanding Transactions
- File: `crates/primitives/src/transaction/mod.rs`
- Topics: Transaction types (Legacy, EIP-1559, EIP-4844), gas concepts
- Rust: Enums with data, pattern matching, serde

**Lesson 4**: Blocks and Headers
- File: `crates/primitives/src/block.rs`
- Topics: Block structure, header fields, body composition
- Rust: Struct composition, builder pattern

**Lesson 5**: The Account Model
- File: `crates/primitives/src/account.rs`
- Topics: Account state (nonce, balance, code, storage), EOAs vs contracts
- Rust: Option types, zero-cost abstractions

**Lesson 6**: Introduction to Storage - MDBX Database
- File: `crates/storage/db/src/implementation/mdbx/mod.rs`
- Topics: Key-value storage, ACID properties, memory-mapped files
- Rust: Unsafe code, FFI bindings, error handling

**Lesson 7**: Database Tables and Schema
- File: `crates/storage/db/src/tables/mod.rs`
- Topics: Table design, primary keys, data organization
- Rust: Const generics, macro usage for table definitions

**Lesson 8**: Encoding and Decoding - Compact Format
- File: `crates/storage/codecs/src/lib.rs`
- Topics: Space-efficient encoding, RLP alternatives
- Rust: Custom derive macros, bitpacking

**Lesson 9**: The Provider Abstraction
- File: `crates/storage/provider/src/traits/mod.rs`
- Topics: Data access patterns, read vs write operations
- Rust: Trait design, associated types, trait objects

**Lesson 10**: Understanding RLP Encoding
- File: `crates/primitives/src/proofs.rs`
- Topics: Recursive Length Prefix encoding, Ethereum's serialization
- Rust: Recursion, generic programming

**Lesson 11**: Introduction to Networking - P2P Basics
- File: `crates/net/network/src/manager.rs`
- Topics: Peer-to-peer networks, TCP/UDP, node discovery
- Rust: Async/await, tokio runtime

**Lesson 12**: The Discovery Protocol (discv4)
- File: `crates/net/discv4/src/lib.rs`
- Topics: Kademlia DHT, node distance, routing tables
- Rust: UDP sockets, concurrent data structures

**Lesson 13**: RLPx and the Wire Protocol
- File: `crates/net/eth-wire/src/protocol.rs`
- Topics: Message types, protocol versions, handshakes
- Rust: State machines, protocol buffers

**Lesson 14**: Transaction Pool Design
- File: `crates/transaction-pool/src/pool/mod.rs`
- Topics: Mempool concepts, transaction ordering, gas pricing
- Rust: BTreeMap, concurrent collections

**Lesson 15**: Understanding the EVM - Basic Concepts
- File: `crates/evm/src/lib.rs`
- Topics: Stack machine, opcodes, gas, memory/storage
- Rust: Trait boundaries, phantom data

**Lesson 16**: Integrating with revm
- File: `crates/revm/src/database.rs`
- Topics: EVM implementation, state access during execution
- Rust: External crate integration, adapter pattern

**Lesson 17**: State and State Transitions
- File: `crates/evm/execution-types/src/bundle_state.rs`
- Topics: State changes, reverts, account touches
- Rust: Copy-on-write, efficient diffs

**Lesson 18**: The Trie - Merkle Patricia Trees
- File: `crates/trie/src/lib.rs`
- Topics: Merkle proofs, state root calculation, Patricia trie structure
- Rust: Tree structures, recursive algorithms

**Lesson 19**: RPC Server Architecture
- File: `crates/rpc/rpc/src/eth/api/mod.rs`
- Topics: JSON-RPC, API namespaces, request handling
- Rust: Async traits, tower middleware

**Lesson 20**: Error Handling in Reth
- File: `crates/primitives/src/result.rs`
- Topics: Error types, propagation, recovery strategies
- Rust: Result type, error conversion, thiserror

### Execution and State Management (Lessons 21-40)

**Lesson 21**: Block Execution Flow
- File: `crates/ethereum/evm/src/execute.rs`
- Topics: Transaction execution order, state updates, receipts
- Rust: Iterator patterns, fold/collect

**Lesson 22**: Gas Mechanics and Metering
- File: `crates/evm/src/metrics.rs`
- Topics: Gas costs, refunds, EIP-1559 base fee
- Rust: Metrics collection, atomic operations

**Lesson 23**: Understanding Receipts
- File: `crates/primitives/src/receipt.rs`
- Topics: Receipt structure, logs, bloom filters
- Rust: Bit manipulation, bloom filter implementation

**Lesson 24**: State Root Calculation
- File: `crates/trie/src/hashed_state.rs`
- Topics: Incremental trie updates, hash computation
- Rust: Parallel iterators, rayon

**Lesson 25**: The Execution Outcome
- File: `crates/evm/execution-types/src/execution_outcome.rs`
- Topics: Bundling execution results, organizing state changes
- Rust: Type-level programming, zero-copy design

**Lesson 26**: Handling Reverts
- File: `crates/revm/src/state_change.rs`
- Topics: Transaction failure, state rollback, revert reasons
- Rust: RAII pattern, drop trait

**Lesson 27**: The Block Executor
- File: `crates/ethereum/evm/src/executor.rs`
- Topics: Block-level execution, system transactions
- Rust: Strategy pattern, dependency injection

**Lesson 28**: Understanding Hardforks
- File: `crates/ethereum-forks/src/hardfork.rs`
- Topics: Fork scheduling, EIP activation, chain configuration
- Rust: Compile-time configuration, feature flags

**Lesson 29**: Storage Proofs and Witnesses
- File: `crates/trie/src/witness.rs`
- Topics: Merkle proofs, light client support
- Rust: Serialization, proof verification

**Lesson 30**: Pruning and State Management
- File: `crates/storage/provider/src/pruner.rs`
- Topics: State pruning, full vs archive nodes
- Rust: Background tasks, channels

**Lesson 31**: Static Files and Cold Storage
- File: `crates/storage/nippy-jar/src/lib.rs`
- Topics: Immutable data storage, compression strategies
- Rust: Custom file formats, mmap

**Lesson 32**: Transaction Validation
- File: `crates/transaction-pool/src/validate.rs`
- Topics: Signature verification, nonce checking, balance validation
- Rust: Cryptography, secp256k1

**Lesson 33**: Understanding Opcodes
- File: `crates/primitives/src/evm.rs`
- Topics: EVM instruction set, stack operations, control flow
- Rust: Instruction dispatch, jump tables

**Lesson 34**: Contract Creation
- File: `crates/evm/src/system_calls.rs`
- Topics: CREATE/CREATE2, init code, address derivation
- Rust: Hash functions, deterministic addressing

**Lesson 35**: Storage Layout and SSTORE/SLOAD
- File: `crates/storage/provider/src/bundle_state_provider.rs`
- Topics: Storage slots, cold/warm access, gas costs
- Rust: HashMap optimizations, caching

**Lesson 36**: Logs and Events
- File: `crates/primitives/src/log.rs`
- Topics: Event emission, topics, data field
- Rust: Variadic generics, const arrays

**Lesson 37**: The Blockchain Tree
- File: `crates/blockchain-tree/src/tree.rs`
- Topics: Fork management, canonical chain, reorgs
- Rust: Tree manipulation, backtracking

**Lesson 38**: Engine API and Consensus Layer
- File: `crates/engine/tree/src/engine.rs`
- Topics: newPayload, forkchoiceUpdated, payload building
- Rust: RPC protocols, async streams

**Lesson 39**: Payload Building
- File: `crates/payload/basic/src/lib.rs`
- Topics: Transaction selection, block assembly, MEV
- Rust: Priority queues, optimization strategies

**Lesson 40**: Understanding MEV and Transaction Ordering
- File: `crates/transaction-pool/src/ordering.rs`
- Topics: Maximal Extractable Value, priority ordering
- Rust: Custom comparators, heap structures

### Advanced Topics and Optimizations (Lessons 41-60)

**Lesson 41**: Parallel EVM Execution
- File: `crates/ethereum/evm/src/parallel.rs`
- Topics: Dependency analysis, parallel transaction execution
- Rust: Thread pools, work stealing

**Lesson 42**: Optimistic Concurrency Control
- File: `crates/storage/provider/src/concurrent.rs`
- Topics: MVCC, conflict detection, retry logic
- Rust: Lock-free programming, atomics

**Lesson 43**: The Staged Sync Pipeline
- File: `crates/stages/src/pipeline.rs`
- Topics: Staged synchronization, checkpoints, unwinding
- Rust: State machines, progress tracking

**Lesson 44**: Headers Stage
- File: `crates/stages/src/stages/headers.rs`
- Topics: Header download, validation, chain selection
- Rust: Buffering strategies, backpressure

**Lesson 45**: Bodies Stage
- File: `crates/stages/src/stages/bodies.rs`
- Topics: Block body download, transaction/uncle validation
- Rust: Parallel downloads, connection pooling

**Lesson 46**: Execution Stage
- File: `crates/stages/src/stages/execution.rs`
- Topics: Batch execution, state updates, progress tracking
- Rust: Chunking, memory management

**Lesson 47**: Merkle Stage
- File: `crates/stages/src/stages/merkle.rs`
- Topics: Incremental trie construction, intermediate hashes
- Rust: Tree algorithms, memoization

**Lesson 48**: Transaction Lookup Stage
- File: `crates/stages/src/stages/tx_lookup.rs`
- Topics: Building transaction indices, hash to number mapping
- Rust: Inverted indices, database transactions

**Lesson 49**: Account History Indexing
- File: `crates/stages/src/stages/history.rs`
- Topics: Historical state access, change sets
- Rust: Time-series data, efficient storage

**Lesson 50**: Consensus Validation
- File: `crates/consensus/consensus/src/validation.rs`
- Topics: Block validation rules, consensus errors
- Rust: Validation pipelines, error accumulation

**Lesson 51**: Understanding EIP-1559
- File: `crates/primitives/src/basefee.rs`
- Topics: Dynamic base fee, priority fees, fee burning
- Rust: Fixed-point arithmetic, overflow handling

**Lesson 52**: EIP-4844 and Blob Transactions
- File: `crates/primitives/src/transaction/eip4844.rs`
- Topics: Blob transactions, KZG commitments, proto-danksharding
- Rust: Cryptographic primitives, large data handling

**Lesson 53**: The Withdrawals System
- File: `crates/ethereum/evm/src/withdrawals.rs`
- Topics: Validator withdrawals, system-level balance updates
- Rust: System calls, privileged operations

**Lesson 54**: Cross-Chain Communication
- File: `crates/optimism/src/bridge.rs`
- Topics: L1-L2 communication, deposits, withdrawals
- Rust: Message passing, serialization

**Lesson 55**: Node Configuration
- File: `crates/node/builder/src/config.rs`
- Topics: Node types, network selection, feature toggles
- Rust: Configuration management, serde

**Lesson 56**: Metrics and Monitoring
- File: `crates/node/metrics/src/lib.rs`
- Topics: Prometheus metrics, performance monitoring
- Rust: Metrics libraries, lazy statics

**Lesson 57**: Database Migrations
- File: `crates/storage/db/src/version.rs`
- Topics: Schema versioning, data migration strategies
- Rust: Version checking, backward compatibility

**Lesson 58**: Network Protocol Upgrades
- File: `crates/net/eth-wire/src/version.rs`
- Topics: Protocol versioning, capability negotiation
- Rust: Protocol compatibility, feature detection

**Lesson 59**: Testing Infrastructure
- File: `crates/node/builder/src/test_utils.rs`
- Topics: Test fixtures, mock providers, deterministic tests
- Rust: Test organization, property testing

**Lesson 60**: Benchmarking and Performance
- File: `crates/storage/db/benches/dbread.rs`
- Topics: Performance measurement, optimization targets
- Rust: Criterion benchmarks, flame graphs

### Production Systems and Advanced Features (Lessons 61-80)

**Lesson 61**: Database Compaction
- File: `crates/storage/db/src/maintenance.rs`
- Topics: Background maintenance, space reclamation
- Rust: Background threads, scheduling

**Lesson 62**: Snapshot Sync
- File: `crates/stages/src/snapshot.rs`
- Topics: Fast sync via snapshots, state reconstruction
- Rust: Streaming protocols, progressive loading

**Lesson 63**: Network Resilience
- File: `crates/net/network/src/resilience.rs`
- Topics: Peer scoring, ban lists, DoS protection
- Rust: Rate limiting, circuit breakers

**Lesson 64**: RPC Rate Limiting
- File: `crates/rpc/rpc-server-types/src/rate_limit.rs`
- Topics: API quotas, fair usage, DoS prevention
- Rust: Token buckets, middleware

**Lesson 65**: Database Sharding
- File: `crates/storage/db/src/sharding.rs`
- Topics: Horizontal scaling, shard routing
- Rust: Consistent hashing, routing tables

**Lesson 66**: Execution Extensions (ExEx)
- File: `crates/exex/src/lib.rs`
- Topics: Pluggable execution, custom indexing
- Rust: Plugin systems, dynamic loading

**Lesson 67**: Advanced Trie Algorithms
- File: `crates/trie/src/parallel.rs`
- Topics: Parallel trie construction, work distribution
- Rust: Work stealing, load balancing

**Lesson 68**: Memory Pool Optimization
- File: `crates/transaction-pool/src/memory.rs`
- Topics: Memory bounds, eviction policies
- Rust: Memory profiling, arena allocation

**Lesson 69**: Chain Reorganization Handling
- File: `crates/blockchain-tree/src/reorg.rs`
- Topics: Reorg detection, state rollback, notification
- Rust: Event systems, observer pattern

**Lesson 70**: Light Client Support
- File: `crates/net/network/src/light.rs`
- Topics: Light client protocols, proof serving
- Rust: Selective synchronization, proof generation

**Lesson 71**: Archive Node Features
- File: `crates/node/core/src/archive.rs`
- Topics: Full history retention, query optimization
- Rust: Data indexing, query planning

**Lesson 72**: Debug and Trace APIs
- File: `crates/rpc/rpc/src/debug/mod.rs`
- Topics: Transaction debugging, state inspection
- Rust: Introspection, debug formatting

**Lesson 73**: Custom RPC Endpoints
- File: `crates/rpc/rpc-builder/src/custom.rs`
- Topics: Extending RPC, custom namespaces
- Rust: Dynamic dispatch, trait objects

**Lesson 74**: WebSocket Support
- File: `crates/rpc/ipc/src/websocket.rs`
- Topics: Bidirectional communication, subscriptions
- Rust: WebSocket protocol, async streams

**Lesson 75**: IPC Communication
- File: `crates/rpc/ipc/src/client.rs`
- Topics: Local inter-process communication
- Rust: Unix sockets, named pipes

**Lesson 76**: State Diff Tracking
- File: `crates/storage/provider/src/diff.rs`
- Topics: State change tracking, diff generation
- Rust: Diff algorithms, change detection

**Lesson 77**: Gas Price Oracle
- File: `crates/rpc/rpc/src/eth/api/fee_history.rs`
- Topics: Fee estimation, historical gas prices
- Rust: Statistical analysis, percentiles

**Lesson 78**: Uncle/Ommer Handling
- File: `crates/consensus/consensus/src/ommers.rs`
- Topics: Uncle blocks, reward calculation
- Rust: Set operations, validation

**Lesson 79**: Chain Specification
- File: `crates/chainspec/src/spec.rs`
- Topics: Network parameters, genesis configuration
- Rust: Static configuration, const evaluation

**Lesson 80**: Node Discovery Optimization
- File: `crates/net/discv5/src/lib.rs`
- Topics: Discovery v5, topic discovery, efficiency
- Rust: Probabilistic algorithms, caching

### Integration and Advanced Patterns (Lessons 81-100)

**Lesson 81**: Building Custom Chains
- File: `crates/optimism/node/src/lib.rs`
- Topics: Extending Reth, custom consensus rules
- Rust: Trait specialization, type families

**Lesson 82**: Multi-Chain Support
- File: `crates/node/types/src/lib.rs`
- Topics: Supporting multiple networks, chain abstraction
- Rust: Generic programming, associated constants

**Lesson 83**: Database Backup and Recovery
- File: `crates/storage/db/src/backup.rs`
- Topics: Consistent backups, point-in-time recovery
- Rust: File operations, consistency guarantees

**Lesson 84**: Performance Profiling
- File: `crates/node/metrics/src/profiling.rs`
- Topics: CPU/memory profiling, bottleneck identification
- Rust: Profiling tools, optimization workflow

**Lesson 85**: Fuzz Testing
- File: `crates/primitives/fuzz/src/lib.rs`
- Topics: Fuzzing strategies, coverage-guided fuzzing
- Rust: Arbitrary trait, fuzzer integration

**Lesson 86**: Property-Based Testing
- File: `crates/storage/db/src/test_utils/proptest.rs`
- Topics: Invariant testing, random test generation
- Rust: Proptest framework, shrinking

**Lesson 87**: Integration Testing
- File: `crates/node/builder/src/integration_tests.rs`
- Topics: End-to-end testing, test scenarios
- Rust: Test harnesses, fixtures

**Lesson 88**: Continuous Integration
- File: `.github/workflows/ci.yml`
- Topics: CI/CD pipelines, automated testing
- Rust: Cross-platform builds, caching

**Lesson 89**: Release Engineering
- File: `crates/node/builder/src/release.rs`
- Topics: Version management, release processes
- Rust: Cargo versioning, feature stability

**Lesson 90**: Documentation Generation
- File: `crates/primitives/src/lib.rs` (doc comments)
- Topics: API documentation, examples
- Rust: Doc comments, doctests

**Lesson 91**: Error Recovery Strategies
- File: `crates/consensus/consensus/src/recovery.rs`
- Topics: Fault tolerance, graceful degradation
- Rust: Panic handling, error boundaries

**Lesson 92**: Security Considerations
- File: `crates/net/network/src/security.rs`
- Topics: Attack vectors, mitigation strategies
- Rust: Safe abstractions, security patterns

**Lesson 93**: Resource Management
- File: `crates/node/core/src/resources.rs`
- Topics: Memory limits, file descriptors, cleanup
- Rust: RAII, resource pools

**Lesson 94**: Observability
- File: `crates/node/events/src/lib.rs`
- Topics: Event streaming, structured logging
- Rust: Tracing ecosystem, spans

**Lesson 95**: Database Optimization
- File: `crates/storage/db/src/optimization.rs`
- Topics: Query optimization, index design
- Rust: B-tree operations, cache efficiency

**Lesson 96**: Network Topology
- File: `crates/net/network/src/topology.rs`
- Topics: Peer selection, network graphs
- Rust: Graph algorithms, connectivity

**Lesson 97**: Cryptographic Primitives
- File: `crates/primitives/src/crypto.rs`
- Topics: Hashing, signatures, key derivation
- Rust: Crypto libraries, constant-time operations

**Lesson 98**: System Calls and Precompiles
- File: `crates/evm/src/precompiles.rs`
- Topics: Built-in contracts, system operations
- Rust: FFI, native extensions

**Lesson 99**: Future Compatibility
- File: `crates/ethereum-forks/src/future.rs`
- Topics: Upcoming EIPs, upgrade paths
- Rust: Feature planning, deprecation

**Lesson 100**: Putting It All Together
- File: `examples/custom_node.rs`
- Topics: Building a complete node, architectural review
- Rust: System design, component integration

## Teaching Philosophy

Each lesson follows the Feynman method:
1. Start with the simplest explanation
2. Build complexity gradually
3. Use concrete examples from the code
4. Explain the "why" behind design decisions
5. Connect concepts to real-world analogies

## Progress Tracking

Track your progress in refactor.md as you complete lessons. Each lesson should include:
- Main concepts learned
- Key code locations explored
- Rust patterns encountered
- Questions for further exploration
