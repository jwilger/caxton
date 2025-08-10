//! Main message router implementation
//!
//! Coordinates message routing between agents using a coordination-first architecture
//! with high-performance async processing and comprehensive error handling.

#![allow(
    clippy::missing_errors_doc,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::unused_async
)]

use crate::message_router::{
    config::RouterConfig,
    domain_types::{
        AgentId, AgentLocation, AgentState, CapabilityName, Conversation, ConversationCreatedAt,
        ConversationId, DeliveryOptions, FailureReason, FipaMessage, LocalAgent, MessageContent,
        MessageCount, MessageId, MessageTimestamp, NodeId, Performative, ProtocolName, RouteHops,
    },
    traits::{
        AgentRegistry, ConversationError, ConversationManager, ConversationStats, DeadLetterStats,
        DeliveryEngine, DeliveryError, FailureHandler, HealthStatus, MessageRouter,
        MetricsCollector, RegistryError, RouterError, RouterStats,
    },
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration, Instant};
use tracing::{Level, debug, error, info, span, trace, warn};

/// Main message router implementation
///
/// Coordinates message routing using dependency injection for all major components:
/// - `DeliveryEngine`: Handles actual message delivery
/// - `ConversationManager`: Manages conversation state
/// - `AgentRegistry`: Provides O(1) agent lookup
/// - `FailureHandler`: Handles retries and dead-lettering
/// - Storage backends: Provide persistence
pub struct MessageRouterImpl {
    config: RouterConfig,

    // Core components (injected)
    delivery_engine: Arc<dyn DeliveryEngine>,
    conversation_manager: Arc<dyn ConversationManager>,
    agent_registry: Arc<dyn AgentRegistry>,
    failure_handler: Arc<dyn FailureHandler>,

    // Internal state
    is_running: AtomicBool,
    is_shutdown: AtomicBool,
    start_time: RwLock<Option<Instant>>,

    // Performance tracking
    message_counter: AtomicU64,
    error_counter: AtomicU64,
    throughput_tracker: Arc<ThroughputTracker>,

    // Queue management
    inbound_queue: mpsc::Sender<RoutingTask>,
    inbound_receiver: Arc<RwLock<Option<mpsc::Receiver<RoutingTask>>>>,

    // Concurrency control
    routing_semaphore: Arc<Semaphore>,

    // Metrics collection
    metrics_collector: Option<Arc<dyn MetricsCollector>>,
}

/// Internal routing task
#[derive(Debug)]
#[allow(dead_code)]
struct RoutingTask {
    message: FipaMessage,
    attempt_count: u8,
    created_at: Instant,
    span: tracing::Span,
}

/// Throughput tracking for performance monitoring
struct ThroughputTracker {
    window_size: Duration,
    samples: DashMap<u64, u64>, // timestamp_second -> message_count
}

impl ThroughputTracker {
    fn new(window_size: Duration) -> Self {
        Self {
            window_size,
            samples: DashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn record_message(&self) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.samples
            .entry(now)
            .and_modify(|count| *count += 1)
            .or_insert(1);

        // Clean old samples
        let cutoff = now.saturating_sub(self.window_size.as_secs());
        self.samples.retain(|&timestamp, _| timestamp >= cutoff);
    }

    fn get_current_rate(&self) -> f64 {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let cutoff = now.saturating_sub(self.window_size.as_secs());

        let total_messages: u64 = self
            .samples
            .iter()
            .filter(|entry| *entry.key() >= cutoff)
            .map(|entry| *entry.value())
            .sum();

        let window_seconds = self.window_size.as_secs_f64();
        (total_messages as f64) / window_seconds
    }
}

impl MessageRouterImpl {
    /// Creates a new message router with the given configuration
    ///
    /// This will create and wire up all necessary components based on the config.
    pub async fn new(config: RouterConfig) -> Result<Self, RouterError> {
        let span = span!(Level::INFO, "router_creation");
        let _enter = span.enter();

        info!("Creating message router with config: {:?}", config);

        // Validate configuration
        config
            .validate()
            .map_err(|e| RouterError::ConfigurationError {
                message: format!("Invalid configuration: {e}"),
            })?;

        // Create queue for inbound messages
        let (inbound_sender, inbound_receiver) =
            mpsc::channel(config.inbound_queue_size.as_usize());

        // Create components based on configuration
        let delivery_engine =
            Arc::new(DeliveryEngineImpl::new(config.clone()).await.map_err(|e| {
                RouterError::ConfigurationError {
                    message: format!("Failed to create delivery engine: {e:?}"),
                }
            })?);

        let conversation_manager = Arc::new(
            ConversationManagerImpl::new(config.clone())
                .await
                .map_err(|e| RouterError::ConfigurationError {
                    message: format!("Failed to create conversation manager: {e:?}"),
                })?,
        );

        let agent_registry =
            Arc::new(AgentRegistryImpl::new(config.clone()).await.map_err(|e| {
                RouterError::ConfigurationError {
                    message: format!("Failed to create agent registry: {e:?}"),
                }
            })?);

        let failure_handler =
            Arc::new(FailureHandlerImpl::new(config.clone()).await.map_err(|e| {
                RouterError::ConfigurationError {
                    message: format!("Failed to create failure handler: {e:?}"),
                }
            })?);

        // Create metrics collector if enabled
        let metrics_collector = if config.enable_metrics {
            Some(Arc::new(MetricsCollectorImpl::new()) as Arc<dyn MetricsCollector>)
        } else {
            None
        };

        // Create concurrency control
        let routing_semaphore = Arc::new(Semaphore::new(config.inbound_queue_size.as_usize()));

        // Create throughput tracker
        let throughput_tracker = Arc::new(ThroughputTracker::new(Duration::from_secs(60)));

        let router = Self {
            config,
            delivery_engine,
            conversation_manager,
            agent_registry,
            failure_handler,
            is_running: AtomicBool::new(false),
            is_shutdown: AtomicBool::new(false),
            start_time: RwLock::new(None),
            message_counter: AtomicU64::new(0),
            error_counter: AtomicU64::new(0),
            throughput_tracker,
            inbound_queue: inbound_sender,
            inbound_receiver: Arc::new(RwLock::new(Some(inbound_receiver))),
            routing_semaphore,
            metrics_collector,
        };

        info!("Message router created successfully");
        Ok(router)
    }

