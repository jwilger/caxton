//! Caxton CLI Binary
//!
//! Command-line interface for interacting with the Caxton application server

use axum::{Router, response::Html, routing::get};
use clap::{Parser, Subcommand};
use std::net::SocketAddr;

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
            // Start HTTP server on port 8080
            let app = Router::new().route("/", get(|| async { Html("Caxton Server") }));

            let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
            let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

            axum::serve(listener, app).await.unwrap();
        }
        None => {
            // No command provided, just exit (preserves existing behavior)
        }
    }
}
