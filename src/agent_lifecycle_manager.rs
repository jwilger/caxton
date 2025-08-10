//! Agent Lifecycle Manager
//!
//! This module provides the core orchestration layer for agent lifecycle management,
//! coordinating deployment, hot reload, and WASM validation operations with
//! comprehensive state tracking and error handling.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::domain::{
    AgentFailureReason, AgentLifecycle, AgentLifecycleState, AgentVersion, DeploymentConfig,
    DeploymentError, DeploymentId, DeploymentRequest, DeploymentResult, DeploymentStatus,
    HotReloadConfig, HotReloadError, HotReloadId, HotReloadRequest, HotReloadResult,
    HotReloadStatus, StateTransitionError, ValidationResult, VersionNumber, WasmModule,
};
use crate::domain_types::{AgentId, AgentName, MemoryBytes};

/// Errors that can occur during agent lifecycle management
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum LifecycleError {
    #[error("Agent not found: {agent_id}")]
    AgentNotFound { agent_id: AgentId },

    #[error("Operation timeout after {timeout_ms}ms")]
    OperationTimeout { timeout_ms: u64 },

    #[error("Invalid state transition: {0}")]
    InvalidStateTransition(#[from] StateTransitionError),

    #[error("Deployment error: {0}")]
    DeploymentError(#[from] DeploymentError),

    #[error("Hot reload error: {0}")]
    HotReloadError(#[from] HotReloadError),

    #[error("Resource allocation failed: {reason}")]
    ResourceAllocationFailed { reason: String },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

/// Result type for lifecycle operations
pub type Result<T> = std::result::Result<T, LifecycleError>;

/// Agent lifecycle status with comprehensive metadata
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct AgentStatus {
    pub lifecycle: AgentLifecycle,
    pub deployment_id: Option<DeploymentId>,
    pub hot_reload_id: Option<HotReloadId>,
    pub memory_allocated: MemoryBytes,
    pub uptime: Duration,
    pub last_activity: SystemTime,
    pub health_status: HealthStatus,
}

/// Health status tracking for agents
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String },
    Unknown,
}

/// Lifecycle operation result with timing information
#[derive(Debug, Clone)]
#[allow(missing_docs)]
pub struct OperationResult {
    pub success: bool,
    pub operation_duration: Duration,
    pub error_message: Option<String>,
    pub started_at: SystemTime,
    pub completed_at: SystemTime,
}

impl OperationResult {
    /// Creates a successful operation result
    pub fn success(started_at: SystemTime) -> Self {
        let completed_at = SystemTime::now();
        Self {
            success: true,
            operation_duration: completed_at.duration_since(started_at).unwrap_or_default(),
            error_message: None,
            started_at,
            completed_at,
        }
    }

    /// Creates a failed operation result
    pub fn failure(started_at: SystemTime, error: String) -> Self {
        let completed_at = SystemTime::now();
        Self {
            success: false,
            operation_duration: completed_at.duration_since(started_at).unwrap_or_default(),
            error_message: Some(error),
            started_at,
            completed_at,
        }
    }
}

/// Core Agent Lifecycle Manager
///
/// Orchestrates the complete lifecycle of agents from deployment through termination,
/// providing coordination between deployment, hot reload, and validation subsystems.
pub struct AgentLifecycleManager {
    /// Active agent lifecycles indexed by agent ID
    agents: Arc<RwLock<HashMap<AgentId, AgentLifecycle>>>,
    /// Agent status tracking
    agent_status: Arc<RwLock<HashMap<AgentId, AgentStatus>>>,
    /// Active deployment operations
    active_deployments: Arc<Mutex<HashMap<DeploymentId, DeploymentRequest>>>,
    /// Active hot reload operations
    active_hot_reloads: Arc<Mutex<HashMap<HotReloadId, HotReloadRequest>>>,
    /// Deployment manager for handling deployments
    deployment_manager: Arc<dyn DeploymentManager + Send + Sync>,
    /// Hot reload manager for live updates
    hot_reload_manager: Arc<dyn HotReloadManager + Send + Sync>,
    /// WASM module validator
    module_validator: Arc<dyn WasmModuleValidator + Send + Sync>,
    /// Default timeout for operations
    default_timeout: Duration,
}

/// Trait for deployment management operations
#[async_trait::async_trait]
pub trait DeploymentManager {
    /// Deploy a new agent or update existing one
    async fn deploy_agent(
        &self,
        request: DeploymentRequest,
    ) -> std::result::Result<DeploymentResult, DeploymentError>;

    /// Get deployment status
    async fn get_deployment_status(
        &self,
        deployment_id: DeploymentId,
    ) -> std::result::Result<DeploymentStatus, DeploymentError>;

    /// Cancel active deployment
    async fn cancel_deployment(
        &self,
        deployment_id: DeploymentId,
    ) -> std::result::Result<(), DeploymentError>;

    /// Rollback deployment to previous version
    async fn rollback_deployment(
        &self,
        deployment_id: DeploymentId,
        target_version: AgentVersion,
    ) -> std::result::Result<DeploymentResult, DeploymentError>;

    /// Clean up deployed agent resources
    async fn cleanup_agent(&self, agent_id: AgentId) -> std::result::Result<(), DeploymentError>;
}

/// Trait for hot reload management operations
#[async_trait::async_trait]
pub trait HotReloadManager {
    /// Perform hot reload of agent code
    async fn hot_reload_agent(
        &self,
        request: HotReloadRequest,
    ) -> std::result::Result<HotReloadResult, HotReloadError>;

    /// Get hot reload status
    async fn get_hot_reload_status(
        &self,
        reload_id: HotReloadId,
    ) -> std::result::Result<HotReloadStatus, HotReloadError>;

    /// Cancel active hot reload
    async fn cancel_hot_reload(
        &self,
        reload_id: HotReloadId,
    ) -> std::result::Result<(), HotReloadError>;

    /// Rollback hot reload to previous version
    async fn rollback_hot_reload(
        &self,
        reload_id: HotReloadId,
        target_version: AgentVersion,
    ) -> std::result::Result<HotReloadResult, HotReloadError>;
}

/// Trait for WASM module validation
#[async_trait::async_trait]
pub trait WasmModuleValidator {
    /// Validate WASM module before deployment
    async fn validate_module(
        &self,
        wasm_bytes: &[u8],
        agent_name: Option<AgentName>,
    ) -> std::result::Result<WasmModule, crate::domain::WasmValidationError>;

    /// Perform security validation
    async fn validate_security(
        &self,
        module: &WasmModule,
    ) -> std::result::Result<ValidationResult, crate::domain::WasmValidationError>;

    /// Extract module metadata
    async fn extract_metadata(
        &self,
        wasm_bytes: &[u8],
    ) -> std::result::Result<HashMap<String, String>, crate::domain::WasmValidationError>;
}

impl AgentLifecycleManager {
    /// Creates a new Agent Lifecycle Manager
    pub fn new(
        deployment_manager: Arc<dyn DeploymentManager + Send + Sync>,
        hot_reload_manager: Arc<dyn HotReloadManager + Send + Sync>,
        module_validator: Arc<dyn WasmModuleValidator + Send + Sync>,
    ) -> Self {
        Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            agent_status: Arc::new(RwLock::new(HashMap::new())),
            active_deployments: Arc::new(Mutex::new(HashMap::new())),
            active_hot_reloads: Arc::new(Mutex::new(HashMap::new())),
            deployment_manager,
            hot_reload_manager,
            module_validator,
            default_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Creates a new Agent Lifecycle Manager with custom timeout
    pub fn with_timeout(
        deployment_manager: Arc<dyn DeploymentManager + Send + Sync>,
        hot_reload_manager: Arc<dyn HotReloadManager + Send + Sync>,
        module_validator: Arc<dyn WasmModuleValidator + Send + Sync>,
        timeout: Duration,
    ) -> Self {
        let mut manager = Self::new(deployment_manager, hot_reload_manager, module_validator);
        manager.default_timeout = timeout;
        manager
    }

    /// Deploy a new agent with comprehensive validation and state management
    ///
    /// # Errors
    ///
    /// Returns `DeploymentError` if:
    /// - Agent validation fails
    /// - Resource allocation fails
    /// - WASM module is invalid
    /// - Agent instance creation fails
    ///
    /// # Panics
    ///
    /// May panic if internal failure reason creation fails (should not happen in normal operation).
    #[tracing::instrument(skip(self, wasm_module_bytes))]
    #[allow(clippy::too_many_lines)]
    pub async fn deploy_agent(
        &self,
        agent_id: AgentId,
        agent_name: Option<AgentName>,
        version: AgentVersion,
        version_number: VersionNumber,
        config: DeploymentConfig,
        wasm_module_bytes: Vec<u8>,
    ) -> Result<DeploymentResult> {
        let start_time = SystemTime::now();
        info!("Starting agent deployment for agent_id: {}", agent_id);

        // Validate WASM module first
        let validated_module = timeout(
            self.default_timeout,
            self.module_validator
                .validate_module(&wasm_module_bytes, agent_name.clone()),
        )
        .await
        .map_err(|_| LifecycleError::OperationTimeout {
            timeout_ms: u64::try_from(self.default_timeout.as_millis()).unwrap_or(u64::MAX),
        })?
        .map_err(|e| LifecycleError::ValidationFailed {
            reason: e.to_string(),
        })?;

        if !validated_module.is_valid() {
            let reasons = validated_module
                .validation_result
                .error_messages()
                .join(", ");
            return Err(LifecycleError::ValidationFailed { reason: reasons });
        }

        // Create agent lifecycle state
        let mut lifecycle =
            AgentLifecycle::new(agent_id, agent_name.clone(), version, version_number);

        // Transition to loaded state
        lifecycle.transition_to(AgentLifecycleState::Loaded, None)?;

        // Store initial lifecycle state
        {
            let mut agents = self.agents.write().await;
            agents.insert(agent_id, lifecycle.clone());
        }

        // Create deployment request
        let deployment_request = DeploymentRequest::new(
            agent_id,
            agent_name,
            None, // Initial deployment
            version,
            version_number,
            config,
            wasm_module_bytes,
        );

        // Validate deployment request
        deployment_request
            .validate()
            .map_err(DeploymentError::ValidationFailed)?;

        let deployment_id = deployment_request.deployment_id;

        // Track active deployment
        {
            let mut active = self.active_deployments.lock().await;
            active.insert(deployment_id, deployment_request.clone());
        }

        // Execute deployment
        let deployment_result = timeout(
            self.default_timeout,
            self.deployment_manager.deploy_agent(deployment_request),
        )
        .await
        .map_err(|_| LifecycleError::OperationTimeout {
            timeout_ms: u64::try_from(self.default_timeout.as_millis()).unwrap_or(u64::MAX),
        })?;

        // Clean up active deployment tracking
        {
            let mut active = self.active_deployments.lock().await;
            active.remove(&deployment_id);
        }

        match deployment_result {
            Ok(result) => {
                // Update lifecycle state based on deployment result
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent_lifecycle) = agents.get_mut(&agent_id) {
                        if result.status.is_success() {
                            if let Err(e) =
                                agent_lifecycle.transition_to(AgentLifecycleState::Ready, None)
                            {
                                warn!("Failed to transition agent to Ready state: {}", e);
                            }
                        } else {
                            let failure_reason =
                                AgentFailureReason::from_error(&DeploymentError::ValidationFailed(
                                    crate::domain::DeploymentValidationError::InvalidStrategy,
                                ))
                                .unwrap_or_else(|_| {
                                    AgentFailureReason::try_new("Deployment failed".to_string())
                                        .unwrap()
                                });
                            if let Err(e) = agent_lifecycle.fail(failure_reason) {
                                warn!("Failed to mark agent as failed: {}", e);
                            }
                        }
                    }
                }

                // Update agent status
                self.update_agent_status(agent_id, Some(deployment_id), None)
                    .await;

                info!(
                    "Agent deployment completed successfully in {:?}",
                    start_time.elapsed().unwrap_or_default()
                );
                Ok(result)
            }
            Err(e) => {
                // Mark agent as failed
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent_lifecycle) = agents.get_mut(&agent_id) {
                        let failure_reason =
                            AgentFailureReason::from_error(&e).unwrap_or_else(|_| {
                                AgentFailureReason::try_new(e.to_string()).unwrap_or_else(|_| {
                                    AgentFailureReason::try_new(
                                        "Unknown deployment failure".to_string(),
                                    )
                                    .unwrap()
                                })
                            });
                        if let Err(transition_err) = agent_lifecycle.fail(failure_reason) {
                            error!("Failed to mark agent as failed: {}", transition_err);
                        }
                    }
                }

                error!("Agent deployment failed: {}", e);
                Err(LifecycleError::DeploymentError(e))
            }
        }
    }

    /// Perform hot reload of an existing agent
    ///
    /// # Errors
    ///
    /// Returns `HotReloadError` if:
    /// - Agent validation fails
    /// - Resource allocation fails
    /// - WASM module is invalid
    /// - Hot reload strategy fails
    ///
    /// # Panics
    ///
    /// May panic if internal failure reason creation fails (should not happen in normal operation).
    #[tracing::instrument(skip(self, wasm_module_bytes))]
    #[allow(clippy::too_many_lines)]
    pub async fn hot_reload_agent(
        &self,
        agent_id: AgentId,
        from_version: AgentVersion,
        to_version: AgentVersion,
        to_version_number: VersionNumber,
        config: HotReloadConfig,
        wasm_module_bytes: Vec<u8>,
    ) -> Result<HotReloadResult> {
        let start_time = SystemTime::now();
        info!(
            "Starting hot reload for agent_id: {} from {} to {}",
            agent_id, from_version, to_version
        );

        // Validate agent exists and is in correct state
        let agent_name = {
            let agents = self.agents.read().await;
            let agent = agents
                .get(&agent_id)
                .ok_or(LifecycleError::AgentNotFound { agent_id })?;

            if !agent.current_state.can_start()
                && agent.current_state != AgentLifecycleState::Running
            {
                return Err(LifecycleError::InvalidStateTransition(
                    StateTransitionError::InvalidTransition {
                        from: agent.current_state,
                        to: AgentLifecycleState::Running,
                    },
                ));
            }

            agent.agent_name.clone()
        };

        // Validate new WASM module
        let validated_module = timeout(
            self.default_timeout,
            self.module_validator
                .validate_module(&wasm_module_bytes, agent_name.clone()),
        )
        .await
        .map_err(|_| LifecycleError::OperationTimeout {
            timeout_ms: u64::try_from(self.default_timeout.as_millis()).unwrap_or(u64::MAX),
        })?
        .map_err(|e| LifecycleError::ValidationFailed {
            reason: e.to_string(),
        })?;

        if !validated_module.is_valid() {
            let reasons = validated_module
                .validation_result
                .error_messages()
                .join(", ");
            return Err(LifecycleError::ValidationFailed { reason: reasons });
        }

        // Create hot reload request
        let hot_reload_request = HotReloadRequest::new(
            agent_id,
            agent_name,
            from_version,
            to_version,
            to_version_number,
            config,
            wasm_module_bytes,
        );

        // Validate hot reload request
        hot_reload_request
            .validate()
            .map_err(HotReloadError::ValidationFailed)?;

        let reload_id = hot_reload_request.reload_id;

        // Track active hot reload
        {
            let mut active = self.active_hot_reloads.lock().await;
            active.insert(reload_id, hot_reload_request.clone());
        }

        // Execute hot reload
        let hot_reload_result = timeout(
            self.default_timeout,
            self.hot_reload_manager.hot_reload_agent(hot_reload_request),
        )
        .await
        .map_err(|_| LifecycleError::OperationTimeout {
            timeout_ms: u64::try_from(self.default_timeout.as_millis()).unwrap_or(u64::MAX),
        })?;

        // Clean up active hot reload tracking
        {
            let mut active = self.active_hot_reloads.lock().await;
            active.remove(&reload_id);
        }

        match hot_reload_result {
            Ok(result) => {
                // Update agent lifecycle based on hot reload result
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent_lifecycle) = agents.get_mut(&agent_id) {
                        if result.status.is_success() {
                            // Update version information
                            agent_lifecycle.version = to_version;
                            agent_lifecycle.version_number = to_version_number;

                            // Ensure agent is in running state after successful hot reload
                            if agent_lifecycle.current_state != AgentLifecycleState::Running {
                                if let Err(e) = agent_lifecycle.start() {
                                    warn!("Failed to start agent after hot reload: {}", e);
                                }
                            }
                        } else {
                            let failure_reason = AgentFailureReason::try_new(
                                result
                                    .error_message
                                    .clone()
                                    .unwrap_or_else(|| "Hot reload failed".to_string()),
                            )
                            .unwrap_or_else(|_| {
                                AgentFailureReason::try_new(
                                    "Unknown hot reload failure".to_string(),
                                )
                                .unwrap()
                            });
                            if let Err(e) = agent_lifecycle.fail(failure_reason) {
                                warn!("Failed to mark agent as failed after hot reload: {}", e);
                            }
                        }
                    }
                }

                // Update agent status
                self.update_agent_status(agent_id, None, Some(reload_id))
                    .await;

                info!(
                    "Hot reload completed successfully in {:?}",
                    start_time.elapsed().unwrap_or_default()
                );
                Ok(result)
            }
            Err(e) => {
                // Mark agent as failed if hot reload fails critically
                {
                    let mut agents = self.agents.write().await;
                    if let Some(agent_lifecycle) = agents.get_mut(&agent_id) {
                        let failure_reason =
                            AgentFailureReason::from_error(&e).unwrap_or_else(|_| {
                                AgentFailureReason::try_new(e.to_string()).unwrap_or_else(|_| {
                                    AgentFailureReason::try_new(
                                        "Unknown hot reload failure".to_string(),
                                    )
                                    .unwrap()
                                })
                            });
                        if let Err(transition_err) = agent_lifecycle.fail(failure_reason) {
                            error!("Failed to mark agent as failed: {}", transition_err);
                        }
                    }
                }

                error!("Hot reload failed: {}", e);
                Err(LifecycleError::HotReloadError(e))
            }
        }
    }

    /// Start an agent (transition from Ready to Running)
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or not in the Ready state.
    pub async fn start_agent(&self, agent_id: AgentId) -> Result<OperationResult> {
        let start_time = SystemTime::now();
        debug!("Starting agent: {}", agent_id);

        let mut agents = self.agents.write().await;
        let agent = agents
            .get_mut(&agent_id)
            .ok_or(LifecycleError::AgentNotFound { agent_id })?;

        agent.start()?;

        // Update agent status
        drop(agents); // Release write lock
        self.update_agent_status(agent_id, None, None).await;

        info!("Agent started successfully: {}", agent_id);
        Ok(OperationResult::success(start_time))
    }

    /// Stop an agent gracefully
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or cannot be stopped.
    pub async fn stop_agent(
        &self,
        agent_id: AgentId,
        drain_timeout: Option<Duration>,
    ) -> Result<OperationResult> {
        let start_time = SystemTime::now();
        info!("Stopping agent: {}", agent_id);

        let drain_duration = drain_timeout.unwrap_or(Duration::from_secs(60));

        // Begin draining if agent is running
        {
            let mut agents = self.agents.write().await;
            let agent = agents
                .get_mut(&agent_id)
                .ok_or(LifecycleError::AgentNotFound { agent_id })?;

            if agent.current_state.can_drain() {
                agent.start_draining()?;
                info!("Agent {} started draining requests", agent_id);
            }
        }

        // Wait for drain timeout or until no pending requests
        let drain_start = Instant::now();
        while drain_start.elapsed() < drain_duration {
            let agents = self.agents.read().await;
            let agent = agents
                .get(&agent_id)
                .ok_or(LifecycleError::AgentNotFound { agent_id })?;

            if agent.is_ready_to_stop() {
                debug!(
                    "Agent {} has no pending requests, stopping immediately",
                    agent_id
                );
                break;
            }

            drop(agents); // Release read lock
            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        // Force stop the agent
        {
            let mut agents = self.agents.write().await;
            let agent = agents
                .get_mut(&agent_id)
                .ok_or(LifecycleError::AgentNotFound { agent_id })?;

            agent.stop()?;
        }

        // Update agent status
        self.update_agent_status(agent_id, None, None).await;

        info!("Agent stopped successfully: {}", agent_id);
        Ok(OperationResult::success(start_time))
    }

    /// Get agent status
    /// Get the current status of an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found.
    pub async fn get_agent_status(&self, agent_id: AgentId) -> Result<AgentStatus> {
        let status_map = self.agent_status.read().await;
        status_map
            .get(&agent_id)
            .cloned()
            .ok_or(LifecycleError::AgentNotFound { agent_id })
    }

    /// List all agents with their current status
    pub async fn list_agents(&self) -> HashMap<AgentId, AgentStatus> {
        self.agent_status.read().await.clone()
    }

    /// Get agent lifecycle state
    /// Get the lifecycle information for an agent
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found.
    pub async fn get_agent_lifecycle(&self, agent_id: AgentId) -> Result<AgentLifecycle> {
        let agents = self.agents.read().await;
        agents
            .get(&agent_id)
            .cloned()
            .ok_or(LifecycleError::AgentNotFound { agent_id })
    }

    /// Remove agent from lifecycle management
    /// Remove an agent from the lifecycle manager
    ///
    /// # Errors
    ///
    /// Returns an error if the agent is not found or cannot be removed.
    pub async fn remove_agent(&self, agent_id: AgentId) -> Result<OperationResult> {
        let start_time = SystemTime::now();
        info!("Removing agent: {}", agent_id);

        // First stop the agent if it's running
        if let Ok(lifecycle) = self.get_agent_lifecycle(agent_id).await {
            if lifecycle.current_state.is_active() {
                self.stop_agent(agent_id, Some(Duration::from_secs(30)))
                    .await?;
            }
        }

        // Clean up deployed resources through deployment manager
        if let Err(deployment_error) = self.deployment_manager.cleanup_agent(agent_id).await {
            warn!("Failed to cleanup agent resources: {}", deployment_error);
            // Continue with removal even if cleanup fails
        }

        // Remove from all tracking maps
        {
            let mut agents = self.agents.write().await;
            agents.remove(&agent_id);
        }

        {
            let mut status = self.agent_status.write().await;
            status.remove(&agent_id);
        }

        info!("Agent removed successfully: {}", agent_id);
        Ok(OperationResult::success(start_time))
    }

    /// Update agent status with latest information
    async fn update_agent_status(
        &self,
        agent_id: AgentId,
        deployment_id: Option<DeploymentId>,
        hot_reload_id: Option<HotReloadId>,
    ) {
        let agents = self.agents.read().await;
        if let Some(lifecycle) = agents.get(&agent_id) {
            let mut status_map = self.agent_status.write().await;

            let existing_status = status_map.get(&agent_id);
            let uptime = existing_status
                .and_then(|s| SystemTime::now().duration_since(s.last_activity).ok())
                .unwrap_or_default();

            let health_status = match lifecycle.current_state {
                AgentLifecycleState::Running => HealthStatus::Healthy,
                AgentLifecycleState::Draining => HealthStatus::Degraded {
                    reason: "Agent is draining requests".to_string(),
                },
                AgentLifecycleState::Failed => HealthStatus::Unhealthy {
                    reason: lifecycle
                        .failure_reason
                        .as_ref()
                        .map_or_else(|| "Unknown failure".to_string(), |r| r.clone().into_inner()),
                },
                _ => HealthStatus::Unknown,
            };

            let agent_status = AgentStatus {
                lifecycle: lifecycle.clone(),
                deployment_id: deployment_id
                    .or_else(|| existing_status.and_then(|s| s.deployment_id)),
                hot_reload_id: hot_reload_id
                    .or_else(|| existing_status.and_then(|s| s.hot_reload_id)),
                memory_allocated: existing_status
                    .map_or_else(MemoryBytes::zero, |s| s.memory_allocated),
                uptime,
                last_activity: SystemTime::now(),
                health_status,
            };

            status_map.insert(agent_id, agent_status);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{DeploymentConfig, DeploymentStrategy, HotReloadConfig, HotReloadStrategy};
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock implementations for testing
    struct MockDeploymentManager {
        should_succeed: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl DeploymentManager for MockDeploymentManager {
        async fn deploy_agent(
            &self,
            request: DeploymentRequest,
        ) -> std::result::Result<DeploymentResult, DeploymentError> {
            if self.should_succeed.load(Ordering::SeqCst) {
                Ok(DeploymentResult::success(
                    request.deployment_id,
                    request.agent_id,
                    SystemTime::now(),
                    SystemTime::now(),
                    None,
                ))
            } else {
                Err(DeploymentError::ValidationFailed(
                    crate::domain::DeploymentValidationError::EmptyWasmModule,
                ))
            }
        }

        async fn get_deployment_status(
            &self,
            _: DeploymentId,
        ) -> std::result::Result<DeploymentStatus, DeploymentError> {
            Ok(DeploymentStatus::Completed)
        }

        async fn cancel_deployment(
            &self,
            _: DeploymentId,
        ) -> std::result::Result<(), DeploymentError> {
            Ok(())
        }

        async fn rollback_deployment(
            &self,
            _: DeploymentId,
            _: AgentVersion,
        ) -> std::result::Result<DeploymentResult, DeploymentError> {
            Ok(DeploymentResult::success(
                DeploymentId::generate(),
                AgentId::generate(),
                SystemTime::now(),
                SystemTime::now(),
                None,
            ))
        }

        async fn cleanup_agent(
            &self,
            _agent_id: AgentId,
        ) -> std::result::Result<(), DeploymentError> {
            // Mock implementation - always succeeds
            Ok(())
        }
    }

    struct MockHotReloadManager;

    #[async_trait::async_trait]
    impl HotReloadManager for MockHotReloadManager {
        async fn hot_reload_agent(
            &self,
            request: HotReloadRequest,
        ) -> std::result::Result<HotReloadResult, HotReloadError> {
            Ok(HotReloadResult::success(
                request.reload_id,
                request.agent_id,
                request.from_version,
                request.to_version,
                SystemTime::now(),
                None,
                vec![],
            ))
        }

        async fn get_hot_reload_status(
            &self,
            _: HotReloadId,
        ) -> std::result::Result<HotReloadStatus, HotReloadError> {
            Ok(HotReloadStatus::Completed)
        }

        async fn cancel_hot_reload(
            &self,
            _: HotReloadId,
        ) -> std::result::Result<(), HotReloadError> {
            Ok(())
        }

        async fn rollback_hot_reload(
            &self,
            _: HotReloadId,
            _: AgentVersion,
        ) -> std::result::Result<HotReloadResult, HotReloadError> {
            Ok(HotReloadResult::success(
                HotReloadId::generate(),
                AgentId::generate(),
                AgentVersion::generate(),
                AgentVersion::generate(),
                SystemTime::now(),
                None,
                vec![],
            ))
        }
    }

    struct MockWasmModuleValidator;

    #[async_trait::async_trait]
    impl WasmModuleValidator for MockWasmModuleValidator {
        async fn validate_module(
            &self,
            wasm_bytes: &[u8],
            _: Option<AgentName>,
        ) -> std::result::Result<WasmModule, crate::domain::WasmValidationError> {
            if wasm_bytes.is_empty() {
                return Err(crate::domain::WasmValidationError::EmptyModule);
            }

            WasmModule::from_bytes(
                AgentVersion::generate(),
                VersionNumber::first(),
                None,
                None,
                wasm_bytes,
                &crate::domain::WasmSecurityPolicy::testing(),
            )
        }

        async fn validate_security(
            &self,
            _: &WasmModule,
        ) -> std::result::Result<ValidationResult, crate::domain::WasmValidationError> {
            Ok(ValidationResult::Valid)
        }

        async fn extract_metadata(
            &self,
            _: &[u8],
        ) -> std::result::Result<HashMap<String, String>, crate::domain::WasmValidationError>
        {
            Ok(HashMap::new())
        }
    }

    fn create_test_manager() -> AgentLifecycleManager {
        let deployment_manager = Arc::new(MockDeploymentManager {
            should_succeed: Arc::new(AtomicBool::new(true)),
        });
        let hot_reload_manager = Arc::new(MockHotReloadManager);
        let module_validator = Arc::new(MockWasmModuleValidator);

        AgentLifecycleManager::with_timeout(
            deployment_manager,
            hot_reload_manager,
            module_validator,
            Duration::from_secs(10),
        )
    }

    #[tokio::test]
    async fn test_successful_agent_deployment() {
        let manager = create_test_manager();

        let agent_id = AgentId::generate();
        let agent_name = Some(AgentName::try_new("test-agent".to_string()).unwrap());
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
        let wasm_bytes = vec![1, 2, 3, 4]; // Mock WASM

        let result = manager
            .deploy_agent(
                agent_id,
                agent_name,
                version,
                version_number,
                config,
                wasm_bytes,
            )
            .await;

        assert!(result.is_ok());
        assert!(result.unwrap().status.is_success());

        // Check agent status
        let status = manager.get_agent_status(agent_id).await.unwrap();
        assert_eq!(status.lifecycle.current_state, AgentLifecycleState::Ready);
    }

    #[tokio::test]
    async fn test_agent_lifecycle_transitions() {
        let manager = create_test_manager();

        let agent_id = AgentId::generate();
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
        let wasm_bytes = vec![1, 2, 3, 4];

        // Deploy agent
        manager
            .deploy_agent(agent_id, None, version, version_number, config, wasm_bytes)
            .await
            .unwrap();

        // Start agent
        let result = manager.start_agent(agent_id).await.unwrap();
        assert!(result.success);

        let status = manager.get_agent_status(agent_id).await.unwrap();
        assert_eq!(status.lifecycle.current_state, AgentLifecycleState::Running);

        // Stop agent
        let result = manager.stop_agent(agent_id, None).await.unwrap();
        assert!(result.success);

        let status = manager.get_agent_status(agent_id).await.unwrap();
        assert_eq!(status.lifecycle.current_state, AgentLifecycleState::Stopped);
    }

    #[tokio::test]
    async fn test_hot_reload() {
        let manager = create_test_manager();

        let agent_id = AgentId::generate();
        let from_version = AgentVersion::generate();
        let to_version = AgentVersion::generate();
        let version_number = VersionNumber::first();

        // Deploy initial agent
        let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
        manager
            .deploy_agent(
                agent_id,
                None,
                from_version,
                version_number,
                config,
                vec![1, 2, 3, 4],
            )
            .await
            .unwrap();

        // Start agent
        manager.start_agent(agent_id).await.unwrap();

        // Hot reload
        let reload_config = HotReloadConfig::new(HotReloadStrategy::Graceful);
        let new_version_number = version_number.next().unwrap();

        let result = manager
            .hot_reload_agent(
                agent_id,
                from_version,
                to_version,
                new_version_number,
                reload_config,
                vec![5, 6, 7, 8], // New WASM
            )
            .await
            .unwrap();

        assert!(result.status.is_success());

        // Check that version was updated
        let lifecycle = manager.get_agent_lifecycle(agent_id).await.unwrap();
        assert_eq!(lifecycle.version, to_version);
        assert_eq!(lifecycle.version_number, new_version_number);
    }

    #[tokio::test]
    async fn test_agent_removal() {
        let manager = create_test_manager();

        let agent_id = AgentId::generate();
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::new(DeploymentStrategy::Immediate);

        // Deploy and start agent
        manager
            .deploy_agent(
                agent_id,
                None,
                version,
                version_number,
                config,
                vec![1, 2, 3, 4],
            )
            .await
            .unwrap();

        manager.start_agent(agent_id).await.unwrap();

        // Remove agent
        let result = manager.remove_agent(agent_id).await.unwrap();
        assert!(result.success);

        // Verify agent is removed
        assert!(manager.get_agent_status(agent_id).await.is_err());
        assert!(manager.get_agent_lifecycle(agent_id).await.is_err());
    }

    #[tokio::test]
    async fn test_validation_failure() {
        let manager = create_test_manager();

        let agent_id = AgentId::generate();
        let version = AgentVersion::generate();
        let version_number = VersionNumber::first();
        let config = DeploymentConfig::new(DeploymentStrategy::Immediate);
        let empty_wasm = vec![]; // Empty WASM should fail validation

        let result = manager
            .deploy_agent(agent_id, None, version, version_number, config, empty_wasm)
            .await;

        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            LifecycleError::ValidationFailed { .. }
        ));
    }
}