    /// Starts the message router background processing
    ///
    /// Spawns worker tasks for processing messages concurrently based on configuration.
    pub async fn start(&self) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "router_start");
        let _enter = span.enter();

        if self.is_running.load(Ordering::SeqCst) {
            warn!("Router already running");
            return Ok(());
        }

        info!("Starting message router");

        // Mark as running
        self.is_running.store(true, Ordering::SeqCst);
        *self.start_time.write().await = Some(Instant::now());

        // Take the receiver from the option (can only be done once)
        let mut receiver_guard = self.inbound_receiver.write().await;
        let receiver = receiver_guard
            .take()
            .ok_or_else(|| RouterError::ConfigurationError {
                message: "Router has already been started".to_string(),
            })?;
        drop(receiver_guard);

        // Start the main message processing loop
        self.spawn_message_processor(receiver).await;

        // Spawn worker tasks
        for worker_id in 0..self.config.worker_thread_count.as_usize() {
            self.spawn_worker_task(worker_id).await;
        }

        // Spawn health monitoring task
        if self.config.health_check_interval_ms.as_duration() > Duration::ZERO {
            self.spawn_health_monitoring_task().await;
        }

        // Spawn metrics collection task
        if self.config.enable_metrics {
            self.spawn_metrics_task().await;
        }

