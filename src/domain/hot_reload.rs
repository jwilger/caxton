//! Hot Reload domain types
//!
//! This module defines types for hot reloading agent WASM modules without
//! disrupting service, including rollback capabilities and version management.

use nutype::nutype;
use serde::{Deserialize, Serialize};
// use std::collections::HashMap; // Unused for now
use std::time::{Duration, SystemTime};
use thiserror::Error;
use uuid::Uuid;

use super::agent_lifecycle::{AgentVersion, VersionNumber};
use super::deployment::{DeploymentId, DeploymentTimeout, ResourceRequirements};
use crate::domain_types::{AgentId, AgentName};

/// Unique identifier for a hot reload operation
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
pub struct HotReloadId(Uuid);

impl HotReloadId {
    /// Creates a new random hot reload ID
    pub fn generate() -> Self {
        Self::new(Uuid::new_v4())
    }
}

/// Hot reload strategy for updating agent code
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HotReloadStrategy {
    /// Graceful reload with request draining
    Graceful,
    /// Immediate reload with connection termination
    Immediate,
    /// Parallel reload keeping old version running
    Parallel,
    /// Traffic splitting between versions
    TrafficSplitting,
}

impl HotReloadStrategy {
    /// Check if strategy requires draining existing requests
    pub fn requires_draining(&self) -> bool {
        matches!(self, Self::Graceful)
    }

    /// Check if strategy allows rollback without downtime
    pub fn supports_zero_downtime_rollback(&self) -> bool {
        matches!(self, Self::Parallel | Self::TrafficSplitting)
    }

    /// Check if strategy can run multiple versions simultaneously
    pub fn supports_multiple_versions(&self) -> bool {
        matches!(self, Self::Parallel | Self::TrafficSplitting)
    }
}

/// Traffic split percentage for parallel deployments
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
    default = 10 // Start with 10% traffic to new version
)]
pub struct TrafficSplitPercentage(u8);

impl TrafficSplitPercentage {
    /// Gets the value as percentage (0-100)
    pub fn as_percentage(&self) -> u8 {
        self.into_inner()
    }

    /// Creates percentage for A/B testing (50/50)
    ///
    /// # Panics
    ///
    /// Panics if 50 is somehow invalid (should never happen).
    pub fn half() -> Self {
        Self::try_new(50).unwrap()
    }

    /// Creates percentage for full traffic to new version
    ///
    /// # Panics
    ///
    /// Panics if 100 is somehow invalid (should never happen).
    pub fn full() -> Self {
        Self::try_new(100).unwrap()
    }

    /// Creates percentage for canary deployment (5%)
    ///
    /// # Panics
    ///
    /// Panics if 5 is somehow invalid (should never happen).
    pub fn canary() -> Self {
        Self::try_new(5).unwrap()
    }

    /// Increment traffic percentage by specified amount
    ///
    /// # Errors
    ///
    /// Returns `TrafficSplitPercentageError` if the new percentage would be invalid.
    pub fn increment_by(&self, amount: u8) -> Result<Self, TrafficSplitPercentageError> {
        let new_value = self.into_inner().saturating_add(amount).min(100);
        Self::try_new(new_value)
    }

    /// Get remaining traffic percentage for old version
    pub fn remaining_percentage(&self) -> u8 {
        100 - self.into_inner()
    }
}

/// Rollback capability configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RollbackCapability {
    pub enabled: bool,
    pub automatic_triggers: Vec<RollbackTrigger>,
    pub preserve_previous_versions: u8,
    pub rollback_timeout: DeploymentTimeout,
    pub health_check_enabled: bool,
}

impl RollbackCapability {
    /// Creates new rollback capability with defaults
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded timeout value is invalid (should never happen).
    pub fn new() -> Self {
        Self {
            enabled: true,
            automatic_triggers: vec![
                RollbackTrigger::HealthCheckFailure,
                RollbackTrigger::ErrorRateThreshold(5.0),
                RollbackTrigger::PerformanceDegradation(50.0),
            ],
            preserve_previous_versions: 3,
            rollback_timeout: DeploymentTimeout::try_new(60_000).unwrap(), // 1 minute
            health_check_enabled: true,
        }
    }

    /// Creates minimal rollback capability (manual only)
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded timeout value is invalid (should never happen).
    pub fn manual_only() -> Self {
        Self {
            enabled: true,
            automatic_triggers: vec![],
            preserve_previous_versions: 1,
            rollback_timeout: DeploymentTimeout::try_new(30_000).unwrap(), // 30 seconds
            health_check_enabled: false,
        }
    }

