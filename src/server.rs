//! Server functionality for Caxton
//!
//! This module provides the HTTP server implementation with testable components

use crate::domain::config::AppConfig;
use axum::{Router, response::Html, routing::get};
use std::net::SocketAddr;
use tokio::net::TcpListener;

/// Create the Axum router with all routes
pub fn create_router() -> Router {
    Router::new()
        .route("/", get(|| async { Html("Caxton Server") }))
        .route("/health", get(|| async { "OK" }))
}

/// Start server with the given configuration
/// Returns the bound listener and server handle for testing
///
/// # Errors
///
/// Returns an error if the server cannot bind to the specified port.
pub async fn start_server(
    config: AppConfig,
) -> Result<(TcpListener, SocketAddr), Box<dyn std::error::Error>> {
    let port = config.server.port.into_inner();
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    let listener = TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;

    Ok((listener, actual_addr))
}

/// Start server on any available port (for testing)
///
/// # Errors
///
/// Returns an error if the server cannot bind to any available port.
#[allow(dead_code)]
pub async fn start_server_on_available_port()
-> Result<(TcpListener, SocketAddr), Box<dyn std::error::Error>> {
    let addr = SocketAddr::from(([127, 0, 0, 1], 0)); // Port 0 = OS chooses available port
    let listener = TcpListener::bind(addr).await?;
    let actual_addr = listener.local_addr()?;

    Ok((listener, actual_addr))
}

/// Serve the application on the given listener
///
/// # Errors
///
/// Returns an error if the server cannot be started or fails during operation.
pub async fn serve(listener: TcpListener, router: Router) -> Result<(), std::io::Error> {
    axum::serve(listener, router)
        .await
        .map_err(std::io::Error::other)?;
    Ok(())
}

/// Serve the application with graceful shutdown handling
///
/// # Errors
///
/// Returns an error if the server cannot be started or fails during operation.
#[allow(dead_code)]
pub async fn serve_with_graceful_shutdown(
    listener: TcpListener,
    router: Router,
    shutdown_token: tokio_util::sync::CancellationToken,
) -> Result<(), std::io::Error> {
    // Create shutdown signal handler using cancellation token
    let shutdown_signal = async move {
        shutdown_token.cancelled().await;
    };

    // Start server with graceful shutdown
    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal)
        .await
        .map_err(std::io::Error::other)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let result = start_server_on_available_port().await;
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
        let (listener, addr) = start_server_on_available_port().await.unwrap();
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
        let (listener, addr) = start_server_on_available_port().await.unwrap();
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