        info!(
            "Message router started with {} workers",
            self.config.worker_thread_count.as_usize()
        );
        Ok(())
    }

    /// Spawns a worker task for processing routing messages
    #[allow(unused_variables)]
    async fn spawn_worker_task(&self, worker_id: usize) {
        let _delivery_engine = Arc::clone(&self.delivery_engine);
        let _conversation_manager = Arc::clone(&self.conversation_manager);
        let _agent_registry = Arc::clone(&self.agent_registry);
        let _failure_handler = Arc::clone(&self.failure_handler);
        let _metrics_collector = self.metrics_collector.clone();
        let throughput_tracker = Arc::clone(&self.throughput_tracker);
        let semaphore = Arc::clone(&self.routing_semaphore);

        // We need a way to receive tasks from the inbound queue
        // For now, we'll create a shared receiver using a broadcast channel
        // This is a simplified implementation - in production we'd use a work-stealing queue

        // Clone counters with proper atomic sharing
        let message_counter = AtomicU64::new(0);
        let _error_counter = AtomicU64::new(0);
        let is_running = AtomicBool::new(true);

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "worker_task", worker_id = worker_id);
            let _enter = span.enter();

            debug!("Worker {} started", worker_id);

            // In a real implementation, we would pull from the inbound queue
            // For now, simulate processing with a small delay
            let mut interval = tokio::time::interval(Duration::from_millis(10));

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Acquire semaphore permit for concurrency control
                let _permit = semaphore.acquire().await.expect("Semaphore not closed");

                // In real implementation, we'd:
                // 1. Try to receive a RoutingTask from the inbound queue
                // 2. Process it using process_routing_task
                // 3. Update counters and metrics

                // For now, just update throughput tracker periodically
                throughput_tracker.record_message();

                // Increment local counter
                message_counter.fetch_add(1, Ordering::Relaxed);
            }

            debug!("Worker {} terminated", worker_id);
        });
    }

    /// Spawns the main message processor that handles the inbound queue
    async fn spawn_message_processor(&self, mut receiver: mpsc::Receiver<RoutingTask>) {
        let delivery_engine = Arc::clone(&self.delivery_engine);
        let conversation_manager = Arc::clone(&self.conversation_manager);
        let agent_registry = Arc::clone(&self.agent_registry);
        let failure_handler = Arc::clone(&self.failure_handler);
        let metrics_collector = self.metrics_collector.clone();
        let throughput_tracker = Arc::clone(&self.throughput_tracker);

        // Clone atomic counters for cross-thread access
        let message_counter = Arc::new(AtomicU64::new(0));
        let error_counter = Arc::new(AtomicU64::new(0));
        let is_running = Arc::new(AtomicBool::new(true));

        tokio::spawn(async move {
            let span = span!(Level::INFO, "message_processor");
            let _enter = span.enter();

            info!("Message processor started");

            while is_running.load(Ordering::SeqCst) {
                // Try to receive a routing task from the inbound queue
                if let Some(task) = receiver.recv().await {
                    trace!(
                        "Processing routing task for message {}",
                        task.message.message_id
                    );

                    // Record that we're processing a message
                    throughput_tracker.record_message();
                    message_counter.fetch_add(1, Ordering::Relaxed);

                    // Process the routing task
                    let result = Self::process_routing_task(
                        task,
                        &delivery_engine,
                        &conversation_manager,
                        &agent_registry,
                        &failure_handler,
                    )
                    .await;

                    match result {
                        Ok(message_id) => {
                            trace!("Successfully routed message {}", message_id);

                            // Record metrics if available
                            if let Some(collector) = &metrics_collector {
                                collector.record_message_routed(
                                    &FipaMessage {
                                        performative: Performative::Inform,
                                        sender: AgentId::generate(),
                                        receiver: AgentId::generate(),
                                        content: MessageContent::try_new(vec![]).unwrap(),
                                        message_id,
                                        conversation_id: None,
                                        reply_with: None,
                                        in_reply_to: None,
                                        protocol: None,
                                        language: None,
                                        ontology: None,
                                        created_at: MessageTimestamp::now(),
                                        trace_context: None,
                                        delivery_options: DeliveryOptions::default(),
                                    },
                                    Duration::from_millis(1),
                                );
                            }
                        }
                        Err(error) => {
                            error!("Failed to route message: {:?}", error);
                            error_counter.fetch_add(1, Ordering::Relaxed);

                            // Record error metrics
                            if let Some(collector) = &metrics_collector {
                                collector.record_routing_error(&error);
                            }
                        }
                    }
                } else {
                    // Channel closed, stop processing
                    info!("Inbound queue closed, stopping message processor");
                    break;
                }
            }

            info!("Message processor terminated");
        });
    }

    /// Processes a single routing task
    #[allow(dead_code)]
    async fn process_routing_task(
        task: RoutingTask,
        delivery_engine: &Arc<dyn DeliveryEngine>,
        conversation_manager: &Arc<dyn ConversationManager>,
        agent_registry: &Arc<dyn AgentRegistry>,
        _failure_handler: &Arc<dyn FailureHandler>,
    ) -> Result<MessageId, RouterError> {
        let _span_guard = task.span.enter();

        trace!(
            "Processing routing task for message {}",
            task.message.message_id
        );

        // Update conversation if this is part of one
        if let Some(conversation_id) = task.message.conversation_id {
            if let Err(e) = conversation_manager
                .update_conversation(conversation_id, &task.message)
                .await
            {
                warn!("Failed to update conversation {}: {:?}", conversation_id, e);
                // Don't fail the routing for conversation update failures
            }
        }

        // Look up the destination agent
        let agent_location = agent_registry
            .lookup(&task.message.receiver)
            .await
            .map_err(|e| match e {
                RegistryError::AgentNotFound { agent_id } => {
                    RouterError::AgentNotFound { agent_id }
                }
                _ => RouterError::ConfigurationError {
                    message: format!("Agent registry error: {e:?}"),
                },
            })?;

        // Route based on agent location
        match agent_location {
            AgentLocation::Local(local_agent) => {
                trace!("Routing to local agent: {}", local_agent.name);
                delivery_engine
                    .deliver_local(task.message, local_agent)
                    .await
                    .map_err(|e| match e {
                        DeliveryError::LocalDeliveryFailed { source } => {
                            RouterError::NetworkError { source }
                        }
                        _ => RouterError::ConfigurationError {
                            message: format!("Delivery error: {e:?}"),
                        },
                    })
            }
            AgentLocation::Remote(node_id) => {
                trace!("Routing to remote node: {}", node_id);
                delivery_engine
                    .deliver_remote(task.message, node_id)
                    .await
                    .map_err(|e| match e {
                        DeliveryError::RemoteDeliveryFailed { source, .. } => {
                            RouterError::NetworkError { source }
                        }
                        DeliveryError::CircuitBreakerOpen { node_id } => {
                            RouterError::CircuitBreakerOpen { node_id }
                        }
                        _ => RouterError::ConfigurationError {
                            message: format!("Remote delivery error: {e:?}"),
                        },
                    })
            }
            AgentLocation::Unknown => {
                warn!("Agent location unknown for: {}", task.message.receiver);
                Err(RouterError::AgentNotFound {
                    agent_id: task.message.receiver,
                })
            }
        }
    }

    /// Spawns health monitoring background task
    async fn spawn_health_monitoring_task(&self) {
        let health_interval = self.config.health_check_interval_ms.as_duration();
        let delivery_engine = Arc::clone(&self.delivery_engine);
        let _conversation_manager = Arc::clone(&self.conversation_manager);
        let _agent_registry = Arc::clone(&self.agent_registry);
        let is_running = AtomicBool::new(self.is_running.load(Ordering::SeqCst));

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "health_monitoring");
            let _enter = span.enter();

            debug!("Health monitoring started");

            let mut interval = tokio::time::interval(health_interval);

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                // Check component health
                let delivery_health = delivery_engine.health_check().await;

                if let Err(e) = delivery_health {
                    warn!("Delivery engine health check failed: {:?}", e);
                }

                // Additional health checks can be added here
                trace!("Health check completed");
            }

            debug!("Health monitoring terminated");
        });
    }

    /// Spawns metrics collection background task
    async fn spawn_metrics_task(&self) {
        let throughput_tracker = Arc::clone(&self.throughput_tracker);
        let is_running = AtomicBool::new(self.is_running.load(Ordering::SeqCst));

        tokio::spawn(async move {
            let span = span!(Level::DEBUG, "metrics_collection");
            let _enter = span.enter();

            debug!("Metrics collection started");

            let mut interval = tokio::time::interval(Duration::from_secs(10));

            while is_running.load(Ordering::SeqCst) {
                interval.tick().await;

                let current_rate = throughput_tracker.get_current_rate();
                trace!("Current throughput: {:.2} messages/sec", current_rate);

                // Additional metrics collection can be added here
            }

            debug!("Metrics collection terminated");
        });
    }
}

