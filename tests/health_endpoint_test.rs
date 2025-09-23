//! Black-box integration tests for health endpoint
//!
//! These tests verify only externally visible HTTP behavior without
//! knowledge of internal implementation structure, following
//! Constitutional Principle XII: Outside-In Black-Box Testing Methodology

use axum::{body::Body, http::Request};
use std::time::Instant;
use tower::ServiceExt; // for `oneshot` method

/// Test GET /health endpoint returns proper JSON response
///
/// Constitutional Step 1: Write black-box integration test testing only externally visible behavior
/// This test MUST fail initially (no implementation exists)
#[tokio::test]
async fn test_get_health_endpoint_returns_ok_response() {
    // Arrange: Create application router (this will fail until router is implemented)
    let app = create_health_app();

    // Act: Make GET request to /health endpoint
    let request = Request::builder()
        .uri("/health")
        .method("GET")
        .body(Body::empty())
        .unwrap();

    let start_time = Instant::now();
    let response = app.oneshot(request).await.unwrap();
    let duration = start_time.elapsed();

    // Assert: Verify complete HTTP contract
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    // Verify response time requirement (<100ms)
    assert!(
        duration.as_millis() < 100,
        "Health endpoint took {}ms, should be <100ms",
        duration.as_millis()
    );

    // Verify JSON response body
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(body, serde_json::json!({"status": "OK"}));
}

/// Test HEAD /health endpoint returns proper headers without body
///
/// Constitutional Step 1: Write next black-box integration test
/// This test should pass since HEAD is just GET without body
#[tokio::test]
async fn test_head_health_endpoint_returns_ok_headers() {
    // Arrange: Create application router
    let app = create_health_app();

    // Act: Make HEAD request to /health endpoint
    let request = Request::builder()
        .uri("/health")
        .method("HEAD")
        .body(Body::empty())
        .unwrap();

    let start_time = Instant::now();
    let response = app.oneshot(request).await.unwrap();
    let duration = start_time.elapsed();

    // Assert: Verify HEAD response contract
    assert_eq!(response.status(), 200);
    assert_eq!(
        response.headers().get("content-type").unwrap(),
        "application/json"
    );

    // Verify response time requirement (<100ms)
    assert!(
        duration.as_millis() < 100,
        "Health endpoint took {}ms, should be <100ms",
        duration.as_millis()
    );

    // Verify no response body for HEAD request
    let body_bytes = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    assert!(
        body_bytes.is_empty(),
        "HEAD request should return empty body"
    );
}

/// Test unsupported HTTP methods return 405 Method Not Allowed
///
/// Constitutional Step 1: Write next black-box integration test
/// This test should fail initially as we haven't explicitly handled method rejection
#[tokio::test]
async fn test_unsupported_methods_return_405() {
    let app = create_health_app();

    // Test POST method (should be rejected)
    let request = Request::builder()
        .uri("/health")
        .method("POST")
        .body(Body::empty())
        .unwrap();

    let response = app.oneshot(request).await.unwrap();

    // Assert: Should return 405 Method Not Allowed
    assert_eq!(response.status(), 405);

    // Should include Allow header showing supported methods
    let allow_header = response.headers().get("allow");
    assert!(
        allow_header.is_some(),
        "405 response should include Allow header"
    );
}

/// Helper function to create test application router
/// Uses the actual application router from the library
fn create_health_app() -> axum::Router {
    caxton::create_app()
}
