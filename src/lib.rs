//! Caxton: A secure WebAssembly runtime for multi-agent systems
//!
//! This crate provides a secure, isolated execution environment for WebAssembly-based agents
//! with comprehensive resource management, security policies, and sandboxing capabilities.

/// Domain types for preventing primitive obsession
pub mod domain_types;
/// Advanced domain types for type-safe state management
// pub mod domain; // TODO: Fix compilation issues before enabling
/// Host function registry and management
pub mod host_functions;
/// High-performance async message router for agent communication
pub mod message_router;
/// Resource management and limit enforcement
pub mod resource_manager;
/// WebAssembly runtime module for agent lifecycle management
pub mod runtime;
/// Sandbox module for isolated agent execution
pub mod sandbox;
/// Security policy configuration and enforcement
pub mod security;

pub use domain_types::{
    AgentId, AgentName, ConnectionPoolSize, CpuFuel, ExecutionTime, FunctionDescription,
    FunctionModuleName, HostFunctionName, MaxAgentMemory, MaxAgents, MaxExports,
    MaxImportFunctions, MaxTableEntries, MaxTotalMemory, MemoryBytes, MessageCount, MessageSize,
    PermissionName, RateLimitPerSecond, StorageCleanupIntervalMs,
};
pub use resource_manager::ResourceLimits;
pub use runtime::{WasmRuntime, WasmRuntimeConfig};
pub use sandbox::Sandbox;
pub use security::SecurityPolicy;