    /// Disables all rollback capabilities
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded timeout value is invalid (should never happen).
    pub fn disabled() -> Self {
        Self {
            enabled: false,
            automatic_triggers: vec![],
            preserve_previous_versions: 0,
            rollback_timeout: DeploymentTimeout::try_new(30_000).unwrap(),
            health_check_enabled: false,
        }
    }

    /// Check if automatic rollback should trigger based on metrics
    pub fn should_trigger_rollback(&self, metrics: &ReloadMetrics) -> bool {
        if !self.enabled {
            return false;
        }

        self.automatic_triggers
            .iter()
            .any(|trigger| trigger.should_trigger(metrics))
    }
}

impl Default for RollbackCapability {
    fn default() -> Self {
        Self::new()
    }
}

/// Automatic rollback triggers
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RollbackTrigger {
    /// Health check failures exceed threshold
    HealthCheckFailure,
    /// Error rate exceeds percentage threshold
    ErrorRateThreshold(f32),
    /// Performance degrades by percentage
    PerformanceDegradation(f32),
    /// Memory usage exceeds threshold
    MemoryThreshold(usize),
    /// Custom metric threshold
    CustomMetric { name: String, threshold: f32 },
}

impl RollbackTrigger {
    /// Check if trigger condition is met based on metrics
    pub fn should_trigger(&self, metrics: &ReloadMetrics) -> bool {
        match self {
            Self::HealthCheckFailure => {
                metrics.health_check_success_rate < 50.0 // Less than 50% success rate
            }
            Self::ErrorRateThreshold(threshold) => metrics.error_rate_percentage > *threshold,
            Self::PerformanceDegradation(threshold) => {
                metrics.performance_degradation_percentage > *threshold
            }
            Self::MemoryThreshold(threshold) => metrics.memory_usage_peak > *threshold,
            Self::CustomMetric {
                name: _,
                threshold: _,
            } => {
                // Custom metrics would be evaluated against external systems
                false
            }
        }
    }
}

/// Hot reload configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotReloadConfig {
    pub strategy: HotReloadStrategy,
    pub traffic_split: TrafficSplitPercentage,
    pub drain_timeout: DeploymentTimeout,
    pub warmup_duration: Duration,
    pub rollback_capability: RollbackCapability,
    pub resource_requirements: ResourceRequirements,
    pub enable_metrics_collection: bool,
    pub progressive_rollout: bool,
}

impl HotReloadConfig {
    /// Creates new hot reload configuration
    /// Creates a new hot reload configuration with the given strategy
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded drain timeout value is invalid (should never happen).
    pub fn new(strategy: HotReloadStrategy) -> Self {
        Self {
            strategy,
            traffic_split: TrafficSplitPercentage::default(),
            drain_timeout: DeploymentTimeout::try_new(60_000).unwrap(), // 1 minute
            warmup_duration: Duration::from_secs(10),
            rollback_capability: RollbackCapability::default(),
            resource_requirements: ResourceRequirements::default(),
            enable_metrics_collection: true,
            progressive_rollout: true,
        }
    }

    /// Creates configuration for graceful hot reload
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded drain timeout value is invalid (should never happen).
    pub fn graceful() -> Self {
        let mut config = Self::new(HotReloadStrategy::Graceful);
        config.drain_timeout = DeploymentTimeout::try_new(120_000).unwrap(); // 2 minutes
        config
    }

    /// Creates configuration for immediate hot reload
    ///
    /// # Panics
    ///
    /// Panics if the hardcoded drain timeout value is invalid (should never happen).
    pub fn immediate() -> Self {
        let mut config = Self::new(HotReloadStrategy::Immediate);
        config.drain_timeout = DeploymentTimeout::try_new(30_000).unwrap(); // 30 seconds (minimum allowed)
        config.rollback_capability = RollbackCapability::manual_only();
        config
    }

    /// Creates configuration for traffic splitting
    pub fn traffic_splitting(split_percentage: TrafficSplitPercentage) -> Self {
        let mut config = Self::new(HotReloadStrategy::TrafficSplitting);
        config.traffic_split = split_percentage;
        config.progressive_rollout = true;
        config
    }

    /// Creates configuration for parallel deployment
    pub fn parallel() -> Self {
        let mut config = Self::new(HotReloadStrategy::Parallel);
        config.resource_requirements.memory_limit = config
            .resource_requirements
            .memory_limit
            .as_bytes()
            .saturating_mul(2) // Double memory for parallel execution
            .try_into()
            .unwrap_or(config.resource_requirements.memory_limit);
        config
    }
}

impl Default for HotReloadConfig {
    fn default() -> Self {
        Self::new(HotReloadStrategy::Graceful)
    }
}

