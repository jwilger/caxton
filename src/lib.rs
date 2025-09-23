//! The public API for caxton

use axum::{Router, http::StatusCode, response::Json, routing::get};
use serde_json::json;

/// Health endpoint handler
async fn health_handler() -> (StatusCode, Json<serde_json::Value>) {
    (StatusCode::OK, Json(json!({"status": "OK"})))
}

/// Create application router
pub fn create_app() -> Router {
    Router::new().route("/health", get(health_handler))
}
