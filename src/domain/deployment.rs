//! Agent Deployment domain types
//!
//! This module defines types for managing agent deployment operations,
//! resource requirements, deployment strategies, and deployment results.

use nutype::nutype;
use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

use super::agent_lifecycle::{AgentVersion, VersionNumber};
use crate::domain_types::{AgentId, AgentName, CpuFuel, MemoryBytes};

/// Unique identifier for a deployment operation
#[nutype(derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    Serialize,
    Deserialize,
    Display,
    TryFrom,
    Into
))]
pub struct DeploymentId(Uuid);

impl DeploymentId {
    /// Creates a new random deployment ID
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Deployment strategy for rolling out agent versions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeploymentStrategy {
    /// Replace all instances immediately
    Immediate,
    /// Rolling deployment with configurable batch size
    Rolling,
    /// Blue-green deployment with traffic switching
    BlueGreen,
    /// Canary deployment with gradual traffic increase
    Canary,
}

impl DeploymentStrategy {
    /// Check if strategy supports gradual rollout
    pub fn supports_gradual_rollout(&self) -> bool {
        matches!(self, Self::Rolling | Self::Canary)
    }

    /// Check if strategy supports instant rollback
    pub fn supports_instant_rollback(&self) -> bool {
        matches!(self, Self::BlueGreen | Self::Canary)
    }
}

/// Batch size for rolling deployments
#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1
)]
pub struct BatchSize(u8);

impl BatchSize {
    /// Gets the value as u8
    pub fn as_u8(&self) -> u8 {
        self.into_inner()
    }

    /// Creates batch size as percentage of total instances
    ///
    /// # Errors
    ///
    /// Returns `BatchSizeError` if the calculated batch size is invalid or if percentage > 100.
    pub fn from_percentage(percentage: u8, total_instances: usize) -> Result<Self, BatchSizeError> {
        if percentage > 100 {
            return Err(Self::try_new(101).unwrap_err()); // This will trigger the validation error
        }

        #[allow(clippy::cast_precision_loss)]
        let total_as_f32 = total_instances as f32;
        let calculated = (total_as_f32 * f32::from(percentage) / 100.0).ceil();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let batch_size = (calculated as u8).max(1);
        Self::try_new(batch_size)
    }
}

/// Timeout for deployment operations
#[nutype(
    validate(greater_or_equal = 30_000, less_or_equal = 1_800_000), // 30 seconds to 30 minutes
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 300_000 // 5 minutes
)]
pub struct DeploymentTimeout(u64);

impl DeploymentTimeout {
    /// Creates timeout from minutes
    ///
    /// # Errors
    ///
    /// Returns `DeploymentTimeoutError` if the timeout in milliseconds is outside the valid range.
    pub fn from_mins(mins: u64) -> Result<Self, DeploymentTimeoutError> {
        Self::try_new(mins * 60 * 1000)
    }

    /// Gets the value as milliseconds
    pub fn as_millis(&self) -> u64 {
        self.into_inner()
    }

    /// Gets the value as Duration
    pub fn as_duration(&self) -> Duration {
        Duration::from_millis(self.into_inner())
    }
}

/// Maximum memory allowed for deployment
#[nutype(
    validate(greater_or_equal = 1_048_576, less_or_equal = 1_073_741_824), // 1MB to 1GB
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 10_485_760 // 10MB
)]
pub struct DeploymentMemoryLimit(usize);

impl DeploymentMemoryLimit {
    /// Creates limit from megabytes
    ///
    /// # Errors
    ///
    /// Returns `DeploymentMemoryLimitError` if the memory limit in bytes is outside the valid range.
    pub fn from_mb(mb: usize) -> Result<Self, DeploymentMemoryLimitError> {
        Self::try_new(mb * 1024 * 1024)
    }

    /// Gets the value as bytes
    pub fn as_bytes(&self) -> usize {
        self.into_inner()
    }

    /// Convert to `MemoryBytes` for compatibility
    pub fn as_memory_bytes(&self) -> MemoryBytes {
        MemoryBytes::try_new(self.into_inner()).unwrap_or_default()
    }
}

/// Maximum CPU fuel allowed for deployment
#[nutype(
    validate(greater_or_equal = 10_000, less_or_equal = 100_000_000), // 10K to 100M
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 1_000_000 // 1M
)]
pub struct DeploymentFuelLimit(u64);

impl DeploymentFuelLimit {
    /// Gets the value as u64
    pub fn as_u64(&self) -> u64 {
        self.into_inner()
    }

    /// Convert to `CpuFuel` for compatibility
    pub fn as_cpu_fuel(&self) -> CpuFuel {
        CpuFuel::try_new(self.into_inner()).unwrap_or_default()
    }
}

