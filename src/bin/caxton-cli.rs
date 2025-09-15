//! Caxton CLI Binary
//!
//! Command-line interface for interacting with the Caxton application server

use clap::{Parser, Subcommand};

/// Caxton CLI - Command-line interface for the Caxton application server
#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the Caxton server
    Serve,
}

fn main() {
    Args::parse();
}
