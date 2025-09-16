//! Caxton CLI Binary
//!
//! Command-line interface for interacting with the Caxton application server

use clap::{Parser, Subcommand};

mod domain;
mod server;

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

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.command {
        Some(Commands::Serve) => {
            // Load configuration from caxton.toml file
            let config = domain::config::load_config(None).expect("Failed to load configuration");

            // Start HTTP server on configured port
            let (listener, _addr) = server::start_server(config)
                .await
                .expect("Failed to start server");
            let router = server::create_router();

            server::serve(listener, router)
                .await
                .expect("Server failed");
        }
        None => {
            // No command provided, just exit (preserves existing behavior)
        }
    }
}