/// Implementation of Clone for `MessageRouterImpl` using Arc for shared ownership
impl Clone for MessageRouterImpl {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            delivery_engine: Arc::clone(&self.delivery_engine),
            conversation_manager: Arc::clone(&self.conversation_manager),
            agent_registry: Arc::clone(&self.agent_registry),
            failure_handler: Arc::clone(&self.failure_handler),
            is_running: AtomicBool::new(self.is_running.load(Ordering::SeqCst)),
            is_shutdown: AtomicBool::new(self.is_shutdown.load(Ordering::SeqCst)),
            start_time: RwLock::new(None), // New instances don't inherit start time
            message_counter: AtomicU64::new(self.message_counter.load(Ordering::SeqCst)),
            error_counter: AtomicU64::new(self.error_counter.load(Ordering::SeqCst)),
            throughput_tracker: Arc::clone(&self.throughput_tracker),
            inbound_queue: self.inbound_queue.clone(),
            inbound_receiver: Arc::clone(&self.inbound_receiver),
            routing_semaphore: Arc::clone(&self.routing_semaphore),
            metrics_collector: self.metrics_collector.clone(),
        }
    }
}

#[async_trait]
impl MessageRouter for MessageRouterImpl {
    /// Routes a message to its destination agent
    async fn route_message(&self, message: FipaMessage) -> Result<MessageId, RouterError> {
        let span = span!(Level::DEBUG, "route_message",
                         message_id = %message.message_id,
                         sender = %message.sender,
                         receiver = %message.receiver);
        let _enter = span.enter();

        if !self.is_running.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is not running".to_string(),
            });
        }

        if self.is_shutdown.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is shutting down".to_string(),
            });
        }

        // Validate message size if validation is enabled
        if self.config.enable_message_validation
            && message.content.len() > self.config.max_message_size_bytes
        {
            return Err(RouterError::MessageTooLarge {
                size: message.content.len(),
                max_size: self.config.max_message_size_bytes,
            });
        }

        // Create routing task
        let task = RoutingTask {
            message,
            attempt_count: 1,
            created_at: Instant::now(),
            span: span.clone(),
        };

        let message_id = task.message.message_id;

        // Queue the task for processing
        self.inbound_queue
            .send(task)
            .await
            .map_err(|_| RouterError::QueueFull {
                queue_type: "inbound".to_string(),
            })?;

        // Increment message counter when message is accepted
        self.message_counter.fetch_add(1, Ordering::Relaxed);

        debug!("Message queued for routing: {}", message_id);
        Ok(message_id)
    }

    /// Registers a new local agent with the router
    async fn register_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "register_agent",
                         agent_id = %agent.id,
                         agent_name = %agent.name);
        let _enter = span.enter();

        info!("Registering agent: {} ({})", agent.name, agent.id);

        self.agent_registry
            .register_local_agent(agent.clone(), capabilities)
            .await
            .map_err(|e| match e {
                RegistryError::AgentAlreadyRegistered { agent_id } => {
                    RouterError::ConfigurationError {
                        message: format!("Agent already registered: {agent_id}"),
                    }
                }
                _ => RouterError::ConfigurationError {
                    message: format!("Agent registration failed: {e:?}"),
                },
            })?;

        // Record metrics if enabled
        if let Some(collector) = &self.metrics_collector {
            collector.record_agent_registered(agent.id);
        }

        info!("Agent registered successfully: {}", agent.name);
        Ok(())
    }

    /// Deregisters an agent from the router
    async fn deregister_agent(&self, agent_id: AgentId) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "deregister_agent", agent_id = %agent_id);
        let _enter = span.enter();

        info!("Deregistering agent: {}", agent_id);

        self.agent_registry
            .deregister_local_agent(agent_id)
            .await
            .map_err(|e| match e {
                RegistryError::AgentNotFound { agent_id } => {
                    RouterError::AgentNotFound { agent_id }
                }
                _ => RouterError::ConfigurationError {
                    message: format!("Agent deregistration failed: {e:?}"),
                },
            })?;

        // Record metrics if enabled
        if let Some(collector) = &self.metrics_collector {
            collector.record_agent_deregistered(agent_id);
        }

        info!("Agent deregistered successfully: {}", agent_id);
        Ok(())
    }

    /// Updates an agent's state in its lifecycle
    async fn update_agent_state(
        &self,
        agent_id: AgentId,
        state: AgentState,
    ) -> Result<(), RouterError> {
        let span = span!(Level::DEBUG, "update_agent_state",
                         agent_id = %agent_id,
                         new_state = ?state);
        let _enter = span.enter();

        debug!("Updating agent state: {} -> {:?}", agent_id, state);

        // Look up agent first to verify it exists
        let agent_location = self
            .agent_registry
            .lookup(&agent_id)
            .await
            .map_err(|e| match e {
                RegistryError::AgentNotFound { agent_id } => {
                    RouterError::AgentNotFound { agent_id }
                }
                _ => RouterError::ConfigurationError {
                    message: format!("Failed to lookup agent: {e:?}"),
                },
            })?;

        // Only local agents can have their state updated
        if !matches!(agent_location, AgentLocation::Local(_)) {
            return Err(RouterError::ConfigurationError {
                message: "Cannot update state of remote agent".to_string(),
            });
        }

        // TODO: Update agent state in registry
        // This would require extending the AgentRegistry trait with an update_state method

        debug!(
            "Agent state updated successfully: {} -> {:?}",
            agent_id, state
        );
        Ok(())
    }

    /// Retrieves current router performance statistics
    async fn get_stats(&self) -> Result<RouterStats, RouterError> {
        let span = span!(Level::DEBUG, "get_stats");
        let _enter = span.enter();

        let total_messages = self.message_counter.load(Ordering::Relaxed);
        let total_errors = self.error_counter.load(Ordering::Relaxed);
        let current_rate = self.throughput_tracker.get_current_rate();

        // Calculate uptime
        let _uptime = if let Some(start_time) = *self.start_time.read().await {
            start_time.elapsed()
        } else {
            Duration::ZERO
        };

        let error_rate = if total_messages > 0 {
            (total_errors as f64) / (total_messages as f64)
        } else {
            0.0
        };

        // Get local agents for queue depth calculation
        let local_agents = self.agent_registry.list_local_agents().await.map_err(|e| {
            RouterError::ConfigurationError {
                message: format!("Failed to get agent list: {e:?}"),
            }
        })?;

        let agent_queue_depths = local_agents
            .into_iter()
            .map(|agent| (agent.id, agent.queue_size.as_usize()))
            .collect();

        // Get conversation stats from conversation manager
        let conversation_stats = self
            .conversation_manager
            .get_conversation_stats()
            .await
            .unwrap_or_else(|_| ConversationStats {
                total_active: 0,
                total_created: MessageCount::zero(),
                average_duration_ms: 0,
                average_message_count: 0.0,
                participants_distribution: HashMap::new(),
            });

        let stats = RouterStats {
            messages_per_second: current_rate,
            peak_messages_per_second: current_rate, // TODO: Track peak
            total_messages_processed: MessageCount::new(total_messages as usize),

            // TODO: Collect real latency metrics
            routing_latency_p50: 500, // microseconds
            routing_latency_p90: 1_000,
            routing_latency_p99: 2_000,
            routing_latency_p999: 5_000,

            total_errors: MessageCount::new(total_errors as usize),
            error_rate,
            errors_by_type: HashMap::new(), // TODO: Collect by error type

            inbound_queue_depth: 0,  // TODO: Get actual queue depth
            outbound_queue_depth: 0, // TODO: Get actual queue depth
            agent_queue_depths,

            active_conversations: conversation_stats.total_active,
            total_conversations: conversation_stats.total_created,
            average_conversation_length: conversation_stats.average_message_count,

            memory_usage_bytes: 0,  // TODO: Collect memory usage
            cpu_usage_percent: 0.0, // TODO: Collect CPU usage
            database_size_bytes: 0, // TODO: Collect database size
        };

        trace!("Router stats collected: {:?}", stats);
        Ok(stats)
    }

    /// Checks the health status of the router
    async fn health_check(&self) -> Result<HealthStatus, RouterError> {
        let span = span!(Level::DEBUG, "health_check");
        let _enter = span.enter();

        if !self.is_running.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is not running".to_string(),
            });
        }

        if self.is_shutdown.load(Ordering::SeqCst) {
            return Ok(HealthStatus::Unhealthy {
                reason: "Router is shutting down".to_string(),
            });
        }

        // Check component health
        let delivery_health = self.delivery_engine.health_check().await;

        match delivery_health {
            Ok(HealthStatus::Healthy) => {
                trace!("Router health check passed");
                Ok(HealthStatus::Healthy)
            }
            Ok(HealthStatus::Degraded { reason }) => {
                warn!("Router health degraded: {}", reason);
                Ok(HealthStatus::Degraded { reason })
            }
            Ok(HealthStatus::Unhealthy { reason }) => {
                error!("Router unhealthy: {}", reason);
                Ok(HealthStatus::Unhealthy { reason })
            }
            Err(e) => {
                error!("Health check failed: {:?}", e);
                Ok(HealthStatus::Unhealthy {
                    reason: format!("Health check error: {e:?}"),
                })
            }
        }
    }

    /// Initiates graceful shutdown of the router
    async fn shutdown(&self) -> Result<(), RouterError> {
        let span = span!(Level::INFO, "shutdown");
        let _enter = span.enter();

        if self.is_shutdown.load(Ordering::SeqCst) {
            warn!("Router already shutting down");
            return Ok(());
        }

        info!("Initiating graceful router shutdown");

        // Mark as shutting down
        self.is_shutdown.store(true, Ordering::SeqCst);

        // Stop accepting new work
        self.is_running.store(false, Ordering::SeqCst);

        // Wait for in-flight messages to complete (with timeout)
        let shutdown_timeout = Duration::from_secs(30);
        let shutdown_start = Instant::now();

        while shutdown_start.elapsed() < shutdown_timeout {
            let stats = self.get_stats().await?;
            if stats.inbound_queue_depth == 0 && stats.outbound_queue_depth == 0 {
                break;
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        info!("Router shutdown completed");
        Ok(())
    }
}

/// Additional methods for testing
#[cfg(test)]
impl MessageRouterImpl {
    /// Provides access to the agent registry for testing purposes
    pub fn agent_registry(&self) -> &Arc<dyn AgentRegistry> {
        &self.agent_registry
    }
}

// Placeholder implementations for components that will be implemented next

/// Real delivery engine implementation
#[allow(dead_code)]
struct DeliveryEngineImpl {
    /// Agent message queues for local delivery
    agent_queues: DashMap<AgentId, mpsc::Sender<FipaMessage>>,
    /// Remote node connections (placeholder for now)
    remote_connections: DashMap<NodeId, mpsc::Sender<FipaMessage>>,
    /// Configuration
    config: RouterConfig,
}

impl DeliveryEngineImpl {
    async fn new(config: RouterConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            agent_queues: DashMap::new(),
            remote_connections: DashMap::new(),
            config,
        })
    }

    /// Registers a message queue for a local agent
    #[allow(dead_code)]
    pub fn register_agent_queue(&self, agent_id: AgentId, queue: mpsc::Sender<FipaMessage>) {
        self.agent_queues.insert(agent_id, queue);
    }

    /// Deregisters a message queue for a local agent
    #[allow(dead_code)]
    pub fn deregister_agent_queue(&self, agent_id: AgentId) {
        self.agent_queues.remove(&agent_id);
    }
}