/// Health check configuration for deployments
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub initial_delay: Duration,
    pub interval: Duration,
    pub timeout: Duration,
    pub success_threshold: u32,
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            initial_delay: Duration::from_secs(10),
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            success_threshold: 2,
            failure_threshold: 3,
        }
    }
}

/// Resource requirements for agent deployment
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub memory_limit: DeploymentMemoryLimit,
    pub fuel_limit: DeploymentFuelLimit,
    pub requires_isolation: bool,
    pub max_concurrent_requests: Option<u32>,
}

impl ResourceRequirements {
    /// Creates new resource requirements
    pub fn new(memory_limit: DeploymentMemoryLimit, fuel_limit: DeploymentFuelLimit) -> Self {
        Self {
            memory_limit,
            fuel_limit,
            requires_isolation: true,
            max_concurrent_requests: Some(100),
        }
    }

    /// Creates minimal resource requirements for testing
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded minimal values are invalid (should never happen).
    pub fn minimal() -> Self {
        Self {
            memory_limit: DeploymentMemoryLimit::try_new(1_048_576).unwrap(), // 1MB
            fuel_limit: DeploymentFuelLimit::try_new(10_000).unwrap(),        // 10K
            requires_isolation: false,
            max_concurrent_requests: Some(1),
        }
    }

    /// Check if requirements are compatible with system limits
    pub fn is_compatible_with(&self, system_memory: usize, system_fuel: u64) -> bool {
        self.memory_limit.as_bytes() <= system_memory && self.fuel_limit.as_u64() <= system_fuel
    }
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self::new(
            DeploymentMemoryLimit::default(),
            DeploymentFuelLimit::default(),
        )
    }
}

/// Deployment configuration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub strategy: DeploymentStrategy,
    pub batch_size: BatchSize,
    pub timeout: DeploymentTimeout,
    pub resource_requirements: ResourceRequirements,
    pub health_check: HealthCheckConfig,
    pub auto_rollback: bool,
    pub rollback_threshold_percentage: u8,
}

impl DeploymentConfig {
    /// Creates a new deployment configuration
    pub fn new(strategy: DeploymentStrategy) -> Self {
        Self {
            strategy,
            batch_size: BatchSize::default(),
            timeout: DeploymentTimeout::default(),
            resource_requirements: ResourceRequirements::default(),
            health_check: HealthCheckConfig::default(),
            auto_rollback: true,
            rollback_threshold_percentage: 10,
        }
    }

    /// Creates configuration for immediate deployment
    pub fn immediate() -> Self {
        Self::new(DeploymentStrategy::Immediate)
    }

    /// Creates configuration for rolling deployment
    pub fn rolling(batch_size: BatchSize) -> Self {
        let mut config = Self::new(DeploymentStrategy::Rolling);
        config.batch_size = batch_size;
        config
    }

    /// Creates configuration for canary deployment
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded batch size value is invalid (should never happen).
    pub fn canary() -> Self {
        let mut config = Self::new(DeploymentStrategy::Canary);
        config.batch_size = BatchSize::try_new(1).unwrap(); // Start with 1 instance
        config.health_check.enabled = true;
        config
    }
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self::new(DeploymentStrategy::Rolling)
    }
}

/// Deployment request containing all necessary information
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeploymentRequest {
    pub deployment_id: DeploymentId,
    pub agent_id: AgentId,
    pub agent_name: Option<AgentName>,
    pub from_version: Option<AgentVersion>,
    pub to_version: AgentVersion,
    pub to_version_number: VersionNumber,
    pub config: DeploymentConfig,
    pub wasm_module_bytes: Vec<u8>,
    pub requested_at: SystemTime,
}

impl DeploymentRequest {
    /// Creates a new deployment request
    pub fn new(
        agent_id: AgentId,
        agent_name: Option<AgentName>,
        from_version: Option<AgentVersion>,
        to_version: AgentVersion,
        to_version_number: VersionNumber,
        config: DeploymentConfig,
        wasm_module_bytes: Vec<u8>,
    ) -> Self {
        Self {
            deployment_id: DeploymentId::generate(),
            agent_id,
            agent_name,
            from_version,
            to_version,
            to_version_number,
            config,
            wasm_module_bytes,
            requested_at: SystemTime::now(),
        }
    }

    /// Check if this is an initial deployment (no previous version)
    pub fn is_initial_deployment(&self) -> bool {
        self.from_version.is_none()
    }

    /// Check if this is an upgrade deployment
    pub fn is_upgrade(&self) -> bool {
        self.from_version.is_some()
    }

