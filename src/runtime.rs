//! Caxton runtime implementation
//!
//! Complete agent lifecycle management with type-safe state tracking,
//! resource monitoring, and inter-agent communication.

use crate::*;
use arc_swap::ArcSwap;
use dashmap::DashMap;
use parking_lot::{Mutex, RwLock};
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};
use tokio::sync::{mpsc, oneshot, Semaphore};
use tokio::time::{interval, Duration as TokioDuration};
// use futures::StreamExt; // Unused import removed

/// Runtime configuration
#[derive(Debug, Clone)]
pub struct CaxtonConfig {
    pub max_agents: usize,
    pub default_timeout: Duration,
    pub observability_enabled: bool,
    pub fipa_compliance_strict: bool,
    pub wasm_isolation_enabled: bool,
    pub tracing_enabled: bool,
    pub metrics_enabled: bool,
    pub event_emission_enabled: bool,
    pub performance_mode: bool,
    pub resource_limits: ResourceLimits,
}

impl Default for CaxtonConfig {
    fn default() -> Self {
        Self {
            max_agents: 100,
            default_timeout: Duration::from_secs(30),
            observability_enabled: true,
            fipa_compliance_strict: false,
            wasm_isolation_enabled: true,
            tracing_enabled: true,
            metrics_enabled: true,
            event_emission_enabled: true,
            performance_mode: false,
            resource_limits: ResourceLimits::default(),
        }
    }
}

/// Resource limits for agents
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_per_agent: u64,
    pub max_cpu_time_per_agent: Duration,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_per_agent: 64 * 1024 * 1024, // 64MB
            max_cpu_time_per_agent: Duration::from_secs(30),
        }
    }
}

/// Agent resource usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResourceUsage {
    pub memory_bytes: u64,
    pub cpu_time_ms: u64,
    pub message_count: u64,
    pub last_activity: DateTime<Utc>,
}

/// Agent instance with runtime state
#[derive(Debug)]
struct AgentInstance {
    pub metadata: AgentMetadata,
    pub resource_usage: AgentResourceUsage,
    pub message_tx: mpsc::UnboundedSender<FipaMessage>,
    pub shutdown_tx: Option<oneshot::Sender<()>>,
    pub wasm_handle: Option<tokio::task::JoinHandle<()>>,
}

/// Message routing entry
#[derive(Debug, Clone)]
struct MessageRoute {
    pub agent_id: AgentId,
    pub capabilities: Vec<String>,
    pub message_tx: mpsc::UnboundedSender<FipaMessage>,
}

/// Main Caxton runtime with complete lifecycle management
#[derive(Clone)]
pub struct CaxtonRuntime {
    config: CaxtonConfig,
    agents: Arc<DashMap<AgentId, AgentInstance>>,
    message_routes: Arc<DashMap<AgentId, MessageRoute>>,
    global_metrics: Arc<GlobalMetrics>,
    spawn_semaphore: Arc<Semaphore>,
    event_tx: mpsc::UnboundedSender<AgentEvent>,
    shutdown_signal: Arc<ArcSwap<bool>>,
}

/// Global runtime metrics
#[derive(Debug, Default)]
struct GlobalMetrics {
    pub total_agents_spawned: AtomicU64,
    pub active_agents: AtomicU64,
    pub total_messages_processed: AtomicU64,
    pub total_memory_used: AtomicU64,
}

