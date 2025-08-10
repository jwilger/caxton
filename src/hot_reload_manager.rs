//! Hot Reload Manager
//!
//! Manages hot reloading of agent WASM modules with zero downtime,
//! supporting graceful, immediate, parallel, and traffic-splitting strategies.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

use crate::agent_lifecycle_manager::HotReloadManager;
use crate::domain::hot_reload::ResourceUsageSnapshot;
#[allow(unused_imports)]
use crate::domain::{
    AgentVersion, HotReloadConfig, HotReloadError, HotReloadId, HotReloadRequest, HotReloadResult,
    HotReloadStatus, HotReloadStrategy, ReloadMetrics, TrafficSplitPercentage, VersionNumber,
    VersionSnapshot,
};
use crate::domain_types::AgentId;
use crate::time_provider::{SharedTimeProvider, production_time_provider};

/// Hot reload execution context
#[derive(Debug, Clone)]
struct HotReloadContext {
    pub request: HotReloadRequest,
    pub started_at: SystemTime,
    pub status: HotReloadStatus,
    pub metrics: ReloadMetrics,
    pub current_traffic_split: TrafficSplitPercentage,
    pub version_snapshots: Vec<VersionSnapshot>,
    pub warmup_completed: bool,
}

/// Agent instance for hot reload operations
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct AgentInstance {
    pub agent_id: AgentId,
    pub version: AgentVersion,
    pub is_active: bool,
    pub memory_usage: usize,
    pub fuel_consumed: u64,
    pub requests_handled: u64,
    pub created_at: SystemTime,
}

/// State preservation data during hot reload
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct PreservedState {
    pub agent_id: AgentId,
    pub state_data: Vec<u8>,
    pub preserved_at: SystemTime,
}

/// Traffic routing decision
#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TrafficDecision {
    pub route_to_new_version: bool,
    pub old_version_weight: u8,
    pub new_version_weight: u8,
}

/// Agent runtime manager interface
#[async_trait::async_trait]
pub trait RuntimeManager {
    /// Create a new agent instance with WASM module
    async fn create_instance(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
        wasm_bytes: &[u8],
    ) -> Result<(), HotReloadError>;