    /// Get module size in bytes
    pub fn module_size(&self) -> usize {
        self.wasm_module_bytes.len()
    }

    /// Validate the deployment request
    ///
    /// # Errors
    ///
    /// Returns `DeploymentValidationError` if the request is invalid (empty WASM, too large, invalid config).
    pub fn validate(&self) -> Result<(), DeploymentValidationError> {
        if self.wasm_module_bytes.is_empty() {
            return Err(DeploymentValidationError::EmptyWasmModule);
        }

        if self.wasm_module_bytes.len() > 50 * 1024 * 1024 {
            // 50MB max
            return Err(DeploymentValidationError::WasmModuleTooLarge {
                size: self.wasm_module_bytes.len(),
                max: 50 * 1024 * 1024,
            });
        }

        if self.config.rollback_threshold_percentage > 100 {
            return Err(DeploymentValidationError::InvalidRollbackThreshold {
                threshold: self.config.rollback_threshold_percentage,
            });
        }

        Ok(())
    }
}

/// Deployment status enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum DeploymentStatus {
    /// Deployment requested but not started
    Pending,
    /// Deployment currently in progress
    InProgress,
    /// Deployment completed successfully
    Completed,
    /// Deployment failed
    Failed,
    /// Deployment was cancelled
    Cancelled,
    /// Deployment was rolled back
    RolledBack,
}

impl DeploymentStatus {
    /// Check if status is terminal (no further transitions expected)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed | Self::Failed | Self::Cancelled | Self::RolledBack
        )
    }

    /// Check if status indicates success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Completed)
    }

    /// Check if status indicates failure
    pub fn is_failure(&self) -> bool {
        matches!(self, Self::Failed | Self::Cancelled | Self::RolledBack)
    }
}

/// Deployment progress tracking
#[nutype(
    validate(greater_or_equal = 0, less_or_equal = 100),
    derive(
        Debug,
        Clone,
        Copy,
        PartialEq,
        Eq,
        PartialOrd,
        Ord,
        Serialize,
        Deserialize,
        Display,
        Default,
        TryFrom,
        Into
    ),
    default = 0
)]
pub struct DeploymentProgress(u8);

impl DeploymentProgress {
    /// Creates progress from percentage (0-100)
    ///
    /// # Errors
    ///
    /// Returns `DeploymentProgressError` if percentage is greater than 100.
    pub fn from_percentage(percentage: u8) -> Result<Self, DeploymentProgressError> {
        Self::try_new(percentage)
    }

    /// Creates completed progress (100%)
    ///
    /// # Panics
    ///
    /// Panics if 100 is somehow invalid (should never happen).
    pub fn completed() -> Self {
        Self::try_new(100).unwrap()
    }

    /// Gets the value as percentage
    pub fn as_percentage(&self) -> u8 {
        self.into_inner()
    }

    /// Check if deployment is complete
    pub fn is_complete(&self) -> bool {
        self.into_inner() == 100
    }
}

/// Deployment result with comprehensive information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub deployment_id: DeploymentId,
    pub agent_id: AgentId,
    pub status: DeploymentStatus,
    pub progress: DeploymentProgress,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error_message: Option<String>,
    pub rollback_version: Option<AgentVersion>,
    pub metrics: Option<DeploymentMetrics>,
}

impl DeploymentResult {
    /// Creates a successful deployment result
    pub fn success(
        deployment_id: DeploymentId,
        agent_id: AgentId,
        started_at: SystemTime,
        completed_at: SystemTime,
        metrics: Option<DeploymentMetrics>,
    ) -> Self {
        Self {
            deployment_id,
            agent_id,
            status: DeploymentStatus::Completed,
            progress: DeploymentProgress::completed(),
            started_at: Some(started_at),
            completed_at: Some(completed_at),
            error_message: None,
            rollback_version: None,
            metrics,
        }
    }

    /// Creates a failed deployment result
    pub fn failure(
        deployment_id: DeploymentId,
        agent_id: AgentId,
        started_at: Option<SystemTime>,
        error_message: String,
        rollback_version: Option<AgentVersion>,
    ) -> Self {
        Self {
            deployment_id,
            agent_id,
            status: DeploymentStatus::Failed,
            progress: DeploymentProgress::default(),
            started_at,
            completed_at: Some(SystemTime::now()),
            error_message: Some(error_message),
            rollback_version,
            metrics: None,
        }
    }

    /// Get deployment duration if available
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        }
    }
}

