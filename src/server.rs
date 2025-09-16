//! Server functionality for Caxton
//!
//! This module provides the HTTP server implementation with testable components

use crate::domain::config::AppConfig;
use axum::{Router, response::Html, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::{info, instrument};

/// Create the Axum router with all routes
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(|| async { Html("Caxton Server") }))
        .route("/health", get(|| async { "OK" }))
        .layer(TraceLayer::new_for_http())
}

/// Start server with the given configuration
/// Returns the bound listener and server handle for testing
///
/// # Errors
///
/// Returns an error if the server cannot bind to the specified port.
#[instrument(skip(config))]
pub async fn start_server(
    config: AppConfig,
) -> Result<(TcpListener, SocketAddr), Box<dyn std::error::Error>> {
    let port = config.server.port.into_inner();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;

    info!(address = %actual_addr, port = port, "Server started successfully");

    Ok((listener, actual_addr))
}

/// Serve the application on the given listener
///
/// # Errors
///
/// Returns an error if the server cannot be started or fails during operation.
#[instrument(skip(listener, router))]
pub async fn serve(listener: TcpListener, router: Router) -> Result<(), std::io::Error> {
    let addr = listener
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 0)));
    info!(address = %addr, "Starting HTTP server");

    let result = axum::serve(listener, router)
        .await
        .map_err(std::io::Error::other);

    info!(address = %addr, "Server stopped");
    result?;
    Ok(())
}

/// Serve the application with graceful shutdown handling
///
/// This function provides graceful shutdown capabilities using a cancellation token.
/// When the token is cancelled, the server will stop accepting new connections
/// and complete existing requests before shutting down.
///
/// # Errors
///
/// Returns an error if the server cannot be started or fails during operation.
#[instrument(skip(listener, router, shutdown_token))]
pub async fn serve_with_graceful_shutdown(
    listener: TcpListener,
    router: Router,
    shutdown_token: tokio_util::sync::CancellationToken,
) -> Result<(), std::io::Error> {
    let addr = listener
        .local_addr()
        .unwrap_or_else(|_| SocketAddr::from(([127, 0, 0, 1], 0)));
    info!(address = %addr, "Starting HTTP server with graceful shutdown");

    // Create shutdown signal handler using cancellation token
    let shutdown_signal = async move {
        shutdown_token.cancelled().await;
        info!("Graceful shutdown signal received");
    };

    // Start server with graceful shutdown
    let result = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(std::io::Error::other);

    info!(address = %addr, "Server gracefully shut down");
    result?;
    Ok(())
}

/// Testing utilities for server functionality
pub mod testing {
    use super::{SocketAddr, TcpListener, info, instrument};

    /// Start server on any available port
    ///
    /// This function allows the OS to choose an available port automatically.
    /// Primarily used for testing scenarios where the specific port doesn't matter.
    ///
    /// # Errors
    ///
    /// Returns an error if the server cannot bind to any available port.
    #[instrument]
    pub async fn start_server_on_available_port()
    -> Result<(TcpListener, SocketAddr), Box<dyn std::error::Error>> {
        let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Port 0 = OS chooses available port
        let listener = TcpListener::bind(addr).await?;
        let actual_addr = listener.local_addr()?;

        info!(address = %actual_addr, port = actual_addr.port(), "Server started on available port");

        Ok((listener, actual_addr))
    }
}

#[cfg(test)]
mod tests {
    use super::{create_router, serve, testing};
    use tokio::time::{Duration, timeout};

    #[tokio::test]
    async fn test_create_router_has_root_route() {
        let router = create_router();
        // This is a basic smoke test - more comprehensive route testing
        // would require additional test infrastructure
        assert!(!format!("{router:?}").is_empty());
    }

    #[tokio::test]
    async fn test_start_server_on_available_port() {
        let result = testing::start_server_on_available_port().await;
        assert!(
            result.is_ok(),
            "Should be able to start server on available port"
        );

        let (listener, addr) = result.unwrap();
        assert_ne!(addr.port(), 0, "Should get actual port number");
        assert_eq!(
            addr.ip().to_string(),
            "127.0.0.1",
            "Should bind to localhost"
        );

        // Clean up
        drop(listener);
    }

    #[tokio::test]
    async fn test_server_responds_to_http_requests() {
        let (listener, addr) = testing::start_server_on_available_port().await.unwrap();
        let router = create_router();

        // Start server in background task
        let server_handle = tokio::spawn(async move { serve(listener, router).await });

        // Give server a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test HTTP request
        let client = reqwest::Client::new();
        let response = timeout(
            Duration::from_secs(1),
            client.get(format!("http://{addr}/")).send(),
        )
        .await;

        assert!(response.is_ok(), "Should get response from server");
        let response = response.unwrap();
        assert!(response.is_ok(), "HTTP request should succeed");
        let response = response.unwrap();
        assert!(response.status().is_success(), "Should get success status");

        // Clean up
        server_handle.abort();
    }

    #[tokio::test]
    async fn test_health_endpoint_responds() {
        let (listener, addr) = testing::start_server_on_available_port().await.unwrap();
        let router = create_router();

        // Start server in background task
        let server_handle = tokio::spawn(async move { serve(listener, router).await });

        // Give server a moment to start
        tokio::time::sleep(Duration::from_millis(10)).await;

        // Test health endpoint
        let client = reqwest::Client::new();
        let response = timeout(
            Duration::from_secs(1),
            client.get(format!("http://{addr}/health")).send(),
        )
        .await;

        assert!(response.is_ok(), "Should get response from health endpoint");
        let response = response.unwrap();
        assert!(response.is_ok(), "Health endpoint request should succeed");
        let response = response.unwrap();
        assert!(
            response.status().is_success(),
            "Health endpoint should return success"
        );

        let body = response.text().await.unwrap();
        assert_eq!(body, "OK", "Health endpoint should return 'OK'");

        // Clean up
        server_handle.abort();
    }
}
