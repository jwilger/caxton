//! Caxton CLI Binary
//!
//! Command-line interface for interacting with the Caxton application server

use clap::Parser;

/// Caxton CLI - Command-line interface for the Caxton application server
#[derive(Parser)]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Args {}

fn main() {
    Args::parse();
}