/// Deployment performance metrics
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeploymentMetrics {
    pub instances_deployed: u32,
    pub instances_failed: u32,
    pub total_duration: Duration,
    pub average_instance_deployment_time: Duration,
    pub memory_usage_peak: usize,
    pub fuel_consumed: u64,
    pub health_check_success_rate: f32,
}

impl DeploymentMetrics {
    /// Calculate success rate as percentage
    pub fn success_rate_percentage(&self) -> f32 {
        let total = self.instances_deployed + self.instances_failed;
        if total == 0 {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        {
            (self.instances_deployed as f32 / total as f32) * 100.0
        }
    }

    /// Check if deployment meets success threshold
    pub fn meets_success_threshold(&self, threshold_percentage: u8) -> bool {
        self.success_rate_percentage() >= f32::from(threshold_percentage)
    }
}

/// Deployment validation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum DeploymentValidationError {
    #[error("WASM module is empty")]
    EmptyWasmModule,

    #[error("WASM module too large: {size} bytes, max {max} bytes")]
    WasmModuleTooLarge { size: usize, max: usize },

    #[error("Invalid rollback threshold: {threshold}%, must be 0-100")]
    InvalidRollbackThreshold { threshold: u8 },

    #[error("Resource requirements exceed system limits")]
    ResourceLimitsExceeded,

    #[error("Invalid deployment strategy for current state")]
    InvalidStrategy,

    #[error("Missing required deployment configuration: {field}")]
    MissingConfiguration { field: String },
}

/// Deployment operation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum DeploymentError {
    #[error("Deployment validation failed: {0}")]
    ValidationFailed(#[from] DeploymentValidationError),

    #[error("Deployment timeout exceeded: {timeout}ms")]
    TimeoutExceeded { timeout: u64 },

    #[error("Insufficient resources: {resource}")]
    InsufficientResources { resource: String },

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Deployment already in progress: {deployment_id}")]
    AlreadyInProgress { deployment_id: DeploymentId },

    #[error("Health check failed: {reason}")]
    HealthCheckFailed { reason: String },

    #[error("Rollback failed: {reason}")]
    RollbackFailed { reason: String },

    #[error("WASM module validation failed: {reason}")]
    WasmValidationFailed { reason: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_strategy() {
        let rolling = DeploymentStrategy::Rolling;
        assert!(rolling.supports_gradual_rollout());
        assert!(!rolling.supports_instant_rollback());

        let blue_green = DeploymentStrategy::BlueGreen;
        assert!(!blue_green.supports_gradual_rollout());
        assert!(blue_green.supports_instant_rollback());
    }

    #[test]
    fn test_batch_size_from_percentage() {
        // Test 10% of 100 instances = 10 instances
        let batch_size = BatchSize::from_percentage(10, 100).unwrap();
        assert_eq!(batch_size.as_u8(), 10);

        // Test 50% of 3 instances = 2 instances (rounded up from 1.5)
        let batch_size = BatchSize::from_percentage(50, 3).unwrap();
        assert_eq!(batch_size.as_u8(), 2);

        // Test minimum batch size of 1
        let batch_size = BatchSize::from_percentage(1, 200).unwrap();
        assert_eq!(batch_size.as_u8(), 2);
    }

    #[test]
    fn test_deployment_request_validation() {
        let agent_id = AgentId::generate();
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::default();

        // Valid request
        let request = DeploymentRequest::new(
            agent_id,
            None,
            None,
            version,
            version_number,
            config.clone(),
            vec![1, 2, 3, 4], // Non-empty WASM
        );
        assert!(request.validate().is_ok());

        // Empty WASM module
        let empty_request = DeploymentRequest::new(
            agent_id,
            None,
            None,
            version,
            version_number,
            config.clone(),
            vec![], // Empty WASM
        );
        assert!(matches!(
            empty_request.validate(),
            Err(DeploymentValidationError::EmptyWasmModule)
        ));
    }

    #[test]
    fn test_deployment_progress() {
        let progress = DeploymentProgress::from_percentage(50).unwrap();
        assert_eq!(progress.as_percentage(), 50);
        assert!(!progress.is_complete());

        let completed = DeploymentProgress::completed();
        assert_eq!(completed.as_percentage(), 100);
        assert!(completed.is_complete());
    }

    #[test]
    fn test_resource_requirements_compatibility() {
        let requirements = ResourceRequirements::minimal();

        // Compatible system
        assert!(requirements.is_compatible_with(10_000_000, 100_000));

        // Incompatible system (insufficient memory)
        assert!(!requirements.is_compatible_with(100, 100_000));

        // Incompatible system (insufficient fuel)
        assert!(!requirements.is_compatible_with(10_000_000, 100));
    }
}
