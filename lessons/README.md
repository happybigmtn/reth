# Reth Learning Curriculum - Learn Ethereum Through Code

*As taught by Richard Feynman*

## Overview

This curriculum teaches you the entire Reth codebase through 100 progressive lessons. Each lesson includes:
- A markdown file explaining concepts in Feynman's teaching style
- Inline comments in the actual source code
- Hands-on exercises to solidify understanding

## Prerequisites

- Intermediate Rust knowledge (you should understand ownership, traits, and async/await)
- Basic understanding of blockchains (what is a transaction, what is a block)
- Curiosity and willingness to dig deep

## How to Use This Curriculum

1. Read each lesson's markdown file completely
2. Open the referenced source files and read the inline comments
3. Complete the exercises at the end of each lesson
4. Experiment with the code - break things and fix them!

## Curriculum Outline

### Foundation (Lessons 1-20): Understanding Ethereum and Rust Basics in Reth

- [x] **Lesson 1**: Introduction to Reth and Ethereum Architecture (`bin/reth/src/main.rs`)
- [ ] **Lesson 2**: The Type System - Primitives and Core Types (`crates/primitives/src/lib.rs`)
- [ ] **Lesson 3**: Understanding Transactions (`crates/primitives/src/transaction/mod.rs`)
- [ ] **Lesson 4**: Blocks and Headers (`crates/primitives/src/block.rs`)
- [ ] **Lesson 5**: The Account Model (`crates/primitives/src/account.rs`)
- [ ] **Lesson 6**: Introduction to Storage - MDBX Database (`crates/storage/db/src/implementation/mdbx/mod.rs`)
- [ ] **Lesson 7**: Database Tables and Schema (`crates/storage/db/src/tables/mod.rs`)
- [ ] **Lesson 8**: Encoding and Decoding - Compact Format (`crates/storage/codecs/src/lib.rs`)
- [ ] **Lesson 9**: The Provider Abstraction (`crates/storage/provider/src/traits/mod.rs`)
- [ ] **Lesson 10**: Understanding RLP Encoding (`crates/primitives/src/proofs.rs`)
- [ ] **Lesson 11**: Introduction to Networking - P2P Basics (`crates/net/network/src/manager.rs`)
- [ ] **Lesson 12**: The Discovery Protocol (`crates/net/discv4/src/lib.rs`)
- [ ] **Lesson 13**: RLPx and the Wire Protocol (`crates/net/eth-wire/src/protocol.rs`)
- [ ] **Lesson 14**: Transaction Pool Design (`crates/transaction-pool/src/pool/mod.rs`)
- [ ] **Lesson 15**: Understanding the EVM - Basic Concepts (`crates/evm/src/lib.rs`)
- [ ] **Lesson 16**: Integrating with revm (`crates/revm/src/database.rs`)
- [ ] **Lesson 17**: State and State Transitions (`crates/evm/execution-types/src/bundle_state.rs`)
- [ ] **Lesson 18**: The Trie - Merkle Patricia Trees (`crates/trie/src/lib.rs`)
- [ ] **Lesson 19**: RPC Server Architecture (`crates/rpc/rpc/src/eth/api/mod.rs`)
- [ ] **Lesson 20**: Error Handling in Reth (`crates/primitives/src/result.rs`)

### Execution and State Management (Lessons 21-40)

- [ ] **Lesson 21**: Block Execution Flow (`crates/ethereum/evm/src/execute.rs`)
- [ ] **Lesson 22**: Gas Mechanics and Metering (`crates/evm/src/metrics.rs`)
- [ ] **Lesson 23**: Understanding Receipts (`crates/primitives/src/receipt.rs`)
- [ ] **Lesson 24**: State Root Calculation (`crates/trie/src/hashed_state.rs`)
- [ ] **Lesson 25**: The Execution Outcome (`crates/evm/execution-types/src/execution_outcome.rs`)
- [ ] **Lesson 26**: Handling Reverts (`crates/revm/src/state_change.rs`)
- [ ] **Lesson 27**: The Block Executor (`crates/ethereum/evm/src/executor.rs`)
- [ ] **Lesson 28**: Understanding Hardforks (`crates/ethereum-forks/src/hardfork.rs`)
- [ ] **Lesson 29**: Storage Proofs and Witnesses (`crates/trie/src/witness.rs`)
- [ ] **Lesson 30**: Pruning and State Management (`crates/storage/provider/src/pruner.rs`)
- [ ] **Lesson 31**: Static Files and Cold Storage (`crates/storage/nippy-jar/src/lib.rs`)
- [ ] **Lesson 32**: Transaction Validation (`crates/transaction-pool/src/validate.rs`)
- [ ] **Lesson 33**: Understanding Opcodes (`crates/primitives/src/evm.rs`)
- [ ] **Lesson 34**: Contract Creation (`crates/evm/src/system_calls.rs`)
- [ ] **Lesson 35**: Storage Layout (`crates/storage/provider/src/bundle_state_provider.rs`)
- [ ] **Lesson 36**: Logs and Events (`crates/primitives/src/log.rs`)
- [ ] **Lesson 37**: The Blockchain Tree (`crates/blockchain-tree/src/tree.rs`)
- [ ] **Lesson 38**: Engine API (`crates/engine/tree/src/engine.rs`)
- [ ] **Lesson 39**: Payload Building (`crates/payload/basic/src/lib.rs`)
- [ ] **Lesson 40**: MEV and Transaction Ordering (`crates/transaction-pool/src/ordering.rs`)

### Advanced Topics and Optimizations (Lessons 41-60)

Topics include parallel execution, staged sync, consensus validation, EIPs implementation, and performance optimizations.

### Production Systems and Advanced Features (Lessons 61-80)

Topics include database maintenance, network resilience, RPC features, and production optimizations.

### Integration and Advanced Patterns (Lessons 81-100)

Topics include building custom chains, testing strategies, security considerations, and putting it all together.

## Teaching Philosophy

Following Richard Feynman's approach:

1. **Start Simple**: Every concept begins with the simplest possible explanation
2. **Build Gradually**: Complexity is added layer by layer
3. **Use Analogies**: Abstract concepts are related to everyday experiences
4. **Question Everything**: Each lesson ends with thought-provoking questions
5. **Learn by Doing**: Hands-on exercises reinforce understanding

## Getting Started

1. Clone the Reth repository
2. Open Lesson 1 (`lessons/1.md`)
3. Have the Reth source code open in your editor
4. Follow along and experiment!

Remember Feynman's words: *"I learned very early the difference between knowing the name of something and knowing something."*

Let's not just learn the names of things in Ethereum - let's truly understand how they work!