/// Hot reload request
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotReloadRequest {
    pub reload_id: HotReloadId,
    pub deployment_id: Option<DeploymentId>,
    pub agent_id: AgentId,
    pub agent_name: Option<AgentName>,
    pub from_version: AgentVersion,
    pub to_version: AgentVersion,
    pub to_version_number: VersionNumber,
    pub config: HotReloadConfig,
    pub new_wasm_module: Vec<u8>,
    pub preserve_state: bool,
    pub requested_at: SystemTime,
}

impl HotReloadRequest {
    /// Creates a new hot reload request
    pub fn new(
        agent_id: AgentId,
        agent_name: Option<AgentName>,
        from_version: AgentVersion,
        to_version: AgentVersion,
        to_version_number: VersionNumber,
        config: HotReloadConfig,
        new_wasm_module: Vec<u8>,
    ) -> Self {
        Self {
            reload_id: HotReloadId::generate(),
            deployment_id: None,
            agent_id,
            agent_name,
            from_version,
            to_version,
            to_version_number,
            config,
            new_wasm_module,
            preserve_state: true,
            requested_at: SystemTime::now(),
        }
    }

    /// Associate with a deployment
    #[must_use]
    pub fn with_deployment(mut self, deployment_id: DeploymentId) -> Self {
        self.deployment_id = Some(deployment_id);
        self
    }

    /// Get WASM module size
    pub fn module_size(&self) -> usize {
        self.new_wasm_module.len()
    }

    /// Validate the hot reload request
    ///
    /// # Errors
    ///
    /// Returns `HotReloadValidationError` if the request is invalid.
    pub fn validate(&self) -> Result<(), HotReloadValidationError> {
        if self.new_wasm_module.is_empty() {
            return Err(HotReloadValidationError::EmptyWasmModule);
        }

        if self.new_wasm_module.len() > 100 * 1024 * 1024 {
            // 100MB max
            return Err(HotReloadValidationError::WasmModuleTooLarge {
                size: self.new_wasm_module.len(),
                max: 100 * 1024 * 1024,
            });
        }

        if self.from_version == self.to_version {
            return Err(HotReloadValidationError::SameVersion);
        }

        if self.config.strategy.supports_multiple_versions()
            && !self.config.resource_requirements.requires_isolation
        {
            return Err(HotReloadValidationError::IsolationRequired);
        }

        Ok(())
    }
}

/// Hot reload status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum HotReloadStatus {
    /// Hot reload requested but not started
    Pending,
    /// Preparing new version (validation, compilation)
    Preparing,
    /// Starting traffic split or parallel execution
    Starting,
    /// Hot reload in progress with metrics collection
    InProgress,
    /// Hot reload completed successfully
    Completed,
    /// Hot reload failed
    Failed,
    /// Hot reload cancelled by user
    Cancelled,
    /// Automatically rolled back due to issues
    RolledBack,
    /// Manual rollback initiated
    RollingBack,
}

impl HotReloadStatus {
    /// Check if status is terminal
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

    /// Check if rollback is possible
    pub fn can_rollback(&self) -> bool {
        matches!(self, Self::InProgress | Self::Completed | Self::Failed)
    }
}

/// Hot reload metrics for monitoring and rollback decisions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReloadMetrics {
    pub requests_processed: u64,
    pub requests_failed: u64,
    pub error_rate_percentage: f32,
    pub average_response_time_ms: f64,
    pub performance_degradation_percentage: f32,
    pub memory_usage_peak: usize,
    pub memory_usage_average: usize,
    pub health_check_success_rate: f32,
    pub traffic_split_actual: TrafficSplitPercentage,
    pub collected_at: SystemTime,
}

