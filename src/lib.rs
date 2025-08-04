//! # Caxton - Multi-Agent Orchestration Platform
//!
//! Caxton is a foundational platform service for multi-agent orchestration, providing
//! WebAssembly-based agent isolation, FIPA protocol messaging, and comprehensive
//! observability through structured logging and OpenTelemetry integration.
//!
//! ## Performance Optimization
//!
//! This crate includes extensive performance optimizations:
//! - High-performance WebAssembly runtime with instance pooling
//! - Optimized FIPA message routing with batching and zero-copy serialization
//! - Memory allocation tracking and optimization with global allocator wrapper
//! - Batched observability events processing for minimal overhead
//! - Comprehensive benchmarking suite for performance regression testing

pub mod performance;

// ## Core Components
//
// - **Agent Management**: Type-safe agent lifecycle with phantom types
// - **FIPA Messaging**: Standards-compliant agent communication protocol
// - **WebAssembly Isolation**: Secure sandboxing for multi-agent systems
// - **Observability**: OpenTelemetry integration with structured events
// - **MCP Integration**: Model Context Protocol for external tool access
//
// ## Architecture
//
// Caxton follows a "functional core, imperative shell" architecture with
// observability-first design principles:
//
// ```rust
// use caxton::*;
//
// #[tokio::main]
// async fn main() -> Result<(), Box<dyn std::error::Error>> {
//     let runtime = CaxtonRuntime::new(CaxtonConfig::default()).await?;
//
//     let agent_id = runtime.spawn_agent(AgentConfig {
//         name: "hello-agent".to_string(),
//         agent_type: AgentType::Worker,
//         capabilities: vec!["greeting".to_string()],
//         max_memory: Some(32 * 1024 * 1024),
//         timeout: Some(std::time::Duration::from_secs(10)),
//     }).await?;
//
//     let message = FipaMessage {
//         performative: FipaPerformative::Request,
//         sender: AgentId::system(),
//         receiver: agent_id,
//         content: serde_json::json!({"action": "greet", "name": "World"}),
//         protocol: Some("greeting".to_string()),
//         ..Default::default()
//     };
//
//     let response = runtime.send_message(message).await?;
//     println!("Response: {:?}", response);
//
//     Ok(())
// }
// ```

// Re-export all public types and functions
pub use crate::agent::*;
pub use crate::core::*;
pub use crate::error::*;
pub use crate::fipa::*;
pub use crate::observability::*;
pub use crate::runtime::*;
pub use crate::wasm::*;

// Core modules
pub mod agent;
pub mod core;
pub mod fipa;
pub mod lifecycle;
pub mod observability;
pub mod runtime;

// Test modules
#[cfg(test)]
pub mod tests {
    pub mod lifecycle_tests;
}
pub mod error;
pub mod wasm;

// Additional functionality
pub mod mcp;
pub mod utils;

// Common imports
pub use ::tracing::{debug, error, info, instrument, warn};
pub use chrono::{DateTime, Utc};
pub use serde::{Deserialize, Serialize};
pub use std::collections::HashMap;
pub use std::time::Duration;
pub use thiserror::Error;
pub use tokio::time::timeout;
pub use uuid::Uuid;