impl CaxtonRuntime {
    /// Create a new Caxton runtime with complete lifecycle management
    #[instrument(name = "runtime_new", skip(config))]
    pub async fn new(config: CaxtonConfig) -> Result<Self, CaxtonError> {
        info!("Initializing Caxton runtime with config: {:?}", config);

        let (event_tx, mut event_rx) = mpsc::unbounded_channel();
        let shutdown_signal = Arc::new(ArcSwap::from_pointee(false));

        let runtime = Self {
            spawn_semaphore: Arc::new(Semaphore::new(config.max_agents)),
            agents: Arc::new(DashMap::new()),
            message_routes: Arc::new(DashMap::new()),
            global_metrics: Arc::new(GlobalMetrics::default()),
            event_tx,
            shutdown_signal: shutdown_signal.clone(),
            config,
        };

        // Start background tasks
        let runtime_clone = runtime.clone();
        tokio::spawn(async move {
            runtime_clone.run_metrics_collector().await;
        });

        let runtime_clone = runtime.clone();
        let shutdown_clone = shutdown_signal.clone();
        tokio::spawn(async move {
            runtime_clone
                .run_event_processor(event_rx, shutdown_clone)
                .await;
        });

        info!("Caxton runtime initialized successfully");
        Ok(runtime)
    }

