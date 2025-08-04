//! Agent lifecycle management with comprehensive state tracking
//!
//! This module provides advanced lifecycle management for agents including
//! automatic resource cleanup, health monitoring, and recovery mechanisms.

use crate::*;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot, RwLock};
use tokio::time::{interval, Duration as TokioDuration};

/// Lifecycle event types for detailed tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleEvent {
    AgentCreated {
        agent_id: AgentId,
        config: AgentConfig,
    },
    AgentStarted {
        agent_id: AgentId,
    },
    AgentSuspended {
        agent_id: AgentId,
        reason: String,
    },
    AgentResumed {
        agent_id: AgentId,
    },
    AgentTerminated {
        agent_id: AgentId,
        reason: String,
    },
    AgentCrashed {
        agent_id: AgentId,
        error: String,
    },
    ResourceLimitExceeded {
        agent_id: AgentId,
        resource_type: String,
        limit: u64,
        current: u64,
    },
    HealthCheckFailed {
        agent_id: AgentId,
        details: String,
    },
    StateTransition {
        agent_id: AgentId,
        from: AgentState,
        to: AgentState,
    },
}

/// Lifecycle manager for coordinating agent operations
#[derive(Clone)]
pub struct AgentLifecycleManager {
    runtime: Arc<CaxtonRuntime>,
    registry: AgentRegistry,
    monitor: AgentMonitor,
    event_tx: mpsc::UnboundedSender<LifecycleEvent>,
    recovery_enabled: Arc<RwLock<bool>>,
}

impl AgentLifecycleManager {
    /// Create a new lifecycle manager
    pub async fn new(runtime: Arc<CaxtonRuntime>) -> Result<Self, CaxtonError> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let manager = Self {
            runtime,
            registry: AgentRegistry::new(),
            monitor: AgentMonitor::new(),
            event_tx,
            recovery_enabled: Arc::new(RwLock::new(true)),
        };