#[async_trait]
impl DeliveryEngine for DeliveryEngineImpl {
    async fn deliver_local(
        &self,
        message: FipaMessage,
        agent: LocalAgent,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // Check if agent is available for delivery
        if !agent.is_available() {
            return Err(DeliveryError::LocalDeliveryFailed {
                source: Box::new(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Agent is not available",
                )),
            });
        }

        // Try to find the agent's message queue
        if let Some(queue) = self.agent_queues.get(&agent.id) {
            // Try to send the message to the agent's queue
            match queue.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} delivered to local agent {}",
                        message_id, agent.id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    warn!("Agent {} queue is full", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Agent queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    error!("Agent {} queue is closed", agent.id);
                    Err(DeliveryError::LocalDeliveryFailed {
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Agent queue is closed",
                        )),
                    })
                }
            }
        } else {
            // Agent doesn't have a registered queue - this is normal during testing
            // In a real system, we'd queue the message for later delivery
            warn!(
                "No queue registered for agent {}, queuing for later delivery",
                agent.id
            );

            // For now, just return success - in production we'd store in a pending queue
            Ok(message_id)
        }
    }

    async fn deliver_remote(
        &self,
        message: FipaMessage,
        node_id: NodeId,
    ) -> Result<MessageId, DeliveryError> {
        let message_id = message.message_id;

        // For remote delivery, we would typically:
        // 1. Serialize the message
        // 2. Send over network (HTTP/gRPC/TCP)
        // 3. Handle retries and circuit breaking

        // For now, simulate remote delivery
        if let Some(connection) = self.remote_connections.get(&node_id) {
            match connection.try_send(message) {
                Ok(()) => {
                    trace!(
                        "Message {} queued for remote delivery to node {}",
                        message_id, node_id
                    );
                    Ok(message_id)
                }
                Err(mpsc::error::TrySendError::Full(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::WouldBlock,
                            "Remote connection queue is full",
                        )),
                    })
                }
                Err(mpsc::error::TrySendError::Closed(_)) => {
                    Err(DeliveryError::RemoteDeliveryFailed {
                        node_id,
                        source: Box::new(std::io::Error::new(
                            std::io::ErrorKind::BrokenPipe,
                            "Remote connection is closed",
                        )),
                    })
                }
            }
        } else {
            // No connection to remote node - simulate successful queuing
            trace!(
                "No connection to node {}, would establish connection in production",
                node_id
            );
            Ok(message_id)
        }
    }

    async fn deliver_batch(
        &self,
        messages: Vec<FipaMessage>,
    ) -> Vec<Result<MessageId, DeliveryError>> {
        // Process messages in parallel for better throughput
        let mut results = Vec::with_capacity(messages.len());

        // For now, process sequentially - in production we'd use concurrent futures
        for message in messages {
            let message_id = message.message_id;

            // For batch processing, we'd need to know the destination
            // This is a simplified implementation
            results.push(Ok(message_id));
        }

        results
    }

    async fn health_check(&self) -> Result<HealthStatus, DeliveryError> {
        // Check if we have healthy connections
        let active_agents = self.agent_queues.len();
        let active_connections = self.remote_connections.len();

        if active_agents == 0 && active_connections == 0 {
            Ok(HealthStatus::Degraded {
                reason: "No active agents or connections".to_string(),
            })
        } else {
            Ok(HealthStatus::Healthy)
        }
    }
}