    /// Stop an agent instance
    async fn stop_instance(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<(), HotReloadError>;

    /// Get instance metrics
    async fn get_instance_metrics(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<(usize, u64, u64), HotReloadError>; // (memory, fuel, requests)

    /// Preserve agent state
    async fn preserve_state(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<Vec<u8>, HotReloadError>;

    /// Restore agent state
    async fn restore_state(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
        state_data: &[u8],
    ) -> Result<(), HotReloadError>;

    /// Check if instance is healthy
    async fn health_check(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
    ) -> Result<bool, HotReloadError>;
}

/// Traffic routing manager interface
#[async_trait::async_trait]
pub trait TrafficRouter {
    /// Route traffic between agent versions
    async fn set_traffic_split(
        &self,
        agent_id: AgentId,
        old_version: AgentVersion,
        new_version: AgentVersion,
        split_percentage: TrafficSplitPercentage,
    ) -> Result<(), HotReloadError>;

    /// Get current traffic distribution
    async fn get_traffic_split(
        &self,
        agent_id: AgentId,
    ) -> Result<TrafficSplitPercentage, HotReloadError>;

    /// Switch all traffic to new version
    async fn switch_traffic(
        &self,
        agent_id: AgentId,
        target_version: AgentVersion,
    ) -> Result<(), HotReloadError>;
}

/// Core hot reload manager implementation
pub struct CaxtonHotReloadManager {
    /// Active hot reload operations
    active_reloads: Arc<RwLock<HashMap<HotReloadId, HotReloadContext>>>,
    /// Version snapshots for rollback
    version_snapshots: Arc<RwLock<HashMap<AgentId, Vec<VersionSnapshot>>>>,
    /// Preserved state data
    preserved_states: Arc<Mutex<HashMap<AgentId, PreservedState>>>,
    /// Runtime manager for agent instances
    runtime_manager: Arc<dyn RuntimeManager + Send + Sync>,
    /// Traffic router for managing traffic splits
    traffic_router: Arc<dyn TrafficRouter + Send + Sync>,
    /// Time provider for testable time operations
    time_provider: SharedTimeProvider,
    /// Maximum concurrent hot reloads
    max_concurrent_reloads: usize,
    /// Default operation timeout
    default_timeout: Duration,
}

impl CaxtonHotReloadManager {
    /// Creates a new hot reload manager
    pub fn new(
        runtime_manager: Arc<dyn RuntimeManager + Send + Sync>,
        traffic_router: Arc<dyn TrafficRouter + Send + Sync>,
    ) -> Self {
        Self::with_time_provider(runtime_manager, traffic_router, production_time_provider())
    }

    /// Creates a new hot reload manager with custom time provider
    pub fn with_time_provider(
        runtime_manager: Arc<dyn RuntimeManager + Send + Sync>,
        traffic_router: Arc<dyn TrafficRouter + Send + Sync>,
        time_provider: SharedTimeProvider,
    ) -> Self {
        Self {
            active_reloads: Arc::new(RwLock::new(HashMap::new())),
            version_snapshots: Arc::new(RwLock::new(HashMap::new())),
            preserved_states: Arc::new(Mutex::new(HashMap::new())),
            runtime_manager,
            traffic_router,
            time_provider,
            max_concurrent_reloads: 5,
            default_timeout: Duration::from_secs(300), // 5 minutes
        }
    }

    /// Creates hot reload manager with custom settings
    pub fn with_limits(
        runtime_manager: Arc<dyn RuntimeManager + Send + Sync>,
        traffic_router: Arc<dyn TrafficRouter + Send + Sync>,
        max_concurrent: usize,
        timeout: Duration,
    ) -> Self {
        Self::with_limits_and_time_provider(
            runtime_manager,
            traffic_router,
            max_concurrent,
            timeout,
            production_time_provider(),
        )
    }

    /// Creates hot reload manager with custom settings and time provider
    pub fn with_limits_and_time_provider(
        runtime_manager: Arc<dyn RuntimeManager + Send + Sync>,
        traffic_router: Arc<dyn TrafficRouter + Send + Sync>,
        max_concurrent: usize,
        timeout: Duration,
        time_provider: SharedTimeProvider,
    ) -> Self {
        Self {
            active_reloads: Arc::new(RwLock::new(HashMap::new())),
            version_snapshots: Arc::new(RwLock::new(HashMap::new())),
            preserved_states: Arc::new(Mutex::new(HashMap::new())),
            runtime_manager,
            traffic_router,
            time_provider,
            max_concurrent_reloads: max_concurrent,
            default_timeout: timeout,
        }
    }

    /// Check if hot reload limit is reached
    async fn check_reload_limit(&self) -> Result<(), HotReloadError> {
        let active = self.active_reloads.read().await;
        if active.len() >= self.max_concurrent_reloads {
            return Err(HotReloadError::InsufficientResources);
        }
        Ok(())
    }

    /// Execute hot reload based on strategy
    async fn execute_hot_reload_strategy(
        &self,
        mut context: HotReloadContext,
    ) -> Result<HotReloadResult, HotReloadError> {
        match context.request.config.strategy {
            HotReloadStrategy::Graceful => self.execute_graceful_reload(&mut context).await,
            HotReloadStrategy::Immediate => self.execute_immediate_reload(&mut context).await,
            HotReloadStrategy::Parallel => self.execute_parallel_reload(&mut context).await,
            HotReloadStrategy::TrafficSplitting => {
                self.execute_traffic_splitting_reload(&mut context).await
            }
        }
    }

    /// Execute graceful hot reload (drain requests then switch)
    async fn execute_graceful_reload(
        &self,
        context: &mut HotReloadContext,
    ) -> Result<HotReloadResult, HotReloadError> {
        info!(
            "Executing graceful hot reload for agent {}",
            context.request.agent_id
        );

        let agent_id = context.request.agent_id;
        let old_version = context.request.from_version;
        let new_version = context.request.to_version;

        // Step 1: Create snapshot of current version
        context.status = HotReloadStatus::Preparing;
        self.create_version_snapshot(agent_id, old_version, context)
            .await?;

        // Step 2: Preserve state if requested
        if context.request.preserve_state {
            let state_data = self
                .runtime_manager
                .preserve_state(agent_id, old_version)
                .await?;

            let preserved_state = PreservedState {
                agent_id,
                state_data,
                preserved_at: SystemTime::now(),
            };

            let mut states = self.preserved_states.lock().await;
            states.insert(agent_id, preserved_state);
        }

        // Step 3: Create new instance
        context.status = HotReloadStatus::Starting;
        self.runtime_manager
            .create_instance(agent_id, new_version, &context.request.new_wasm_module)
            .await?;

        // Step 4: Warmup period
        if context.request.config.warmup_duration > Duration::from_secs(0) {
            info!(
                "Warming up new version for {:?}",
                context.request.config.warmup_duration
            );
            self.time_provider
                .sleep(context.request.config.warmup_duration)
                .await;
        }
        context.warmup_completed = true;

        // Step 5: Health check new version
        if !self
            .runtime_manager
            .health_check(agent_id, new_version)
            .await?
        {
            return Err(HotReloadError::StatePreservationFailed {
                reason: "New version failed health check".to_string(),
            });
        }

        // Step 6: Restore state if preserved
        if context.request.preserve_state {
            let states = self.preserved_states.lock().await;
            if let Some(preserved) = states.get(&agent_id) {
                self.runtime_manager
                    .restore_state(agent_id, new_version, &preserved.state_data)
                    .await?;
            }
        }

        // Step 7: Drain old version
        context.status = HotReloadStatus::InProgress;

        if !self.time_provider.should_skip_delays() {
            info!(
                "Draining old version for {:?}",
                context.request.config.drain_timeout.as_duration()
            );
            let drain_start = self.time_provider.instant();
            let drain_duration = context.request.config.drain_timeout.as_duration();

            // Wait for drain to complete or timeout
            while drain_start.elapsed() < drain_duration {
                // In a real implementation, we'd check if there are pending requests
                // For now, we'll just wait with small increments
                self.time_provider.sleep(Duration::from_millis(100)).await;

                // Check if drain is complete (placeholder for real implementation)
                // if self.is_drain_complete(agent_id, old_version).await { break; }
            }
        } else {
            debug!("Skipping drain wait in test mode");
        }

        // Step 8: Switch traffic to new version
        self.traffic_router
            .switch_traffic(agent_id, new_version)
            .await?;

        // Step 9: Stop old version
        self.runtime_manager
            .stop_instance(agent_id, old_version)
            .await?;

        context.status = HotReloadStatus::Completed;

        // Collect final metrics
        self.update_metrics(context).await?;

        Ok(HotReloadResult::success(
            context.request.reload_id,
            agent_id,
            old_version,
            new_version,
            context.started_at,
            Some(context.metrics.clone()),
            context.version_snapshots.clone(),
        ))
    }

    /// Execute immediate hot reload (terminate old, start new)
    async fn execute_immediate_reload(
        &self,
        context: &mut HotReloadContext,
    ) -> Result<HotReloadResult, HotReloadError> {
        info!(
            "Executing immediate hot reload for agent {}",
            context.request.agent_id
        );

        let agent_id = context.request.agent_id;
        let old_version = context.request.from_version;
        let new_version = context.request.to_version;

        // Step 1: Create version snapshot
        context.status = HotReloadStatus::Preparing;
        self.create_version_snapshot(agent_id, old_version, context)
            .await?;

        // Step 2: Create new instance first (before stopping old one)
        context.status = HotReloadStatus::Starting;
        self.runtime_manager
            .create_instance(agent_id, new_version, &context.request.new_wasm_module)
            .await?;

        // Step 3: Switch traffic to new version
        context.status = HotReloadStatus::InProgress;
        self.traffic_router
            .switch_traffic(agent_id, new_version)
            .await?;

        // Step 4: Stop old version (only after new one is running)
        self.runtime_manager
            .stop_instance(agent_id, old_version)
            .await?;

        // Step 5: Health check
        if !self
            .runtime_manager
            .health_check(agent_id, new_version)
            .await?
        {
            warn!("New version failed health check, but immediate reload cannot rollback");
        }

        context.status = HotReloadStatus::Completed;

        // Collect metrics
        self.update_metrics(context).await?;

        Ok(HotReloadResult::success(
            context.request.reload_id,
            agent_id,
            old_version,
            new_version,
            context.started_at,
            Some(context.metrics.clone()),
            context.version_snapshots.clone(),
        ))
    }

    /// Execute parallel hot reload (run both versions simultaneously)
    async fn execute_parallel_reload(
        &self,
        context: &mut HotReloadContext,
    ) -> Result<HotReloadResult, HotReloadError> {
        info!(
            "Executing parallel hot reload for agent {}",
            context.request.agent_id
        );

        let agent_id = context.request.agent_id;
        let old_version = context.request.from_version;
        let new_version = context.request.to_version;

        // Step 1: Create version snapshot
        context.status = HotReloadStatus::Preparing;
        self.create_version_snapshot(agent_id, old_version, context)
            .await?;

        // Step 2: Create new instance alongside old one
        context.status = HotReloadStatus::Starting;
        self.runtime_manager
            .create_instance(agent_id, new_version, &context.request.new_wasm_module)
            .await?;

        // Step 3: Warmup new version
        if context.request.config.warmup_duration > Duration::from_secs(0) {
            self.time_provider
                .sleep(context.request.config.warmup_duration)
                .await;
        }
        context.warmup_completed = true;

        // Step 4: Health check new version
        if !self
            .runtime_manager
            .health_check(agent_id, new_version)
            .await?
        {
            // Rollback by stopping new version
            self.runtime_manager
                .stop_instance(agent_id, new_version)
                .await?;
            return Err(HotReloadError::AutomaticRollback {
                reason: "New version failed health check".to_string(),
            });
        }

        // Step 5: Run both versions in parallel for monitoring
        context.status = HotReloadStatus::InProgress;

        // Monitor both versions for a period
        let monitoring_duration = if self.time_provider.should_skip_delays() {
            Duration::from_millis(1) // Skip monitoring in tests
        } else {
            Duration::from_secs(60) // 1 minute monitoring in production
        };
        let monitor_start = SystemTime::now();

        while monitor_start.elapsed().unwrap_or_default() < monitoring_duration {
            // Collect metrics from both versions
            self.update_metrics(context).await?;

            // Check if rollback is needed
            if context
                .request
                .config
                .rollback_capability
                .should_trigger_rollback(&context.metrics)
            {
                warn!("Automatic rollback triggered during parallel execution");
                self.runtime_manager
                    .stop_instance(agent_id, new_version)
                    .await?;
                return Err(HotReloadError::AutomaticRollback {
                    reason: "Metrics triggered automatic rollback".to_string(),
                });
            }

            let check_interval = if self.time_provider.should_skip_delays() {
                Duration::from_millis(1)
            } else {
                Duration::from_secs(5)
            };
            self.time_provider.sleep(check_interval).await;
        }

        // Step 6: Switch traffic to new version
        self.traffic_router
            .switch_traffic(agent_id, new_version)
            .await?;

        // Step 7: Stop old version
        self.runtime_manager
            .stop_instance(agent_id, old_version)
            .await?;

        context.status = HotReloadStatus::Completed;

        Ok(HotReloadResult::success(
            context.request.reload_id,
            agent_id,
            old_version,
            new_version,
            context.started_at,
            Some(context.metrics.clone()),
            context.version_snapshots.clone(),
        ))
    }

    /// Execute traffic splitting hot reload (gradual traffic shift)
    async fn execute_traffic_splitting_reload(
        &self,
        context: &mut HotReloadContext,
    ) -> Result<HotReloadResult, HotReloadError> {
        info!(
            "Executing traffic splitting hot reload for agent {}",
            context.request.agent_id
        );

        let agent_id = context.request.agent_id;
        let old_version = context.request.from_version;
        let new_version = context.request.to_version;

        // Step 1: Create version snapshot
        context.status = HotReloadStatus::Preparing;
        self.create_version_snapshot(agent_id, old_version, context)
            .await?;

        // Step 2: Create new instance
        context.status = HotReloadStatus::Starting;
        self.runtime_manager
            .create_instance(agent_id, new_version, &context.request.new_wasm_module)
            .await?;

        // Step 3: Warmup
        if context.request.config.warmup_duration > Duration::from_secs(0) {
            self.time_provider
                .sleep(context.request.config.warmup_duration)
                .await;
        }
        context.warmup_completed = true;

        // Step 4: Health check
        if !self
            .runtime_manager
            .health_check(agent_id, new_version)
            .await?
        {
            self.runtime_manager
                .stop_instance(agent_id, new_version)
                .await?;
            return Err(HotReloadError::AutomaticRollback {
                reason: "New version failed initial health check".to_string(),
            });
        }

        context.status = HotReloadStatus::InProgress;

        // Step 5: Gradual traffic splitting
        let traffic_steps = if context.request.config.progressive_rollout {
            vec![5, 10, 25, 50, 75, 100]
        } else {
            vec![context.request.config.traffic_split.as_percentage()]
        };

        for (i, percentage) in traffic_steps.iter().enumerate() {
            let split_percentage = TrafficSplitPercentage::try_new(*percentage).map_err(|_| {
                HotReloadError::TrafficSplittingFailed {
                    reason: "Invalid traffic percentage".to_string(),
                }
            })?;

            info!("Setting traffic split to {}% for new version", percentage);

            // Set traffic split
            self.traffic_router
                .set_traffic_split(agent_id, old_version, new_version, split_percentage)
                .await?;

            context.current_traffic_split = split_percentage;

            // Monitor at this traffic level
            let monitor_duration = if self.time_provider.should_skip_delays() {
                Duration::from_millis(1) // Skip monitoring in tests
            } else {
                Duration::from_secs(30) // 30 seconds per step in production
            };
            let step_start = SystemTime::now();

            while step_start.elapsed().unwrap_or_default() < monitor_duration {
                self.update_metrics(context).await?;

                // Check rollback conditions
                if context
                    .request
                    .config
                    .rollback_capability
                    .should_trigger_rollback(&context.metrics)
                {
                    warn!("Automatic rollback triggered at {}% traffic", percentage);

                    // Rollback to 0% traffic to new version
                    let zero_split = TrafficSplitPercentage::try_new(0).unwrap();
                    self.traffic_router
                        .set_traffic_split(agent_id, old_version, new_version, zero_split)
                        .await?;

                    // Stop new version
                    self.runtime_manager
                        .stop_instance(agent_id, new_version)
                        .await?;

                    return Err(HotReloadError::AutomaticRollback {
                        reason: format!("Rollback triggered at {}% traffic", percentage),
                    });
                }

                let check_interval = if self.time_provider.should_skip_delays() {
                    Duration::from_millis(1)
                } else {
                    Duration::from_secs(5)
                };
                self.time_provider.sleep(check_interval).await;
            }

            // If not the last step, continue to next traffic percentage
            if i < traffic_steps.len() - 1 {
                info!(
                    "Traffic split at {}% successful, proceeding to next step",
                    percentage
                );
            }
        }

        // Step 6: Complete rollout - switch traffic and stop old version
        self.traffic_router
            .switch_traffic(agent_id, new_version)
            .await?;

        self.runtime_manager
            .stop_instance(agent_id, old_version)
            .await?;

        context.status = HotReloadStatus::Completed;

        Ok(HotReloadResult::success(
            context.request.reload_id,
            agent_id,
            old_version,
            new_version,
            context.started_at,
            Some(context.metrics.clone()),
            context.version_snapshots.clone(),
        ))
    }

    /// Create version snapshot for rollback capability
    async fn create_version_snapshot(
        &self,
        agent_id: AgentId,
        version: AgentVersion,
        context: &mut HotReloadContext,
    ) -> Result<(), HotReloadError> {
        debug!(
            "Creating version snapshot for agent {} version {}",
            agent_id, version
        );

        // Get current metrics
        let (memory, fuel, requests) = self
            .runtime_manager
            .get_instance_metrics(agent_id, version)
            .await?;

        let snapshot = VersionSnapshot {
            version,
            version_number: context.request.to_version_number,
            wasm_module: context.request.new_wasm_module.clone(),
            created_at: SystemTime::now(),
            resource_usage: ResourceUsageSnapshot {
                memory_allocated: memory,
                fuel_consumed: fuel,
                requests_handled: requests,
                average_response_time_ms: 100, // Would be calculated from actual metrics
            },
        };

        context.version_snapshots.push(snapshot.clone());

        // Store in global snapshots
        let mut snapshots = self.version_snapshots.write().await;
        let agent_snapshots = snapshots.entry(agent_id).or_insert_with(Vec::new);
        agent_snapshots.push(snapshot);

        // Keep only the configured number of snapshots
        let max_snapshots = context
            .request
            .config
            .rollback_capability
            .preserve_previous_versions as usize;
        if agent_snapshots.len() > max_snapshots {
            agent_snapshots.drain(0..agent_snapshots.len() - max_snapshots);
        }

        Ok(())
    }

    /// Update metrics during hot reload
    async fn update_metrics(&self, context: &mut HotReloadContext) -> Result<(), HotReloadError> {
        let agent_id = context.request.agent_id;
        let new_version = context.request.to_version;

        // Get metrics from new version
        if let Ok((memory, _fuel, requests)) = self
            .runtime_manager
            .get_instance_metrics(agent_id, new_version)
            .await
        {
            // Update context metrics (simplified)
            context.metrics.memory_usage_peak = context.metrics.memory_usage_peak.max(memory);
            context.metrics.requests_processed = requests;

            // Health check
            if let Ok(healthy) = self
                .runtime_manager
                .health_check(agent_id, new_version)
                .await
            {
                context.metrics.health_check_success_rate = if healthy { 100.0 } else { 0.0 };
            }

            context.metrics.collected_at = SystemTime::now();
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl HotReloadManager for CaxtonHotReloadManager {
    /// Perform hot reload of agent WASM module
    async fn hot_reload_agent(
        &self,
        request: HotReloadRequest,
    ) -> std::result::Result<HotReloadResult, HotReloadError> {
        info!(
            "Starting hot reload for agent {} with strategy {:?}",
            request.agent_id, request.config.strategy
        );

        // Check reload limits
        self.check_reload_limit().await?;

        // Validate request
        request.validate()?;

        // Create hot reload context
        let context = HotReloadContext {
            request: request.clone(),
            started_at: SystemTime::now(),
            status: HotReloadStatus::Pending,
            metrics: ReloadMetrics::new(),
            current_traffic_split: request.config.traffic_split,
            version_snapshots: Vec::new(),
            warmup_completed: false,
        };

        // Track active reload
        {
            let mut active = self.active_reloads.write().await;
            active.insert(request.reload_id, context.clone());
        }

        // Execute hot reload strategy
        let result = timeout(
            self.default_timeout,
            self.execute_hot_reload_strategy(context),
        )
        .await
        .map_err(|_| HotReloadError::TimeoutExceeded {
            timeout: u64::try_from(self.default_timeout.as_millis()).unwrap_or(u64::MAX),
        })?;

        // Remove from active reloads
        {
            let mut active = self.active_reloads.write().await;
            active.remove(&request.reload_id);
        }

        match &result {
            Ok(reload_result) => {
                info!(
                    "Hot reload completed successfully for agent {} in {:?}",
                    request.agent_id,
                    reload_result.duration().unwrap_or_default()
                );
            }
            Err(e) => {
                error!("Hot reload failed for agent {}: {}", request.agent_id, e);
            }
        }

        result
    }

    /// Get hot reload status
    async fn get_hot_reload_status(
        &self,
        reload_id: HotReloadId,
    ) -> std::result::Result<HotReloadStatus, HotReloadError> {
        let active = self.active_reloads.read().await;
        if let Some(context) = active.get(&reload_id) {
            Ok(context.status)
        } else {
            // Not in active reloads, assume completed
            Ok(HotReloadStatus::Completed)
        }
    }

    /// Cancel active hot reload
    async fn cancel_hot_reload(
        &self,
        reload_id: HotReloadId,
    ) -> std::result::Result<(), HotReloadError> {
        let mut active = self.active_reloads.write().await;

        if let Some(context) = active.remove(&reload_id) {
            info!("Cancelling hot reload {}", reload_id);

            // Stop new instance if it was created
            if let Err(e) = self
                .runtime_manager
                .stop_instance(context.request.agent_id, context.request.to_version)
                .await
            {
                warn!("Failed to stop new instance during cancellation: {}", e);
            }

            // Reset traffic to old version
            if let Err(e) = self
                .traffic_router
                .switch_traffic(context.request.agent_id, context.request.from_version)
                .await
            {
                warn!("Failed to reset traffic during cancellation: {}", e);
            }

            Ok(())
        } else {
            Err(HotReloadError::AlreadyInProgress { reload_id })
        }
    }

    /// Rollback hot reload to previous version
    async fn rollback_hot_reload(
        &self,
        reload_id: HotReloadId,
        target_version: AgentVersion,
    ) -> std::result::Result<HotReloadResult, HotReloadError> {
        info!(
            "Rolling back hot reload {} to version {}",
            reload_id, target_version
        );

        let active = self.active_reloads.read().await;

        if let Some(context) = active.get(&reload_id) {
            let agent_id = context.request.agent_id;

            // Find the target version snapshot
            let snapshots = self.version_snapshots.read().await;
            let agent_snapshots =
                snapshots
                    .get(&agent_id)
                    .ok_or(HotReloadError::VersionNotFound {
                        version: target_version,
                    })?;

            let target_snapshot = agent_snapshots
                .iter()
                .find(|s| s.version == target_version)
                .ok_or(HotReloadError::VersionNotFound {
                    version: target_version,
                })?;

            // Stop current version
            self.runtime_manager
                .stop_instance(agent_id, context.request.to_version)
                .await
                .map_err(|e| HotReloadError::RollbackFailed {
                    reason: format!("Failed to stop current version: {}", e),
                })?;

            // Deploy target version
            self.runtime_manager
                .create_instance(agent_id, target_version, &target_snapshot.wasm_module)
                .await
                .map_err(|e| HotReloadError::RollbackFailed {
                    reason: format!("Failed to create target version instance: {}", e),
                })?;

            // Switch traffic
            self.traffic_router
                .switch_traffic(agent_id, target_version)
                .await
                .map_err(|e| HotReloadError::RollbackFailed {
                    reason: format!("Failed to switch traffic: {}", e),
                })?;

            Ok(HotReloadResult::rollback(
                reload_id,
                agent_id,
                context.request.from_version,
                context.request.to_version,
                Some(context.started_at),
                format!("Rolled back to version {}", target_version),
                Some(context.metrics.clone()),
            ))
        } else {
            Err(HotReloadError::AlreadyInProgress { reload_id })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::TrafficSplitPercentage;
    use std::sync::atomic::{AtomicBool, Ordering};

    // Mock implementations for testing
    struct MockRuntimeManager {
        should_succeed: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl RuntimeManager for MockRuntimeManager {
        async fn create_instance(
            &self,
            _: AgentId,
            _: AgentVersion,
            _: &[u8],
        ) -> Result<(), HotReloadError> {
            if self.should_succeed.load(Ordering::SeqCst) {
                Ok(())
            } else {
                Err(HotReloadError::StatePreservationFailed {
                    reason: "Mock creation failure".to_string(),
                })
            }
        }

        async fn stop_instance(&self, _: AgentId, _: AgentVersion) -> Result<(), HotReloadError> {
            Ok(())
        }

        async fn get_instance_metrics(
            &self,
            _: AgentId,
            _: AgentVersion,
        ) -> Result<(usize, u64, u64), HotReloadError> {
            Ok((1024, 1000, 100))
        }

        async fn preserve_state(
            &self,
            _: AgentId,
            _: AgentVersion,
        ) -> Result<Vec<u8>, HotReloadError> {
            Ok(vec![1, 2, 3, 4])
        }

        async fn restore_state(
            &self,
            _: AgentId,
            _: AgentVersion,
            _: &[u8],
        ) -> Result<(), HotReloadError> {
            Ok(())
        }

        async fn health_check(&self, _: AgentId, _: AgentVersion) -> Result<bool, HotReloadError> {
            Ok(self.should_succeed.load(Ordering::SeqCst))
        }
    }

    struct MockTrafficRouter;

    #[async_trait::async_trait]
    impl TrafficRouter for MockTrafficRouter {
        async fn set_traffic_split(
            &self,
            _: AgentId,
            _: AgentVersion,
            _: AgentVersion,
            _: TrafficSplitPercentage,
        ) -> Result<(), HotReloadError> {
            Ok(())
        }

        async fn get_traffic_split(
            &self,
            _: AgentId,
        ) -> Result<TrafficSplitPercentage, HotReloadError> {
            Ok(TrafficSplitPercentage::half())
        }

        async fn switch_traffic(&self, _: AgentId, _: AgentVersion) -> Result<(), HotReloadError> {
            Ok(())
        }
    }

    fn create_test_hot_reload_manager() -> CaxtonHotReloadManager {
        let runtime_manager = Arc::new(MockRuntimeManager {
            should_succeed: Arc::new(AtomicBool::new(true)),
        });
        let traffic_router = Arc::new(MockTrafficRouter);

        CaxtonHotReloadManager::new(runtime_manager, traffic_router)
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "Slow test - involves sleep operations"]
    async fn test_graceful_hot_reload() {
        let manager = create_test_hot_reload_manager();

        let mut config = HotReloadConfig::graceful();
        // Reduce timeouts for testing
        config.warmup_duration = Duration::from_millis(1);

        let request = HotReloadRequest::new(
            AgentId::generate(),
            None,
            AgentVersion::generate(),
            AgentVersion::generate(),
            VersionNumber::first().next().unwrap(),
            config,
            vec![5, 6, 7, 8],
        );

        let result =
            tokio::time::timeout(Duration::from_secs(1), manager.hot_reload_agent(request)).await;
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_ok());
    }

    #[tokio::test]
    async fn test_immediate_hot_reload() {
        let manager = create_test_hot_reload_manager();

        let mut config = HotReloadConfig::immediate();
        // Reduce warmup for testing
        config.warmup_duration = Duration::from_millis(1);

        let request = HotReloadRequest::new(
            AgentId::generate(),
            None,
            AgentVersion::generate(),
            AgentVersion::generate(),
            VersionNumber::first().next().unwrap(),
            config,
            vec![5, 6, 7, 8],
        );

        let result = manager.hot_reload_agent(request).await;
        assert!(result.is_ok());
    }

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    #[ignore = "Slow test - involves sleep operations"]
    async fn test_traffic_splitting_hot_reload() {
        let manager = create_test_hot_reload_manager();

        let traffic_split = TrafficSplitPercentage::try_new(25).unwrap();
        let mut config = HotReloadConfig::traffic_splitting(traffic_split);
        // Reduce timeouts for testing
        config.warmup_duration = Duration::from_millis(1);

        let request = HotReloadRequest::new(
            AgentId::generate(),
            None,
            AgentVersion::generate(),
            AgentVersion::generate(),
            VersionNumber::first().next().unwrap(),
            config,
            vec![5, 6, 7, 8],
        );

        let result =
            tokio::time::timeout(Duration::from_secs(1), manager.hot_reload_agent(request)).await;
        assert!(result.is_ok());
        let inner_result = result.unwrap();
        assert!(inner_result.is_ok());
    }

    #[tokio::test]
    async fn test_hot_reload_status() {
        let manager = create_test_hot_reload_manager();
        let reload_id = HotReloadId::generate();

        // Non-existent reload should return completed
        let status = manager.get_hot_reload_status(reload_id).await;
        assert!(status.is_ok());
        assert_eq!(status.unwrap(), HotReloadStatus::Completed);
    }

    #[tokio::test]
    async fn test_hot_reload_cancellation() {
        let manager = create_test_hot_reload_manager();
        let reload_id = HotReloadId::generate();

        // Cancelling non-existent reload should return error
        let result = manager.cancel_hot_reload(reload_id).await;
        assert!(result.is_err());
    }
}
