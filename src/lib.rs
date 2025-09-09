//! Caxton: A secure WebAssembly runtime for multi-agent systems
//!
//! This crate provides a secure, isolated execution environment for WebAssembly-based agents
//! with comprehensive resource management, security policies, and sandboxing capabilities.

/// Database module for embedded SQLite storage
pub mod database;
/// Advanced domain types for type-safe state management
pub mod domain;
/// Domain types for preventing primitive obsession
pub mod domain_types;
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
/// Storage layer for persisting agent registry, routing tables, and conversation state
pub mod storage;

/// Core agent lifecycle management orchestration
pub mod agent_lifecycle_manager;
/// CI pipeline configuration validation
pub mod ci_pipeline;
/// Agent deployment operations and strategies
pub mod deployment_manager;
/// Hot reload operations with zero downtime
pub mod hot_reload_manager;
/// REST API endpoints for management interface
pub mod rest_api;
/// Time abstraction layer for testable time operations
pub mod time_provider;
/// WASM module validation and security analysis
pub mod wasm_module_validator;

pub use database::{
    DatabaseConfig, DatabaseConnection, DatabaseError, DatabasePath, DatabaseResult,
};
pub use domain_types::{
    AgentId, AgentName, ConnectionPoolSize, CpuFuel, CpuFuelAmount, CpuFuelConsumed, ExecutionTime,
    FunctionDescription, FunctionModuleName, HostFunctionName, MaxAgentMemory, MaxAgents,
    MaxExports, MaxImportFunctions, MaxTableEntries, MaxTotalMemory, MemoryBytes, MessageCount,
    MessageSize, PermissionName, QueueDepth, RateLimitPerSecond, ResourceCreationError,
    RetryAttempt, StorageCleanupIntervalMs, TestAgentId, TestSequence, ValidationError, WorkerId,
};
// Re-export key domain types at the crate level
pub use domain::{
    AgentLifecycle, AgentLifecycleState, AgentVersion, CustomValidationRule, DeploymentConfig,
    DeploymentError, DeploymentId, DeploymentRequest, DeploymentResult, DeploymentStatus,
    DeploymentStrategy, HotReloadConfig, HotReloadError, HotReloadId, HotReloadRequest,
    HotReloadResult, HotReloadStatus, HotReloadStrategy, ResourceRequirements,
    TrafficSplitPercentage, ValidationFailure, ValidationResult, ValidationRuleType,
    ValidationWarning, VersionNumber, WasmModule, WasmSecurityPolicy, WasmValidationError,
};
pub use resource_manager::ResourceLimits;
pub use runtime::{WasmRuntime, WasmRuntimeConfig};
pub use sandbox::Sandbox;
pub use security::SecurityPolicy;

// Re-export Agent Lifecycle Management components
pub use agent_lifecycle_manager::{
    AgentLifecycleManager, AgentStatus, DeploymentManager as DeploymentManagerTrait, HealthStatus,
    HotReloadManager as HotReloadManagerTrait, LifecycleError, OperationResult,
    WasmModuleValidator as WasmModuleValidatorTrait,
};
pub use deployment_manager::{CaxtonDeploymentManager, InstanceManager, ResourceAllocator};
pub use hot_reload_manager::{CaxtonHotReloadManager, RuntimeManager, TrafficRouter};
pub use rest_api::{
    Agent, AgentDeploymentRequest, DeploymentResponse, ErrorResponse, HealthCheckResponse,
    create_app, start_server,
};
pub use wasm_module_validator::{
    CaxtonWasmModuleValidator, ValidationConfig, ValidationStatistics,
};
