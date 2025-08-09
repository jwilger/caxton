//! Caxton - A secure WebAssembly runtime for multi-agent systems
//!
//! This is the main entry point for the Caxton server application.

use anyhow::Result;
use caxton::{WasmRuntime, WasmRuntimeConfig};
use tracing::{error, info};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env().add_directive("caxton=info".parse()?),
        )
        .init();

    info!("Starting Caxton WebAssembly Runtime Server");

    // Load configuration
    let config = WasmRuntimeConfig::default();
    info!(
        "Configuration loaded: max_agents={}, debug={}",
        config.max_agents.into_inner(),
        config.enable_debug
    );

    // Initialize the runtime
    let runtime = WasmRuntime::new(config)?;
    info!("Runtime initialized successfully");

    if runtime.is_initialized() {
        info!("Caxton server is ready to accept agent deployments");

        // TODO: Start gRPC server (Story 006)
        // TODO: Start REST gateway (Story 007)
        // TODO: Initialize CLI interface (Story 008)

        // For now, just keep the server running
        info!("Server running. Press Ctrl+C to stop.");
        tokio::signal::ctrl_c().await?;
        info!("Shutdown signal received");
    } else {
        error!("Failed to initialize runtime");
        return Err(anyhow::anyhow!("Runtime initialization failed"));
    }

    info!("Caxton server shutting down gracefully");
    Ok(())
}