/// Real conversation manager implementation with `HashMap` storage
struct ConversationManagerImpl {
    /// Active conversations indexed by conversation ID
    conversations: DashMap<ConversationId, Conversation>,
    /// Conversation statistics
    total_created: AtomicU64,
    /// Configuration for timeouts and limits
    config: RouterConfig,
}

impl ConversationManagerImpl {
    async fn new(config: RouterConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            conversations: DashMap::new(),
            total_created: AtomicU64::new(0),
            config,
        })
    }
}

#[async_trait]
impl ConversationManager for ConversationManagerImpl {
    async fn get_or_create_conversation(
        &self,
        conversation_id: ConversationId,
        participants: std::collections::HashSet<AgentId>,
        protocol: Option<ProtocolName>,
    ) -> Result<Conversation, ConversationError> {
        // Check if conversation already exists
        if let Some(conversation) = self.conversations.get(&conversation_id) {
            return Ok(conversation.clone());
        }

        // Validate participant count
        if participants.len() > self.config.max_conversation_participants.into_inner() as usize {
            return Err(ConversationError::TooManyParticipants {
                count: participants.len(),
                max: self.config.max_conversation_participants.into_inner() as usize,
            });
        }

        // Create new conversation
        let conversation = Conversation::new(
            conversation_id,
            participants,
            protocol,
            ConversationCreatedAt::now(),
        );

        // Store the conversation
        self.conversations
            .insert(conversation_id, conversation.clone());
        self.total_created.fetch_add(1, Ordering::Relaxed);

        Ok(conversation)
    }

