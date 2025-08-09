//! Caxton: A secure WebAssembly runtime for multi-agent systems
//!
//! This crate provides a secure, isolated execution environment for WebAssembly-based agents
//! with comprehensive resource management, security policies, and sandboxing capabilities.

/// Host function registry and management
pub mod host_functions;
/// Resource management and limit enforcement
pub mod resource_manager;
/// WebAssembly runtime module for agent lifecycle management
pub mod runtime;
/// Sandbox module for isolated agent execution
pub mod sandbox;
/// Security policy configuration and enforcement
pub mod security;

pub use resource_manager::ResourceLimits;
pub use runtime::{AgentId, WasmRuntime, WasmRuntimeConfig};
pub use sandbox::Sandbox;
pub use security::SecurityPolicy;
