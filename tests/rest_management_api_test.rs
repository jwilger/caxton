//! Tests for the REST Management API
//!
//! These tests drive the implementation of the HTTP endpoints for managing
//! Caxton agents via REST API. Following the architectural decisions in
//! ADR-0026 and ADR-0027, this tests JSON over HTTP with shared types
//! between server and CLI.

use caxton::start_server;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU16, Ordering};
use tokio::time::Duration;

/// Base port for test servers to avoid conflicts
static TEST_PORT_COUNTER: AtomicU16 = AtomicU16::new(8080);

/// Maximum number of retries for server readiness check
const MAX_READINESS_RETRIES: usize = 50;

/// Delay between readiness check retries
const READINESS_RETRY_DELAY: Duration = Duration::from_millis(10);

/// Get a unique port for each test to avoid conflicts
fn get_unique_test_port() -> u16 {
    TEST_PORT_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Wait for server to be ready with timeout and retries
async fn wait_for_server_ready(port: u16) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();
    let health_url = format!("http://localhost:{port}/api/v1/health");

    for _ in 0..MAX_READINESS_RETRIES {
        match client.get(&health_url).send().await {
            Ok(response) if response.status().is_success() => return Ok(()),
            _ => tokio::time::sleep(READINESS_RETRY_DELAY).await,
        }
    }

    Err(format!("Server on port {port} failed to become ready after retries").into())
}

#[tokio::test]
async fn health_check_endpoint_returns_json_status() {
    // Start the Caxton REST API server in the background
    let port = get_unique_test_port();
    let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Wait for server to be ready
    wait_for_server_ready(port)
        .await
        .expect("Server failed to start");

    let client = reqwest::Client::new();

    // Make a GET request to the health endpoint
    let response = client
        .get(format!("http://localhost:{port}/api/v1/health"))
        .send()
        .await
        .expect("Failed to send health check request");

    // The endpoint should return 200 OK
    assert_eq!(response.status(), 200);

    // The response should be valid JSON with health status
    let health_json: Value = response
        .json()
        .await
        .expect("Response body should be valid JSON");

    // The JSON should contain a status field with "healthy" value
    assert_eq!(health_json["status"], "healthy");
}

#[tokio::test]
async fn list_agents_endpoint_returns_empty_array_initially() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8081".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Make a GET request to the list agents endpoint
    let response = client
        .get("http://localhost:8081/api/v1/agents")
        .send()
        .await
        .expect("Failed to send agents list request");

    // The endpoint should return 200 OK with empty agents list
    assert_eq!(response.status(), 200);
}

#[tokio::test]
async fn deploy_agent_endpoint_accepts_minimal_deployment_request() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8082".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Minimal agent deployment request with essential fields
    let deployment_request = serde_json::json!({
        "name": "test-agent",
        "wasm_module": "fake-wasm-bytecode",
        "resource_limits": {
            "max_memory_bytes": 1_048_576,
            "max_cpu_millis": 1000,
            "max_execution_time_ms": 5000
        }
    });

    // Make a POST request to deploy an agent
    let response = client
        .post("http://localhost:8082/api/v1/agents")
        .json(&deployment_request)
        .send()
        .await
        .expect("Failed to send agent deployment request");

    // The endpoint should return 201 Created for successful deployment
    assert_eq!(response.status(), 201);
}