impl ReloadMetrics {
    /// Creates new metrics instance
    pub fn new() -> Self {
        Self {
            requests_processed: 0,
            requests_failed: 0,
            error_rate_percentage: 0.0,
            average_response_time_ms: 0.0,
            performance_degradation_percentage: 0.0,
            memory_usage_peak: 0,
            memory_usage_average: 0,
            health_check_success_rate: 100.0,
            traffic_split_actual: TrafficSplitPercentage::default(),
            collected_at: SystemTime::now(),
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f32 {
        if self.requests_processed == 0 {
            return 100.0;
        }
        let success = self.requests_processed - self.requests_failed;
        #[allow(clippy::cast_precision_loss)]
        {
            (success as f32 / self.requests_processed as f32) * 100.0
        }
    }

    /// Check if metrics indicate healthy operation
    pub fn is_healthy(&self) -> bool {
        self.error_rate_percentage < 5.0
            && self.health_check_success_rate > 90.0
            && self.performance_degradation_percentage < 20.0
    }

    /// Update metrics with new request data
    pub fn update(&mut self, success: bool, response_time_ms: f64, memory_usage: usize) {
        self.requests_processed += 1;
        if !success {
            self.requests_failed += 1;
        }

        self.error_rate_percentage = {
            #[allow(clippy::cast_precision_loss)]
            let result = (self.requests_failed as f32 / self.requests_processed as f32) * 100.0;
            result
        };

        // Update average response time (simple moving average approximation)
        #[allow(clippy::cast_precision_loss)]
        let total_requests = self.requests_processed as f64;
        self.average_response_time_ms = ((self.average_response_time_ms * (total_requests - 1.0))
            + response_time_ms)
            / total_requests;

        self.memory_usage_peak = self.memory_usage_peak.max(memory_usage);
        {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let total_as_u64 = total_requests as u64;
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let total_as_usize = total_requests as usize;
            #[allow(clippy::cast_possible_truncation)]
            {
                self.memory_usage_average = ((self.memory_usage_average as u64
                    * (total_as_u64 - 1))
                    + memory_usage as u64) as usize
                    / total_as_usize;
            }
        }

        self.collected_at = SystemTime::now();
    }
}

impl Default for ReloadMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Version tracking for rollback capabilities
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionSnapshot {
    pub version: AgentVersion,
    pub version_number: VersionNumber,
    pub wasm_module: Vec<u8>,
    pub created_at: SystemTime,
    pub resource_usage: ResourceUsageSnapshot,
}

/// Resource usage snapshot for version comparison
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResourceUsageSnapshot {
    pub memory_allocated: usize,
    pub fuel_consumed: u64,
    pub requests_handled: u64,
    pub average_response_time_ms: u64,
}

/// Hot reload result
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HotReloadResult {
    pub reload_id: HotReloadId,
    pub agent_id: AgentId,
    pub status: HotReloadStatus,
    pub from_version: AgentVersion,
    pub to_version: AgentVersion,
    pub started_at: Option<SystemTime>,
    pub completed_at: Option<SystemTime>,
    pub error_message: Option<String>,
    pub rollback_reason: Option<String>,
    pub metrics: Option<ReloadMetrics>,
    pub preserved_versions: Vec<VersionSnapshot>,
}

impl HotReloadResult {
    /// Creates successful hot reload result
    pub fn success(
        reload_id: HotReloadId,
        agent_id: AgentId,
        from_version: AgentVersion,
        to_version: AgentVersion,
        started_at: SystemTime,
        metrics: Option<ReloadMetrics>,
        preserved_versions: Vec<VersionSnapshot>,
    ) -> Self {
        Self {
            reload_id,
            agent_id,
            status: HotReloadStatus::Completed,
            from_version,
            to_version,
            started_at: Some(started_at),
            completed_at: Some(SystemTime::now()),
            error_message: None,
            rollback_reason: None,
            metrics,
            preserved_versions,
        }
    }

    /// Creates failed hot reload result
    pub fn failure(
        reload_id: HotReloadId,
        agent_id: AgentId,
        from_version: AgentVersion,
        to_version: AgentVersion,
        started_at: Option<SystemTime>,
        error_message: String,
    ) -> Self {
        Self {
            reload_id,
            agent_id,
            status: HotReloadStatus::Failed,
            from_version,
            to_version,
            started_at,
            completed_at: Some(SystemTime::now()),
            error_message: Some(error_message),
            rollback_reason: None,
            metrics: None,
            preserved_versions: vec![],
        }
    }

    /// Creates rolled back hot reload result
    pub fn rollback(
        reload_id: HotReloadId,
        agent_id: AgentId,
        from_version: AgentVersion,
        to_version: AgentVersion,
        started_at: Option<SystemTime>,
        rollback_reason: String,
        metrics: Option<ReloadMetrics>,
    ) -> Self {
        Self {
            reload_id,
            agent_id,
            status: HotReloadStatus::RolledBack,
            from_version: to_version, // Swapped because we rolled back
            to_version: from_version, // Swapped because we rolled back
            started_at,
            completed_at: Some(SystemTime::now()),
            error_message: None,
            rollback_reason: Some(rollback_reason),
            metrics,
            preserved_versions: vec![],
        }
    }

    /// Get hot reload duration
    pub fn duration(&self) -> Option<Duration> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        }
    }
}

