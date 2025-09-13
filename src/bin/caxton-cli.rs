//! Caxton CLI Binary
//!
//! Command-line interface for managing the Caxton agent runtime server.

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "caxton-cli")]
#[command(about = "Command-line interface for Caxton agent runtime")]
#[command(version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage the Caxton server
    Server,
}

fn main() {
    let _cli = Cli::parse();
}