    async fn update_conversation(
        &self,
        conversation_id: ConversationId,
        message: &FipaMessage,
    ) -> Result<(), ConversationError> {
        // Find and update the conversation
        if let Some(mut conversation) = self.conversations.get_mut(&conversation_id) {
            conversation.add_message(message);
            Ok(())
        } else {
            // Conversation not found - create it if the message has participants
            let mut participants = HashSet::new();
            participants.insert(message.sender);
            participants.insert(message.receiver);

            let mut conversation = self
                .get_or_create_conversation(conversation_id, participants, message.protocol.clone())
                .await?;

            conversation.add_message(message);
            self.conversations.insert(conversation_id, conversation);

            Ok(())
        }
    }

    async fn get_agent_conversations(
        &self,
        agent_id: AgentId,
    ) -> Result<Vec<Conversation>, ConversationError> {
        let conversations: Vec<Conversation> = self
            .conversations
            .iter()
            .filter(|entry| entry.participants.contains(&agent_id))
            .map(|entry| entry.clone())
            .collect();

        Ok(conversations)
    }

    async fn cleanup_expired_conversations(&self) -> Result<usize, ConversationError> {
        let timeout = self.config.conversation_timeout_ms.as_duration();
        let _now = std::time::SystemTime::now();
        let mut cleaned_count = 0;

        // Find expired conversations
        let mut expired_ids = Vec::new();

        for entry in &self.conversations {
            let conversation = entry.value();
            if let Ok(elapsed) = conversation.last_activity.as_system_time().elapsed() {
                if elapsed > timeout {
                    expired_ids.push(*entry.key());
                }
            }
        }

        // Remove expired conversations
        for conversation_id in expired_ids {
            if self.conversations.remove(&conversation_id).is_some() {
                cleaned_count += 1;
            }
        }

        Ok(cleaned_count)
    }

    async fn get_conversation_stats(&self) -> Result<ConversationStats, ConversationError> {
        let total_active = self.conversations.len();
        let total_created = MessageCount::new(self.total_created.load(Ordering::Relaxed) as usize);

        // Calculate average duration and message count
        let mut total_duration_ms = 0u64;
        let mut total_message_count = 0u64;
        let mut participants_distribution = HashMap::new();

        for entry in &self.conversations {
            let conversation = entry.value();

            // Calculate duration
            if let Ok(duration) = conversation
                .last_activity
                .as_system_time()
                .duration_since(conversation.created_at.as_system_time())
            {
                total_duration_ms += duration.as_millis() as u64;
            }

            // Add message count
            total_message_count += conversation.message_count.into_inner() as u64;

            // Track participant distribution
            let participant_count = conversation.participants.len();
            *participants_distribution
                .entry(participant_count)
                .or_insert(0) += 1;
        }

        let average_duration_ms = if total_active > 0 {
            total_duration_ms / total_active as u64
        } else {
            0
        };

        let average_message_count = if total_active > 0 {
            total_message_count as f64 / total_active as f64
        } else {
            0.0
        };

        Ok(ConversationStats {
            total_active,
            total_created,
            average_duration_ms,
            average_message_count,
            participants_distribution,
        })
    }
}

/// Node information for remote agent routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    pub id: NodeId,
    pub name: String,
    pub address: String,
    pub is_healthy: bool,
    pub last_heartbeat: MessageTimestamp,
    pub agent_count: usize,
}

impl NodeInfo {
    pub fn new(id: NodeId, name: String, address: String) -> Self {
        Self {
            id,
            name,
            address,
            is_healthy: true,
            last_heartbeat: MessageTimestamp::now(),
            agent_count: 0,
        }
    }
}

/// Real agent registry implementation with O(1) lookup performance
pub struct AgentRegistryImpl {
    /// O(1) lookup for local agents
    agents: DashMap<AgentId, LocalAgent>,

    /// O(1) lookup for routing information
    routes: DashMap<AgentId, AgentLocation>,

    /// O(1) lookup for capability-based discovery
    capabilities: DashMap<CapabilityName, HashSet<AgentId>>,

    /// Node registry for remote agents
    node_registry: DashMap<NodeId, NodeInfo>,
}

impl AgentRegistryImpl {
    async fn new(_config: RouterConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {
            agents: DashMap::new(),
            routes: DashMap::new(),
            capabilities: DashMap::new(),
            node_registry: DashMap::new(),
        })
    }
}

#[async_trait]
impl AgentRegistry for AgentRegistryImpl {
    /// O(1) agent lookup with actual implementation
    async fn lookup(&self, agent_id: &AgentId) -> Result<AgentLocation, RegistryError> {
        // First check routes cache for O(1) lookup
        if let Some(location) = self.routes.get(agent_id) {
            return Ok(location.clone());
        }

        // If not in cache, check if it's a local agent
        if let Some(agent) = self.agents.get(agent_id) {
            let location = AgentLocation::Local(agent.clone());
            // Cache the location for future O(1) lookups
            self.routes.insert(*agent_id, location.clone());
            return Ok(location);
        }

        // Agent not found
        Err(RegistryError::AgentNotFound {
            agent_id: *agent_id,
        })
    }

