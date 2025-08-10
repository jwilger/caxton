//! Caxton: A secure WebAssembly runtime for multi-agent systems
//!
//! This crate provides a secure, isolated execution environment for WebAssembly-based agents
//! with comprehensive resource management, security policies, and sandboxing capabilities.

#![allow(clippy::uninlined_format_args)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::if_not_else)]
#![allow(clippy::manual_div_ceil)]
#![allow(clippy::map_unwrap_or)]
#![allow(clippy::redundant_closure_for_method_calls)]
#![allow(clippy::struct_excessive_bools)]
#![allow(clippy::new_without_default)]
#![allow(clippy::unused_self)]
#![allow(clippy::unnecessary_wraps)]
#![allow(clippy::return_self_not_must_use)]
#![allow(clippy::cast_sign_loss)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::match_same_arms)]
#![allow(clippy::no_effect_underscore_binding)]
#![allow(clippy::absurd_extreme_comparisons)]
#![allow(clippy::useless_vec)]
#![allow(clippy::type_complexity)]
#![allow(clippy::float_cmp)]

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

/// Core agent lifecycle management orchestration
pub mod agent_lifecycle_manager;
/// Agent deployment operations and strategies
pub mod deployment_manager;
/// Hot reload operations with zero downtime
pub mod hot_reload_manager;
/// Time abstraction layer for testable time operations
pub mod time_provider;
/// WASM module validation and security analysis
pub mod wasm_module_validator;

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
pub use wasm_module_validator::{
    CaxtonWasmModuleValidator, ValidationConfig, ValidationStatistics,
};