/// Hot reload validation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum HotReloadValidationError {
    #[error("WASM module is empty")]
    EmptyWasmModule,

    #[error("WASM module too large: {size} bytes, max {max} bytes")]
    WasmModuleTooLarge { size: usize, max: usize },

    #[error("Source and target versions are the same")]
    SameVersion,

    #[error("Resource isolation required for multi-version strategy")]
    IsolationRequired,

    #[error("Invalid traffic split configuration")]
    InvalidTrafficSplit,

    #[error("Rollback capability required for strategy")]
    RollbackRequired,
}

/// Hot reload operation errors
#[derive(Debug, Clone, Error, PartialEq, Eq)]
pub enum HotReloadError {
    #[error("Hot reload validation failed: {0}")]
    ValidationFailed(#[from] HotReloadValidationError),

    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Hot reload already in progress: {reload_id}")]
    AlreadyInProgress { reload_id: HotReloadId },

    #[error("Insufficient resources for parallel execution")]
    InsufficientResources,

    #[error("State preservation failed: {reason}")]
    StatePreservationFailed { reason: String },

    #[error("Traffic splitting failed: {reason}")]
    TrafficSplittingFailed { reason: String },

    #[error("Automatic rollback triggered: {reason}")]
    AutomaticRollback { reason: String },

    #[error("Manual rollback failed: {reason}")]
    RollbackFailed { reason: String },

    #[error("Version not found for rollback: {version}")]
    VersionNotFound { version: AgentVersion },

    #[error("Hot reload timeout exceeded: {timeout}ms")]
    TimeoutExceeded { timeout: u64 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hot_reload_strategy() {
        let graceful = HotReloadStrategy::Graceful;
        assert!(graceful.requires_draining());
        assert!(!graceful.supports_zero_downtime_rollback());
        assert!(!graceful.supports_multiple_versions());

        let parallel = HotReloadStrategy::Parallel;
        assert!(!parallel.requires_draining());
        assert!(parallel.supports_zero_downtime_rollback());
        assert!(parallel.supports_multiple_versions());
    }

    #[test]
    fn test_traffic_split_percentage() {
        let split = TrafficSplitPercentage::default();
        assert_eq!(split.as_percentage(), 10);
        assert_eq!(split.remaining_percentage(), 90);

        let half = TrafficSplitPercentage::half();
        assert_eq!(half.as_percentage(), 50);
        assert_eq!(half.remaining_percentage(), 50);

        let incremented = split.increment_by(20).unwrap();
        assert_eq!(incremented.as_percentage(), 30);
    }

    #[test]
    fn test_rollback_triggers() {
        let mut metrics = ReloadMetrics::new();
        metrics.error_rate_percentage = 10.0;
        metrics.health_check_success_rate = 30.0;

        let error_trigger = RollbackTrigger::ErrorRateThreshold(5.0);
        assert!(error_trigger.should_trigger(&metrics));

        let health_trigger = RollbackTrigger::HealthCheckFailure;
        assert!(health_trigger.should_trigger(&metrics));

        let perf_trigger = RollbackTrigger::PerformanceDegradation(20.0);
        assert!(!perf_trigger.should_trigger(&metrics)); // No degradation set
    }

    #[test]
    fn test_reload_metrics_update() {
        let mut metrics = ReloadMetrics::new();

        // Process successful request
        metrics.update(true, 100.0, 1024);
        assert_eq!(metrics.requests_processed, 1);
        assert_eq!(metrics.requests_failed, 0);
        assert!((metrics.error_rate_percentage - 0.0).abs() < f32::EPSILON);
        assert!((metrics.average_response_time_ms - 100.0).abs() < f64::EPSILON);

        // Process failed request
        metrics.update(false, 200.0, 2048);
        assert_eq!(metrics.requests_processed, 2);
        assert_eq!(metrics.requests_failed, 1);
        assert!((metrics.error_rate_percentage - 50.0).abs() < f32::EPSILON);
        assert!((metrics.average_response_time_ms - 150.0).abs() < f64::EPSILON);
        assert_eq!(metrics.memory_usage_peak, 2048);
    }

    #[test]
    fn test_hot_reload_request_validation() {
        let agent_id = AgentId::generate();
        let from_version = AgentVersion::generate();
        let to_version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = HotReloadConfig::default();

        // Valid request
        let request = HotReloadRequest::new(
            agent_id,
            None,
            from_version,
            to_version,
            version_number,
            config.clone(),
            vec![1, 2, 3, 4],
        );
        assert!(request.validate().is_ok());

        // Same version error
        let same_version_request = HotReloadRequest::new(
            agent_id,
            None,
            from_version,
            from_version, // Same as from_version
            version_number,
            config.clone(),
            vec![1, 2, 3, 4],
        );
        assert!(matches!(
            same_version_request.validate(),
            Err(HotReloadValidationError::SameVersion)
        ));
    }
}
