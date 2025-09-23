//! Performance tests for health endpoint
//!
//! These tests verify the <100ms response time requirement
//! following Constitutional Principle XII

use axum::{Router, body::Body, http::Request};
use std::time::Instant;
use tower::ServiceExt;

/// Test health endpoint performance requirement (<100ms)
///
/// Constitutional Step 1: Write black-box performance test
/// This test verifies the response time under normal conditions
#[tokio::test]
async fn test_health_endpoint_performance_under_100ms() {
    let app = create_health_app();

    // Run multiple requests to get average performance
    let mut durations = Vec::new();
    for _ in 0..10 {
        let request = Request::builder()
            .uri("/health")
            .method("GET")
            .body(Body::empty())
            .unwrap();

        let start_time = Instant::now();
        let response = app.clone().oneshot(request).await.unwrap();
        let duration = start_time.elapsed();

        assert_eq!(response.status(), 200);
        durations.push(duration);
    }

    // Verify all requests completed under 100ms
    for (i, duration) in durations.iter().enumerate() {
        assert!(
            duration.as_millis() < 100,
            "Request {} took {}ms, should be <100ms",
            i,
            duration.as_millis()
        );
    }

    // Calculate and verify average performance
    let total_duration: std::time::Duration = durations.iter().sum();
    let avg_duration = total_duration
        / durations
            .len()
            .try_into()
            .expect("Too many test iterations");
    assert!(
        avg_duration.as_millis() < 100,
        "Average response time {}ms should be <100ms",
        avg_duration.as_millis()
    );
}

/// Helper function to create test application router
/// Uses the actual application router from the library
fn create_health_app() -> Router {
    caxton::create_app()
}
