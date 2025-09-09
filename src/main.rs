//! Caxton - A secure WebAssembly runtime for multi-agent systems
//!
//! This is the main entry point for the Caxton server application.

use anyhow::Result;
use caxton::{WasmRuntime, WasmRuntimeConfig, create_app};
use tokio::net::TcpListener;
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

        // Start REST API server
        let app = create_app();
        let listener = TcpListener::bind("localhost:8080").await?;
        info!("REST API server listening on http://localhost:8080");

        // TODO: Initialize CLI interface (Story 008)

        // Run the HTTP server
        axum::serve(listener, app).await?;
    } else {
        error!("Failed to initialize runtime");
        return Err(anyhow::anyhow!("Runtime initialization failed"));
    }

    info!("Caxton server shutting down gracefully");
    Ok(())
}