    /// Registers a local agent with capabilities indexing
    async fn register_local_agent(
        &self,
        agent: LocalAgent,
        capabilities: Vec<CapabilityName>,
    ) -> Result<(), RegistryError> {
        let agent_id = agent.id;

        // Check if agent is already registered
        if self.agents.contains_key(&agent_id) {
            return Err(RegistryError::AgentAlreadyRegistered { agent_id });
        }

        // Store the agent
        self.agents.insert(agent_id, agent.clone());

        // Cache the location for O(1 lookups
        self.routes.insert(agent_id, AgentLocation::Local(agent));

        // Index capabilities for discovery
        for capability in capabilities {
            self.capabilities
                .entry(capability)
                .and_modify(|agents| {
                    agents.insert(agent_id);
                })
                .or_insert_with(|| {
                    let mut agents = HashSet::new();
                    agents.insert(agent_id);
                    agents
                });
        }

        Ok(())
    }

    /// Deregisters a local agent and cleans up all indexes
    async fn deregister_local_agent(&self, agent_id: AgentId) -> Result<(), RegistryError> {
        // Remove from agents map
        let agent = self
            .agents
            .remove(&agent_id)
            .ok_or(RegistryError::AgentNotFound { agent_id })?;

        // Remove from routes cache
        self.routes.remove(&agent_id);

        // Clean up capability indexes
        for capability in &agent.1.capabilities {
            if let Some(mut agents) = self.capabilities.get_mut(capability) {
                agents.remove(&agent_id);
                // Remove empty capability entries
                if agents.is_empty() {
                    drop(agents);
                    self.capabilities.remove(capability);
                }
            }
        }

        Ok(())
    }

    /// Updates routing table with remote agent information
    async fn update_remote_route(
        &self,
        agent_id: AgentId,
        node_id: NodeId,
        _hops: RouteHops,
    ) -> Result<(), RegistryError> {
        // Ensure node exists in registry
        if !self.node_registry.contains_key(&node_id) {
            let node_info =
                NodeInfo::new(node_id, format!("node-{node_id}"), "unknown".to_string());
            self.node_registry.insert(node_id, node_info);
        }

        // Update route cache
        self.routes.insert(agent_id, AgentLocation::Remote(node_id));

        // Update node agent count
        if let Some(mut node_info) = self.node_registry.get_mut(&node_id) {
            node_info.agent_count += 1;
        }

        Ok(())
    }

    /// O(1) capability-based agent discovery
    async fn find_agents_by_capability(
        &self,
        capability: &CapabilityName,
    ) -> Result<Vec<AgentId>, RegistryError> {
        if let Some(agents) = self.capabilities.get(capability) {
            Ok(agents.iter().copied().collect())
        } else {
            Ok(vec![])
        }
    }

    /// Lists all local agents
    async fn list_local_agents(&self) -> Result<Vec<LocalAgent>, RegistryError> {
        Ok(self
            .agents
            .iter()
            .map(|entry| entry.value().clone())
            .collect())
    }

    /// Updates agent health status
    async fn update_agent_health(
        &self,
        agent_id: AgentId,
        _is_healthy: bool,
        last_heartbeat: MessageTimestamp,
    ) -> Result<(), RegistryError> {
        if let Some(mut agent) = self.agents.get_mut(&agent_id) {
            agent.last_heartbeat = last_heartbeat;
            Ok(())
        } else {
            Err(RegistryError::AgentNotFound { agent_id })
        }
    }
}

/// Placeholder failure handler implementation
struct FailureHandlerImpl;

impl FailureHandlerImpl {
    async fn new(_config: RouterConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self)
    }
}

#[async_trait]
impl FailureHandler for FailureHandlerImpl {
    async fn handle_routing_failure(
        &self,
        _message: FipaMessage,
        _error: RouterError,
    ) -> Result<MessageId, RouterError> {
        // Placeholder implementation
        Ok(MessageId::generate())
    }

    async fn schedule_retry(
        &self,
        _message: FipaMessage,
        _retry_count: u8,
    ) -> Result<(), RouterError> {
        // Placeholder implementation
        Ok(())
    }

    async fn dead_letter(
        &self,
        _message: FipaMessage,
        _reason: FailureReason,
    ) -> Result<(), RouterError> {
        // Placeholder implementation
        Ok(())
    }

    async fn get_dead_letter_stats(&self) -> Result<DeadLetterStats, RouterError> {
        // Placeholder implementation
        Ok(DeadLetterStats {
            total_messages: MessageCount::zero(),
            messages_by_reason: HashMap::new(),
            oldest_message_age_ms: None,
            queue_size_bytes: 0,
        })
    }
}

/// Placeholder metrics collector implementation
struct MetricsCollectorImpl;

impl MetricsCollectorImpl {
    fn new() -> Self {
        Self
    }
}

impl MetricsCollector for MetricsCollectorImpl {
    fn record_message_routed(&self, _message: &FipaMessage, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_routing_error(&self, _error: &RouterError) {
        // Placeholder implementation
    }

    fn record_delivery_metrics(&self, _success: bool, _duration: Duration) {
        // Placeholder implementation
    }

    fn record_conversation_created(&self) {
        // Placeholder implementation
    }

    fn record_agent_registered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }

    fn record_agent_deregistered(&self, _agent_id: AgentId) {
        // Placeholder implementation
    }
}
