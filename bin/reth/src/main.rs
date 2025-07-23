#![allow(missing_docs)]

// LESSON 1: The Global Allocator
// In high-performance systems like Reth, memory allocation matters!
// Rust lets us choose our memory allocator. By default, Rust uses the system allocator,
// but Reth uses a custom one (usually jemalloc) for better performance.
// Think of an allocator as a "memory manager" - when you need memory for a new object,
// the allocator finds space for it. Some allocators are faster than others.
#[global_allocator]
static ALLOC: reth_cli_util::allocator::Allocator = reth_cli_util::allocator::new_allocator();

use clap::Parser;
use reth::{args::RessArgs, cli::Cli, ress::install_ress_subprotocol};
use reth_ethereum_cli::chainspec::EthereumChainSpecParser;
use reth_node_builder::NodeHandle;
use reth_node_ethereum::EthereumNode;
use tracing::info;

// LESSON 1: The Main Function - Where Everything Begins
// This is the entry point of Reth. Notice how clean it is!
// In Rust, main() is where your program starts, just like in C.
// But unlike C, Rust's main can be much more sophisticated.
fn main() {
    // LESSON 1: Crash Handler Installation
    // This installs a handler for segmentation faults (crashes).
    // If Reth crashes, this handler will print useful debugging information.
    // It's like having a black box recorder in an airplane!
    reth_cli_util::sigsegv_handler::install();

    // LESSON 1: Enabling Backtraces for Better Debugging
    // A backtrace shows the chain of function calls that led to an error.
    // It's like following breadcrumbs back through the code.
    // The `unsafe` block is needed because modifying environment variables
    // at runtime can cause race conditions in multi-threaded programs.
    if std::env::var_os("RUST_BACKTRACE").is_none() {
        unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    }

    // LESSON 1: The Power of Rust's Error Handling
    // This `if let Err(err)` pattern is Rust's way of handling errors elegantly.
    // Instead of try-catch blocks, Rust uses Result<T, E> types.
    // If something goes wrong, we get an Err(error), otherwise Ok(value).
    if let Err(err) =
        // LESSON 1: Command Line Interface (CLI) Parsing
        // Cli::parse() reads command-line arguments (like --help or --datadir).
        // The type parameters <EthereumChainSpecParser, RessArgs> tell Rust:
        // - How to parse Ethereum-specific chain configurations
        // - What additional arguments to accept (Ress is a Reth feature)
        Cli::<EthereumChainSpecParser, RessArgs>::parse().run(async move |builder, ress_args| {
            // LESSON 1: Structured Logging with Tracing
            // `info!` is like println! but much more powerful.
            // It includes timestamps, can be filtered by level, and sent to various outputs.
            // The target: "reth::cli" helps categorize where this log came from.
            info!(target: "reth::cli", "Launching node");
            
            // LESSON 1: The Node Builder Pattern
            // This creates our Ethereum node. Notice the method chaining:
            // builder.node().launch_with_debug_capabilities()
            // This is the "builder pattern" - we configure step by step, then build.
            // The `await?` means: wait for this async operation, and if it fails, return the error.
            let NodeHandle { node, node_exit_future } =
                builder.node(EthereumNode::default()).launch_with_debug_capabilities().await?;

            // LESSON 1: Optional Features
            // Not all features are always enabled. This checks if the user wants
            // the "ress" subprotocol (a Reth-specific feature).
            if ress_args.enabled {
                install_ress_subprotocol(
                    ress_args,
                    node.provider,      // Database access
                    node.evm_config,    // EVM configuration
                    node.network,       // P2P network access
                    node.task_executor, // For spawning async tasks
                    node.add_ons_handle.engine_events.new_listener(),
                )?;
            }

            // LESSON 1: Async/Await - Rust's Concurrency Model
            // This waits for the node to shut down. The node runs until:
            // - User presses Ctrl+C
            // - A fatal error occurs
            // - The node is shut down programmatically
            node_exit_future.await
        })
    {
        // LESSON 1: Error Reporting
        // If anything went wrong, print the error and exit with code 1.
        // Exit code 0 means success, anything else means failure.
        eprintln!("Error: {err:?}");
        std::process::exit(1);
    }
}