        // Start background event processor
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.process_lifecycle_events(event_rx).await;
        });

        // Start health monitoring
        let manager_clone = manager.clone();
        tokio::spawn(async move {
            manager_clone.run_health_monitor().await;
        });

        Ok(manager)
    }

    /// Create and start a new agent with full lifecycle tracking
    #[instrument(name = "create_agent", skip(self, config), fields(agent_name = %config.name))]
    pub async fn create_agent(&self, config: AgentConfig) -> Result<AgentId, CaxtonError> {
        info!("Creating agent with lifecycle management: {}", config.name);

        // Emit creation event
        let _ = self.event_tx.send(LifecycleEvent::AgentCreated {
            agent_id: AgentId::new(), // Temporary ID for event
            config: config.clone(),
        });

        // Spawn agent through runtime
        let agent_id = self.runtime.spawn_agent(config).await?;

        // Register in our lifecycle registry
        let (metadata, _) = self.runtime.get_agent_metadata(&agent_id).await?;
        self.registry.register(agent_id.clone(), metadata);

        // Emit started event
        let _ = self.event_tx.send(LifecycleEvent::AgentStarted {
            agent_id: agent_id.clone(),
        });

        info!(agent_id = %agent_id, "Agent created and registered successfully");
        Ok(agent_id)
    }

    /// Gracefully stop an agent with cleanup
    #[instrument(name = "stop_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn stop_agent(
        &self,
        agent_id: &AgentId,
        reason: Option<String>,
    ) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Stopping agent with reason: {:?}", reason);

        // Perform health check before stopping
        let health_result = self.monitor.health_check(agent_id).await;
        if !health_result.healthy {
            warn!(agent_id = %agent_id, "Stopping unhealthy agent: {:?}", health_result.errors);
        }

        // Terminate through runtime
        let timeout = Duration::from_secs(30);
        self.runtime.terminate_agent(agent_id, timeout).await?;

        // Clean up monitoring
        self.monitor.cleanup_agent(agent_id);

        // Unregister from lifecycle registry
        self.registry.unregister(agent_id);

        // Emit termination event
        let _ = self.event_tx.send(LifecycleEvent::AgentTerminated {
            agent_id: agent_id.clone(),
            reason: reason.unwrap_or_else(|| "Graceful shutdown".to_string()),
        });

        info!(agent_id = %agent_id, "Agent stopped and cleaned up successfully");
        Ok(())
    }

    /// Suspend an agent for maintenance or debugging
    #[instrument(name = "suspend_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn suspend_agent(
        &self,
        agent_id: &AgentId,
        reason: String,
    ) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Suspending agent: {}", reason);

        self.runtime.suspend_agent(agent_id).await?;

        // Emit suspension event
        let _ = self.event_tx.send(LifecycleEvent::AgentSuspended {
            agent_id: agent_id.clone(),
            reason,
        });

        Ok(())
    }

    /// Resume a suspended agent
    #[instrument(name = "resume_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn resume_agent(&self, agent_id: &AgentId) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Resuming agent");

        self.runtime.resume_agent(agent_id).await?;

        // Emit resume event
        let _ = self.event_tx.send(LifecycleEvent::AgentResumed {
            agent_id: agent_id.clone(),
        });

        Ok(())
    }

    /// Force terminate an unresponsive agent
    #[instrument(name = "force_terminate_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn force_terminate_agent(
        &self,
        agent_id: &AgentId,
        reason: String,
    ) -> Result<(), CaxtonError> {
        warn!(agent_id = %agent_id, "Force terminating agent: {}", reason);

        self.runtime.force_terminate_agent(agent_id).await?;

        // Clean up monitoring and registry
        self.monitor.cleanup_agent(agent_id);
        self.registry.unregister(agent_id);

        // Emit crash event
        let _ = self.event_tx.send(LifecycleEvent::AgentCrashed {
            agent_id: agent_id.clone(),
            error: reason,
        });

        Ok(())
    }

    /// Get comprehensive agent status including lifecycle information
    pub async fn get_agent_status(
        &self,
        agent_id: &AgentId,
    ) -> Result<AgentStatusReport, CaxtonError> {
        let state = self.runtime.get_agent_state(agent_id).await?;
        let (metadata, resource_usage) = self.runtime.get_agent_metadata(agent_id).await?;
        let health_result = self.monitor.health_check(agent_id).await;
        let performance_metrics = self.monitor.get_performance_metrics(agent_id);

        Ok(AgentStatusReport {
            agent_id: agent_id.clone(),
            state,
            metadata,
            resource_usage,
            health_status: health_result,
            performance_metrics,
            last_updated: Utc::now(),
        })
    }

    /// List all agents managed by this lifecycle manager
    pub async fn list_managed_agents(&self) -> Vec<(AgentId, AgentMetadata)> {
        self.registry.list_all()
    }

    /// Get system-wide lifecycle statistics
    pub async fn get_lifecycle_stats(&self) -> LifecycleStats {
        let state_counts = self.registry.count_by_state();
        let capability_distribution = self.registry.capability_distribution();
        let system_stats = self.monitor.get_system_stats();

        LifecycleStats {
            total_agents: self.registry.total_count(),
            agents_by_state: state_counts,
            capability_distribution,
            system_metrics: system_stats,
            recovery_enabled: *self.recovery_enabled.read().await,
            last_updated: Utc::now(),
        }
    }

    /// Enable or disable automatic recovery
    pub async fn set_recovery_enabled(&self, enabled: bool) {
        *self.recovery_enabled.write().await = enabled;
        info!(
            "Automatic recovery {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Background task to process lifecycle events
    async fn process_lifecycle_events(
        &self,
        mut event_rx: mpsc::UnboundedReceiver<LifecycleEvent>,
    ) {
        while let Some(event) = event_rx.recv().await {
            match &event {
                LifecycleEvent::AgentCrashed { agent_id, error } => {
                    error!(agent_id = %agent_id, "Agent crashed: {}", error);

                    // Attempt recovery if enabled
                    if *self.recovery_enabled.read().await {
                        if let Err(e) = self.attempt_recovery(agent_id).await {
                            error!(agent_id = %agent_id, "Recovery failed: {:?}", e);
                        }
                    }
                }
                LifecycleEvent::HealthCheckFailed { agent_id, details } => {
                    warn!(agent_id = %agent_id, "Health check failed: {}", details);
                }
                LifecycleEvent::ResourceLimitExceeded {
                    agent_id,
                    resource_type,
                    limit,
                    current,
                } => {
                    warn!(
                        agent_id = %agent_id,
                        "Resource limit exceeded - {}: {} / {} limit",
                        resource_type, current, limit
                    );
                }
                _ => {
                    debug!("Lifecycle event: {:?}", event);
                }
            }

            // Record event in monitoring system
            self.record_lifecycle_event(&event).await;
        }
    }

    /// Background health monitoring
    async fn run_health_monitor(&self) {
        let mut interval = interval(TokioDuration::from_secs(60)); // Check every minute

        loop {
            interval.tick().await;

            let agents = self.registry.list_all();
            for (agent_id, _metadata) in agents {
                let health_result = self.monitor.health_check(&agent_id).await;

                if !health_result.healthy {
                    let _ = self.event_tx.send(LifecycleEvent::HealthCheckFailed {
                        agent_id: agent_id.clone(),
                        details: health_result.errors.join("; "),
                    });
                }

                // Record health check event
                self.monitor.record_event(&AgentEvent {
                    agent_id: agent_id.clone(),
                    timestamp: std::time::SystemTime::now(),
                    event_type: AgentEventType::HealthCheck {
                        status: HealthStatus {
                            healthy: health_result.healthy,
                            timestamp: health_result.timestamp,
                            details: if health_result.errors.is_empty() {
                                None
                            } else {
                                Some(health_result.errors.join("; "))
                            },
                            metrics: health_result
                                .details
                                .iter()
                                .filter_map(|(k, v)| {
                                    v.parse::<f64>().ok().map(|val| (k.clone(), val))
                                })
                                .collect(),
                        },
                    },
                    trace_id: None,
                });
            }
        }
    }

    /// Attempt to recover a crashed agent
    async fn attempt_recovery(&self, agent_id: &AgentId) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Attempting agent recovery");

        // Get original agent metadata to recreate
        if let Some(metadata) = self.registry.get_metadata(agent_id) {
            // Create new agent config from metadata
            let config = AgentConfig {
                name: format!("{}-recovered", metadata.name),
                agent_type: metadata.agent_type,
                capabilities: metadata.capabilities,
                max_memory: metadata
                    .properties
                    .get("max_memory_bytes")
                    .and_then(|s| s.parse().ok()),
                timeout: metadata
                    .properties
                    .get("timeout_ms")
                    .and_then(|s| s.parse::<u64>().ok())
                    .map(Duration::from_millis),
            };

            // Create replacement agent
            let new_agent_id = self.create_agent(config).await?;
            info!(
                old_agent_id = %agent_id,
                new_agent_id = %new_agent_id,
                "Agent recovery successful"
            );

            Ok(())
        } else {
            Err(CaxtonError::Agent(
                "Cannot recover agent: metadata not found".to_string(),
            ))
        }
    }

    /// Record lifecycle event in monitoring system
    async fn record_lifecycle_event(&self, event: &LifecycleEvent) {
        // Convert lifecycle event to agent event format for monitoring
        let agent_event = match event {
            LifecycleEvent::StateTransition { agent_id, from, to } => Some(AgentEvent {
                agent_id: agent_id.clone(),
                timestamp: std::time::SystemTime::now(),
                event_type: AgentEventType::StateChange {
                    from: from.clone(),
                    to: to.clone(),
                },
                trace_id: None,
            }),
            LifecycleEvent::AgentCrashed { agent_id, error } => Some(AgentEvent {
                agent_id: agent_id.clone(),
                timestamp: std::time::SystemTime::now(),
                event_type: AgentEventType::Crashed(error.clone()),
                trace_id: None,
            }),
            _ => None,
        };

        if let Some(event) = agent_event {
            self.monitor.record_event(&event);
        }
    }
}

/// Comprehensive agent status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentStatusReport {
    pub agent_id: AgentId,
    pub state: AgentState,
    pub metadata: AgentMetadata,
    pub resource_usage: AgentResourceUsage,
    pub health_status: HealthCheckResult,
    pub performance_metrics: Option<AgentPerformanceMetrics>,
    pub last_updated: DateTime<Utc>,
}

/// System-wide lifecycle statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleStats {
    pub total_agents: usize,
    pub agents_by_state: HashMap<AgentState, usize>,
    pub capability_distribution: HashMap<String, usize>,
    pub system_metrics: HashMap<String, String>,
    pub recovery_enabled: bool,
    pub last_updated: DateTime<Utc>,
}