    /// Spawn a new agent with complete lifecycle management
    #[instrument(name = "spawn_agent", skip(self, config), fields(agent_name = %config.name))]
    pub async fn spawn_agent(&self, config: AgentConfig) -> Result<AgentId, CaxtonError> {
        // Acquire semaphore permit to limit concurrent agents
        let _permit = self
            .spawn_semaphore
            .acquire()
            .await
            .map_err(|_| CaxtonError::Runtime("Failed to acquire spawn permit".to_string()))?;

        let agent_id = AgentId::new();
        info!(agent_id = %agent_id, "Spawning agent: {}", config.name);

        // Create agent metadata
        let mut metadata = AgentMetadata::new(config.name.clone(), config.agent_type.clone());
        for capability in &config.capabilities {
            metadata.add_capability(capability);
        }

        // Set resource limits in properties
        if let Some(max_memory) = config.max_memory {
            metadata.set_property("max_memory", &max_memory.to_string());
        }
        if let Some(timeout) = config.timeout {
            metadata.set_property("timeout_ms", &timeout.as_millis().to_string());
        }

        // Create message channels
        let (message_tx, message_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Initialize resource usage tracking
        let resource_usage = AgentResourceUsage {
            memory_bytes: 0,
            cpu_time_ms: 0,
            message_count: 0,
            last_activity: Utc::now(),
        };

        // Create agent instance
        let agent_instance = AgentInstance {
            metadata: metadata.clone(),
            resource_usage,
            message_tx: message_tx.clone(),
            shutdown_tx: Some(shutdown_tx),
            wasm_handle: None,
        };

        // Start agent task
        let agent_task = self
            .start_agent_task(agent_id.clone(), config, message_rx, shutdown_rx)
            .await?;

        // Update agent instance with task handle
        let mut agent_instance = agent_instance;
        agent_instance.wasm_handle = Some(agent_task);

        // Register agent
        self.agents.insert(agent_id.clone(), agent_instance);

        // Register message route
        let route = MessageRoute {
            agent_id: agent_id.clone(),
            capabilities: metadata.capabilities.clone(),
            message_tx,
        };
        self.message_routes.insert(agent_id.clone(), route);

        // Emit agent spawned event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::StateChange {
                from: AgentState::Initializing,
                to: AgentState::Ready,
            },
            trace_id: None,
        });

        // Update metrics
        self.global_metrics
            .total_agents_spawned
            .fetch_add(1, Ordering::Relaxed);
        self.global_metrics
            .active_agents
            .fetch_add(1, Ordering::Relaxed);

        info!(agent_id = %agent_id, "Agent spawned successfully");
        Ok(agent_id)
    }

    /// Terminate an agent with graceful shutdown
    #[instrument(name = "terminate_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn terminate_agent(
        &self,
        agent_id: &AgentId,
        timeout: Duration,
    ) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Terminating agent with timeout: {:?}", timeout);

        // Get agent instance
        let mut agent_instance = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        // Update state to terminating
        agent_instance.metadata.set_state(AgentState::Terminating);

        // Send shutdown signal
        if let Some(shutdown_tx) = agent_instance.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        // Wait for task completion with timeout
        if let Some(handle) = agent_instance.wasm_handle.take() {
            match tokio::time::timeout(timeout, handle).await {
                Ok(Ok(())) => {
                    info!(agent_id = %agent_id, "Agent terminated gracefully");
                }
                Ok(Err(e)) => {
                    warn!(agent_id = %agent_id, "Agent task panicked: {:?}", e);
                }
                Err(_) => {
                    warn!(agent_id = %agent_id, "Agent termination timed out, forcing shutdown");
                    // Task handle is dropped, which should abort the task
                }
            }
        }

        // Update final state
        agent_instance.metadata.set_state(AgentState::Terminated);

        // Emit termination event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::StateChange {
                from: AgentState::Terminating,
                to: AgentState::Terminated,
            },
            trace_id: None,
        });

        // Clean up routes
        self.message_routes.remove(agent_id);

        // Update metrics
        self.global_metrics
            .active_agents
            .fetch_sub(1, Ordering::Relaxed);

        info!(agent_id = %agent_id, "Agent termination complete");
        Ok(())
    }

    /// Send a message to an agent with routing and validation
    #[instrument(name = "send_message", skip(self, message), fields(receiver = %message.receiver))]
    pub async fn send_message(&self, message: FipaMessage) -> Result<FipaMessage, CaxtonError> {
        validate_fipa_message(&message)?;

        debug!("Sending message to agent: {}", message.receiver);

        // Find message route for target agent
        let route = self
            .message_routes
            .get(&message.receiver)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", message.receiver)))?;

        // Send message to agent
        route
            .message_tx
            .send(message.clone())
            .map_err(|_| CaxtonError::Runtime("Failed to send message to agent".to_string()))?;

        // Update metrics
        self.global_metrics
            .total_messages_processed
            .fetch_add(1, Ordering::Relaxed);

        // Update agent's last activity and message count
        if let Some(mut agent) = self.agents.get_mut(&message.receiver) {
            agent.resource_usage.message_count += 1;
            agent.resource_usage.last_activity = Utc::now();
        }

        // Emit message sent event
        self.emit_agent_event(AgentEvent {
            agent_id: message.receiver.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::MessageReceived(message.clone()),
            trace_id: None,
        });

        debug!("Message sent successfully to agent: {}", message.receiver);

        // For now, return an acknowledgment (in real implementation, this might wait for response)
        Ok(FipaMessage {
            performative: FipaPerformative::Inform,
            sender: message.receiver,
            receiver: message.sender,
            content: serde_json::json!({"status": "received"}),
            conversation_id: message.conversation_id,
            in_reply_to: message.reply_with,
            ..Default::default()
        })
    }

    pub async fn receive_message(&self) -> Option<FipaMessage> {
        // Stub implementation
        None
    }

    /// Get current agent state and metadata
    #[instrument(name = "get_agent_state", skip(self), fields(agent_id = %agent_id))]
    pub async fn get_agent_state(&self, agent_id: &AgentId) -> Result<AgentState, CaxtonError> {
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        Ok(agent.metadata.state.clone())
    }

    /// Suspend an agent (pause execution without termination)
    #[instrument(name = "suspend_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn suspend_agent(&self, agent_id: &AgentId) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Suspending agent");

        let mut agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        let current_state = agent.metadata.state.clone();
        if current_state == AgentState::Terminated || current_state == AgentState::Terminating {
            return Err(CaxtonError::Agent(format!(
                "Cannot suspend agent in state: {:?}",
                current_state
            )));
        }

        agent.metadata.set_state(AgentState::Suspended);

        // Emit suspension event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::StateChange {
                from: current_state,
                to: AgentState::Suspended,
            },
            trace_id: None,
        });

        info!(agent_id = %agent_id, "Agent suspended successfully");
        Ok(())
    }

    /// Resume a suspended agent
    #[instrument(name = "resume_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn resume_agent(&self, agent_id: &AgentId) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Resuming agent");

        let mut agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        if agent.metadata.state != AgentState::Suspended {
            return Err(CaxtonError::Agent(format!(
                "Agent is not suspended: {:?}",
                agent.metadata.state
            )));
        }

        agent.metadata.set_state(AgentState::Ready);

        // Emit resume event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::StateChange {
                from: AgentState::Suspended,
                to: AgentState::Ready,
            },
            trace_id: None,
        });

        info!(agent_id = %agent_id, "Agent resumed successfully");
        Ok(())
    }

    /// Force terminate an agent (immediate shutdown)
    #[instrument(name = "force_terminate_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn force_terminate_agent(&self, agent_id: &AgentId) -> Result<(), CaxtonError> {
        warn!(agent_id = %agent_id, "Force terminating agent (immediate shutdown)");

        // Get and remove agent instance immediately
        let (_, mut agent_instance) = self
            .agents
            .remove(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        // Force abort the task if it exists
        if let Some(handle) = agent_instance.wasm_handle.take() {
            handle.abort();
        }

        // Update final state
        agent_instance.metadata.set_state(AgentState::Terminated);

        // Emit force termination event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::StateChange {
                from: AgentState::Terminating,
                to: AgentState::Terminated,
            },
            trace_id: None,
        });

        // Clean up routes
        self.message_routes.remove(agent_id);

        // Update metrics
        self.global_metrics
            .active_agents
            .fetch_sub(1, Ordering::Relaxed);

        warn!(agent_id = %agent_id, "Agent force terminated");
        Ok(())
    }

    /// Get comprehensive resource usage for an agent
    #[instrument(name = "get_agent_resource_usage", skip(self), fields(agent_id = %agent_id))]
    pub async fn get_agent_resource_usage(
        &self,
        agent_id: &AgentId,
    ) -> Result<AgentResourceUsage, CaxtonError> {
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        Ok(agent.resource_usage.clone())
    }

    /// Update agent resource limits dynamically
    #[instrument(name = "update_agent_limits", skip(self), fields(agent_id = %agent_id))]
    pub async fn update_agent_limits(
        &self,
        agent_id: &AgentId,
        max_memory: Option<u64>,
        max_cpu_time: Option<Duration>,
    ) -> Result<(), CaxtonError> {
        info!(agent_id = %agent_id, "Updating agent resource limits");

        let mut agent = self
            .agents
            .get_mut(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        if let Some(memory_limit) = max_memory {
            agent
                .metadata
                .set_property("max_memory", &memory_limit.to_string());
        }

        if let Some(cpu_limit) = max_cpu_time {
            agent
                .metadata
                .set_property("max_cpu_time_ms", &cpu_limit.as_millis().to_string());
        }

        info!(agent_id = %agent_id, "Agent limits updated successfully");
        Ok(())
    }

    /// Perform health check on a specific agent
    #[instrument(name = "health_check_agent", skip(self), fields(agent_id = %agent_id))]
    pub async fn health_check_agent(
        &self,
        agent_id: &AgentId,
    ) -> Result<HealthStatus, CaxtonError> {
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        let now = Utc::now();
        let last_activity = agent.resource_usage.last_activity;
        let inactive_duration = now.signed_duration_since(last_activity).num_seconds();

        let healthy = match agent.metadata.state {
            AgentState::Ready | AgentState::Processing => {
                inactive_duration < 300 // Healthy if active within 5 minutes
            }
            AgentState::Suspended => true, // Suspended agents are considered healthy
            _ => false,
        };

        let mut details = None;
        if !healthy {
            details = Some(format!(
                "Agent inactive for {} seconds in state {:?}",
                inactive_duration, agent.metadata.state
            ));
        }

        let mut metrics = HashMap::new();
        metrics.insert(
            "memory_bytes".to_string(),
            agent.resource_usage.memory_bytes as f64,
        );
        metrics.insert(
            "cpu_time_ms".to_string(),
            agent.resource_usage.cpu_time_ms as f64,
        );
        metrics.insert(
            "message_count".to_string(),
            agent.resource_usage.message_count as f64,
        );
        metrics.insert("inactive_seconds".to_string(), inactive_duration as f64);

        let health_status = HealthStatus {
            healthy,
            timestamp: now,
            details,
            metrics,
        };

        // Emit health check event
        self.emit_agent_event(AgentEvent {
            agent_id: agent_id.clone(),
            timestamp: std::time::SystemTime::now(),
            event_type: AgentEventType::HealthCheck {
                status: health_status.clone(),
            },
            trace_id: None,
        });

        Ok(health_status)
    }

    /// Get agent metadata including resource usage
    #[instrument(name = "get_agent_metadata", skip(self), fields(agent_id = %agent_id))]
    pub async fn get_agent_metadata(
        &self,
        agent_id: &AgentId,
    ) -> Result<(AgentMetadata, AgentResourceUsage), CaxtonError> {
        let agent = self
            .agents
            .get(agent_id)
            .ok_or_else(|| CaxtonError::Agent(format!("Agent not found: {}", agent_id)))?;

        Ok((agent.metadata.clone(), agent.resource_usage.clone()))
    }

    /// List all active agents with their capabilities
    #[instrument(name = "list_agents", skip(self))]
    pub async fn list_agents(&self) -> Vec<(AgentId, Vec<String>)> {
        self.message_routes
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().capabilities.clone()))
            .collect()
    }

    /// Get global runtime metrics
    pub fn get_metrics(&self) -> (u64, u64, u64, u64) {
        (
            self.global_metrics
                .total_agents_spawned
                .load(Ordering::Relaxed),
            self.global_metrics.active_agents.load(Ordering::Relaxed),
            self.global_metrics
                .total_messages_processed
                .load(Ordering::Relaxed),
            self.global_metrics
                .total_memory_used
                .load(Ordering::Relaxed),
        )
    }

    /// Gracefully shutdown the runtime
    #[instrument(name = "shutdown", skip(self))]
    pub async fn shutdown(&self, timeout: Duration) -> Result<(), CaxtonError> {
        info!("Shutting down Caxton runtime");

        // Signal shutdown
        self.shutdown_signal.store(Arc::new(true));

        // Collect all agent IDs
        let agent_ids: Vec<AgentId> = self
            .agents
            .iter()
            .map(|entry| entry.key().clone())
            .collect();

        // Terminate all agents with timeout
        let per_agent_timeout = timeout / agent_ids.len().max(1) as u32;

        for agent_id in agent_ids {
            if let Err(e) = self.terminate_agent(&agent_id, per_agent_timeout).await {
                warn!("Failed to terminate agent {}: {:?}", agent_id, e);
            }
        }

        info!("Caxton runtime shutdown complete");
        Ok(())
    }

    /// Emit an agent event to the event processing system
    fn emit_agent_event(&self, event: AgentEvent) {
        if let Err(_) = self.event_tx.send(event) {
            warn!("Failed to emit agent event - event channel closed");
        }
    }

    /// Start an individual agent task with message processing
    async fn start_agent_task(
        &self,
        agent_id: AgentId,
        config: AgentConfig,
        mut message_rx: mpsc::UnboundedReceiver<FipaMessage>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<tokio::task::JoinHandle<()>, CaxtonError> {
        let event_tx = self.event_tx.clone();
        let agents = self.agents.clone();

        let handle = tokio::spawn(async move {
            info!(agent_id = %agent_id, "Agent task started: {}", config.name);

            loop {
                tokio::select! {
                    // Handle incoming messages
                    message = message_rx.recv() => {
                        match message {
                            Some(msg) => {
                                debug!(agent_id = %agent_id, "Processing message: {:?}", msg.performative);

                                // Update agent state to processing
                                if let Some(mut agent) = agents.get_mut(&agent_id) {
                                    agent.metadata.set_state(AgentState::Processing);
                                    agent.resource_usage.last_activity = Utc::now();
                                }

                                // Process message (in real implementation, this would invoke WASM)
                                // For now, just log the message processing
                                tokio::time::sleep(TokioDuration::from_millis(10)).await;

                                // Update agent state back to ready
                                if let Some(mut agent) = agents.get_mut(&agent_id) {
                                    agent.metadata.set_state(AgentState::Ready);
                                }
                            }
                            None => {
                                warn!(agent_id = %agent_id, "Message channel closed");
                                break;
                            }
                        }
                    }

                    // Handle shutdown signal
                    _ = &mut shutdown_rx => {
                        info!(agent_id = %agent_id, "Agent received shutdown signal");
                        break;
                    }
                }
            }

            info!(agent_id = %agent_id, "Agent task completed");
        });

        Ok(handle)
    }

    /// Background task for collecting system and agent metrics
    async fn run_metrics_collector(&self) {
        let mut interval = interval(TokioDuration::from_secs(30));
        let mut sys = sysinfo::System::new_all();

        while !**self.shutdown_signal.load() {
            interval.tick().await;

            // Refresh system information
            sys.refresh_all();

            // Collect agent resource metrics
            let mut total_memory: u64 = 0;
            let mut unhealthy_agents = 0;
            let now = Utc::now();

            for mut agent_entry in self.agents.iter_mut() {
                let agent = agent_entry.value_mut();
                let agent_id = agent_entry.key();

                // Update resource usage from system (simplified)
                // In real implementation, this would track per-process/WASM instance
                let estimated_memory = 1024 * 1024; // 1MB base per agent
                agent.resource_usage.memory_bytes = estimated_memory;
                total_memory += estimated_memory;

                // Check agent health based on activity
                let inactive_duration = now
                    .signed_duration_since(agent.resource_usage.last_activity)
                    .num_seconds();
                if inactive_duration > 600 {
                    // 10 minutes inactive
                    unhealthy_agents += 1;

                    // Emit resource usage event
                    self.emit_agent_event(AgentEvent {
                        agent_id: agent_id.clone(),
                        timestamp: std::time::SystemTime::now(),
                        event_type: AgentEventType::ResourceUsage {
                            memory_mb: agent.resource_usage.memory_bytes / 1024 / 1024,
                            cpu_percent: 0.0, // Would be calculated from actual process data
                        },
                        trace_id: None,
                    });
                }
            }

            // Update global metrics
            self.global_metrics
                .total_memory_used
                .store(total_memory, Ordering::Relaxed);

            // Log comprehensive metrics
            let (spawned, active, messages, memory) = self.get_metrics();
            info!(
                "Runtime metrics - Spawned: {}, Active: {}, Messages: {}, Memory: {} MB, Unhealthy: {}",
                spawned, active, messages, memory / 1024 / 1024, unhealthy_agents
            );

            // Log system-wide resource usage
            debug!(
                "System metrics - Total memory: {} MB, CPU count: {}",
                sys.total_memory() / 1024 / 1024,
                sys.cpus().len()
            );
        }
    }

    /// Background task for processing agent events
    async fn run_event_processor(
        &self,
        mut event_rx: mpsc::UnboundedReceiver<AgentEvent>,
        shutdown_signal: Arc<ArcSwap<bool>>,
    ) {
        while !**shutdown_signal.load() {
            match event_rx.recv().await {
                Some(event) => {
                    // Process agent event (logging, metrics, external notifications, etc.)
                    match &event.event_type {
                        AgentEventType::StateChange { from, to } => {
                            info!(
                                agent_id = %event.agent_id,
                                "Agent state transition: {:?} -> {:?}",
                                from, to
                            );
                        }
                        AgentEventType::MessageReceived(msg) => {
                            debug!(
                                agent_id = %event.agent_id,
                                "Agent received message: {:?}",
                                msg.performative
                            );
                        }
                        AgentEventType::MessageSent(msg) => {
                            debug!(
                                agent_id = %event.agent_id,
                                "Agent sent message: {:?}",
                                msg.performative
                            );
                        }
                        AgentEventType::Crashed(reason) => {
                            error!(
                                agent_id = %event.agent_id,
                                "Agent crashed: {}",
                                reason
                            );
                        }
                    }
                }
                None => {
                    warn!("Event channel closed");
                    break;
                }
            }
        }
    }
}
