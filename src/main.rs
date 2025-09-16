//! Caxton CLI Binary
//!
//! Command-line interface for interacting with the Caxton application server

use caxton::{domain, server};
use clap::{Parser, Subcommand};
use tokio::signal;

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

/// Set up signal handling for graceful shutdown
///
/// This function creates a future that completes when SIGTERM, SIGINT (Ctrl+C),
/// or other shutdown signals are received. When a signal is received, it cancels
/// the provided token to initiate graceful shutdown.
async fn setup_shutdown_signal(shutdown_token: tokio_util::sync::CancellationToken) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        () = ctrl_c => {
            println!("Received Ctrl+C, initiating graceful shutdown...");
        },
        () = terminate => {
            println!("Received SIGTERM, initiating graceful shutdown...");
        },
    }

    shutdown_token.cancel();
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

            // Set up graceful shutdown signal handling
            let shutdown_token = tokio_util::sync::CancellationToken::new();
            let shutdown_signal = setup_shutdown_signal(shutdown_token.clone());

            // Start server with graceful shutdown capability
            tokio::select! {
                result = server::serve_with_graceful_shutdown(listener, router, shutdown_token) => {
                    result.expect("Server failed");
                }
                () = shutdown_signal => {
                    // Signal received, graceful shutdown already initiated
                }
            }
        }
        None => {
            // No command provided, just exit (preserves existing behavior)
        }
    }
}