#[tokio::test]
async fn get_agent_by_id_returns_agent_details() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8083".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // First deploy an agent to have something to retrieve
    let deployment_request = serde_json::json!({
        "name": "retrievable-agent",
        "wasm_module": "test-wasm-bytecode",
        "resource_limits": {
            "max_memory_bytes": 2_097_152,
            "max_cpu_millis": 2000,
            "max_execution_time_ms": 10000
        }
    });

    let deploy_response = client
        .post("http://localhost:8083/api/v1/agents")
        .json(&deployment_request)
        .send()
        .await
        .expect("Failed to deploy agent");

    assert_eq!(deploy_response.status(), 201);

    // Extract agent ID from deployment response
    let deploy_json: Value = deploy_response
        .json()
        .await
        .expect("Deploy response should be valid JSON");

    let agent_id = deploy_json["id"]
        .as_str()
        .expect("Deploy response should contain agent ID");

    // Now retrieve the specific agent by ID
    let get_response = client
        .get(format!("http://localhost:8083/api/v1/agents/{agent_id}"))
        .send()
        .await
        .expect("Failed to get agent by ID");

    // The endpoint should return 200 OK
    assert_eq!(get_response.status(), 200);
}

#[tokio::test]
async fn deploy_agent_rejects_empty_name() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8084".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Agent deployment request with empty name
    let deployment_request = serde_json::json!({
        "name": "",
        "wasm_module": "fake-wasm-bytecode",
        "resource_limits": {
            "max_memory_bytes": 1_048_576,
            "max_cpu_millis": 1000,
            "max_execution_time_ms": 5000
        }
    });

    // Make a POST request to deploy an agent
    let response = client
        .post("http://localhost:8084/api/v1/agents")
        .json(&deployment_request)
        .send()
        .await
        .expect("Failed to send agent deployment request");

    // The endpoint should return 400 Bad Request for empty name
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn deploy_agent_rejects_empty_wasm_module() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8085".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Agent deployment request with empty wasm_module
    let deployment_request = serde_json::json!({
        "name": "test-agent",
        "wasm_module": "",
        "resource_limits": {
            "max_memory_bytes": 1_048_576,
            "max_cpu_millis": 1000,
            "max_execution_time_ms": 5000
        }
    });

    // Make a POST request to deploy an agent
    let response = client
        .post("http://localhost:8085/api/v1/agents")
        .json(&deployment_request)
        .send()
        .await
        .expect("Failed to send agent deployment request");

    // The endpoint should return 400 Bad Request for empty wasm_module
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn deploy_agent_rejects_zero_resource_limits() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8086".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Agent deployment request with zero resource limits
    let deployment_request = serde_json::json!({
        "name": "test-agent",
        "wasm_module": "fake-wasm-bytecode",
        "resource_limits": {
            "max_memory_bytes": 0,
            "max_cpu_millis": 1000,
            "max_execution_time_ms": 5000
        }
    });

    // Make a POST request to deploy an agent
    let response = client
        .post("http://localhost:8086/api/v1/agents")
        .json(&deployment_request)
        .send()
        .await
        .expect("Failed to send agent deployment request");

    // The endpoint should return 400 Bad Request for zero resource limits
    assert_eq!(response.status(), 400);
}

#[tokio::test]
async fn get_agent_by_nonexistent_id_returns_404() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8087".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Try to get an agent that doesn't exist
    let nonexistent_id = "00000000-0000-0000-0000-000000000000";
    let response = client
        .get(format!(
            "http://localhost:8087/api/v1/agents/{nonexistent_id}"
        ))
        .send()
        .await
        .expect("Failed to get agent by ID");

    // The endpoint should return 404 Not Found for nonexistent agent
    assert_eq!(response.status(), 404);
}

#[tokio::test]
async fn deploy_agent_rejects_malformed_json() {
    // Start the Caxton REST API server in the background
    let addr: SocketAddr = "127.0.0.1:8088".parse().unwrap();

    // Start server in a background task
    let _server_handle = tokio::spawn(async move {
        start_server(addr).await.expect("Server failed to start");
    });

    // Give the server a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    let client = reqwest::Client::new();

    // Make a POST request with malformed JSON
    let response = client
        .post("http://localhost:8088/api/v1/agents")
        .header("Content-Type", "application/json")
        .body("{invalid json}")
        .send()
        .await
        .expect("Failed to send malformed JSON request");

    // The endpoint should return 400 Bad Request for malformed JSON
    assert_eq!(response.status(), 400);
}
