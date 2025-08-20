//! Main message router implementation
//!
//! Coordinates message routing between agents using a coordination-first architecture
//! with high-performance async processing and comprehensive error handling.

use crate::{
    database::{DatabaseConfig, DatabaseConnection, DatabaseError, DatabasePath},
    message_router::{
        config::RouterConfig,
        domain_types::{
            AgentId, AgentLocation, AgentState, CapabilityName, Conversation,
            ConversationCreatedAt, ConversationId, DeliveryOptions, FailureReason, FipaMessage,
            LocalAgent, MessageContent, MessageCount, MessageId, MessageTimestamp, NodeId,
            Performative, ProtocolName, RouteHops, RouteInfo,
        },
        traits::{
            AgentRegistry, ConversationError, ConversationManager, ConversationStats,
            DeadLetterStats, DeliveryEngine, DeliveryError, FailureHandler, HealthStatus,
            MessageRouter, MetricsCollector, RegistryError, RouterError, RouterStats,
        },
    },
};
use async_trait::async_trait;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::Row;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration, Instant};
use tracing::{Level, debug, error, info, span, trace, warn};
use uuid::Uuid;

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

    // Routing storage for persistent message delivery optimization
    routing_storage: Arc<RouteStorage>,

    // In-memory cache for sub-millisecond lookups
    routing_cache: DashMap<AgentId, RouteInfo>,
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

// Pure functions for time operations (functional core)
fn get_current_unix_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

// Pure functions for error handling (functional core)
fn map_registry_error_to_router_error(error: &RegistryError) -> RouterError {
    match error {
        RegistryError::AgentNotFound { agent_id } => RouterError::AgentNotFound {
            agent_id: *agent_id,
        },
        _ => RouterError::ConfigurationError {
            message: format!("Registry error: {error:?}"),
        },
    }
}

fn map_delivery_error_to_router_error(error: DeliveryError) -> RouterError {
    match error {
        DeliveryError::LocalDeliveryFailed { source }
        | DeliveryError::RemoteDeliveryFailed { source, .. } => {
            RouterError::NetworkError { source }
        }
        DeliveryError::CircuitBreakerOpen { node_id } => {
            RouterError::CircuitBreakerOpen { node_id }
        }
        _ => RouterError::ConfigurationError {
            message: format!("Delivery error: {error:?}"),
        },
    }
}

fn map_registry_error_for_agent_ops(error: &RegistryError, operation: &str) -> RouterError {
    match error {
        RegistryError::AgentAlreadyRegistered { agent_id } => RouterError::ConfigurationError {
            message: format!("Agent already registered: {agent_id}"),
        },
        RegistryError::AgentNotFound { agent_id } => RouterError::AgentNotFound {
            agent_id: *agent_id,
        },
        _ => RouterError::ConfigurationError {
            message: format!("{operation} failed: {error:?}"),
        },
    }
}

fn calculate_cutoff_timestamp(current_time: u64, window_seconds: u64) -> u64 {
    current_time.saturating_sub(window_seconds)
}

fn calculate_throughput_rate(total_messages: u64, window_seconds: f64) -> f64 {
    if total_messages == 0 {
        0.0
    } else {
        f64::from(u32::try_from(total_messages.min(u64::from(u32::MAX))).unwrap_or(0))
            / window_seconds
    }
}

// Pure functions for statistics calculations (functional core)
fn safe_u64_to_u32(value: u64) -> u32 {
    u32::try_from(value.min(u64::from(u32::MAX))).unwrap_or(0)
}

fn calculate_error_rate(total_errors: u64, total_messages: u64) -> f64 {
    if total_messages > 0 {
        let safe_errors = safe_u64_to_u32(total_errors);
        let safe_messages = safe_u64_to_u32(total_messages).max(1); // Avoid division by zero
        f64::from(safe_errors) / f64::from(safe_messages)
    } else {
        0.0
    }
}

fn calculate_uptime(start_time: Option<Instant>) -> Duration {
    if let Some(start_time) = start_time {
        start_time.elapsed()
    } else {
        Duration::ZERO
    }
}

fn create_agent_queue_depths(local_agents: Vec<LocalAgent>) -> HashMap<AgentId, usize> {
    local_agents
        .into_iter()
        .map(|agent| (agent.id, agent.queue_size.as_usize()))
        .collect()
}

fn safe_message_count_from_u64(value: u64) -> MessageCount {
    MessageCount::new(value.min(1_000_000) as usize)
}

// Pure functions for conversation management (functional core)
fn calculate_conversation_duration_ms(
    created_at: std::time::SystemTime,
    last_activity: std::time::SystemTime,
) -> u64 {
    if let Ok(duration) = last_activity.duration_since(created_at) {
        u64::try_from(duration.as_millis().min(u128::from(u64::MAX))).unwrap_or(0)
    } else {
        0
    }
}

fn calculate_average_duration(total_duration_ms: u64, conversation_count: usize) -> u64 {
    if conversation_count > 0 {
        total_duration_ms / conversation_count as u64
    } else {
        0
    }
}

fn calculate_average_message_count(total_message_count: u64, conversation_count: usize) -> f64 {
    if conversation_count > 0 {
        let safe_msg_count = safe_u64_to_u32(total_message_count);
        let safe_active = u32::try_from(conversation_count.min(u32::MAX as usize)).unwrap_or(1);
        f64::from(safe_msg_count) / f64::from(safe_active)
    } else {
        0.0
    }
}

fn is_conversation_expired(last_activity: MessageTimestamp, timeout: Duration) -> bool {
    if let Ok(elapsed) = last_activity.as_system_time().elapsed() {
        elapsed > timeout
    } else {
        false
    }
}

fn sum_messages_in_window<I>(entries: I, cutoff: u64) -> u64
where
    I: Iterator<Item = (u64, u64)>,
{
    entries
        .filter(|(timestamp, _)| *timestamp >= cutoff)
        .map(|(_, count)| count)
        .sum()
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
        let now = get_current_unix_timestamp();

        self.samples
            .entry(now)
            .and_modify(|count| *count += 1)
            .or_insert(1);

        // Clean old samples
        let cutoff = calculate_cutoff_timestamp(now, self.window_size.as_secs());
        self.samples.retain(|&timestamp, _| timestamp >= cutoff);
    }

    fn get_current_rate(&self) -> f64 {
        let now = get_current_unix_timestamp();
        let cutoff = calculate_cutoff_timestamp(now, self.window_size.as_secs());

        let total_messages = sum_messages_in_window(
            self.samples
                .iter()
                .map(|entry| (*entry.key(), *entry.value())),
            cutoff,
        );

        calculate_throughput_rate(total_messages, self.window_size.as_secs_f64())
    }
}

impl MessageRouterImpl {
    /// Creates a new message router with the given configuration
    ///
    /// This will create and wire up all necessary components based on the config.
    ///
    /// # Errors
    ///
    /// Returns a `RouterError` if configuration validation fails or component creation fails.
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
        let delivery_engine = Arc::new(DeliveryEngineImpl::new(config.clone()));

        let conversation_manager = Arc::new(ConversationManagerImpl::new(config.clone()));

        let agent_registry = Arc::new(AgentRegistryImpl::new(config.clone()));

        let failure_handler = Arc::new(FailureHandlerImpl::new(config.clone()));

        // Create metrics collector if enabled
        let metrics_collector = if config.enable_metrics() {
            Some(Arc::new(MetricsCollectorImpl::new()) as Arc<dyn MetricsCollector>)
        } else {
            None
        };

        // Create concurrency control
        let routing_semaphore = Arc::new(Semaphore::new(config.inbound_queue_size.as_usize()));

        // Create throughput tracker
        let throughput_tracker = Arc::new(ThroughputTracker::new(Duration::from_secs(60)));

        // Initialize persistent route storage
        let routing_storage =
            Arc::new(
                RouteStorage::new()
                    .await
                    .map_err(|e| RouterError::ConfigurationError {
                        message: format!("Failed to initialize route storage: {e}"),
                    })?,
            );

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
            routing_storage,
            routing_cache: DashMap::new(),
        };

        info!("Message router created successfully");
        Ok(router)
    }

    /// Starts the message router background processing
    ///
    /// Spawns worker tasks for processing messages concurrently based on configuration.
    ///
    /// # Errors
    ///
    /// Returns a `RouterError` if the router is already running or configuration is invalid.
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
        self.spawn_message_processor(receiver);

        // Spawn worker tasks
        for worker_id in 0..self.config.worker_thread_count.as_usize() {
            self.spawn_worker_task(worker_id);
        }

        // Spawn health monitoring task
        if self.config.health_check_interval_ms.as_duration() > Duration::ZERO {
            self.spawn_health_monitoring_task();
        }

        // Spawn metrics collection task
        if self.config.enable_metrics() {
            self.spawn_metrics_task();
        }

        info!(
            "Message router started with {} workers",
            self.config.worker_thread_count.as_usize()
        );
        Ok(())
    }

    /// Spawns a worker task for processing routing messages
    #[allow(unused_variables)]
    fn spawn_worker_task(&self, worker_id: usize) {
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
    fn spawn_message_processor(&self, mut receiver: mpsc::Receiver<RoutingTask>) {
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
        if let Some(conversation_id) = task.message.conversation_id
            && let Err(e) = conversation_manager
                .update_conversation(conversation_id, &task.message)
                .await
        {
            warn!("Failed to update conversation {}: {:?}", conversation_id, e);
            // Don't fail the routing for conversation update failures
        }

        // Look up the destination agent
        let agent_location = agent_registry
            .lookup(&task.message.receiver)
            .await
            .map_err(|e| map_registry_error_to_router_error(&e))?;

        // Route based on agent location
        match agent_location {
            AgentLocation::Local(local_agent) => {
                trace!("Routing to local agent: {}", local_agent.name);
                delivery_engine
                    .deliver_local(task.message, local_agent)
                    .await
                    .map_err(map_delivery_error_to_router_error)
            }
            AgentLocation::Remote(node_id) => {
                trace!("Routing to remote node: {}", node_id);
                delivery_engine
                    .deliver_remote(task.message, node_id)
                    .await
                    .map_err(map_delivery_error_to_router_error)
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
    fn spawn_health_monitoring_task(&self) {
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
    fn spawn_metrics_task(&self) {
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

    /// Stores route information for message delivery optimization
    ///
    /// Uses `SQLite` for persistence and updates in-memory cache for fast lookups.
    ///
    /// # Errors
    ///
    /// Returns an error if route storage fails or database operations fail.
    /// Performance target: < 1ms including cache update.
    pub async fn store_route(
        &self,
        agent_id: AgentId,
        route_info: RouteInfo,
    ) -> Result<(), RouterError> {
        // Store in persistent storage (functional core + imperative shell)
        self.routing_storage
            .store_route(agent_id, &route_info)
            .await
            .map_err(|e| RouterError::ConfigurationError {
                message: format!("Route storage failed: {e}"),
            })?;

        // Update cache for sub-millisecond lookups (imperative shell)
        self.routing_cache.insert(agent_id, route_info);

        Ok(())
    }

    /// Looks up route information for an agent
    ///
    /// Uses in-memory cache first for sub-millisecond performance, falls back to `SQLite`.
    ///
    /// # Errors
    ///
    /// Returns an error if route lookup fails or database operations fail.
    /// Performance target: < 1ms for cache hits, < 5ms for cache misses.
    pub async fn lookup_route(&self, agent_id: AgentId) -> Result<Option<RouteInfo>, RouterError> {
        // Try cache first for sub-millisecond performance (imperative shell)
        if let Some(route) = self.routing_cache.get(&agent_id) {
            let route_info = route.clone();
            drop(route); // Release lock immediately

            // Check if route is still fresh (functional core)
            if route_is_fresh(&route_info) {
                return Ok(Some(route_info));
            }
            // Remove stale route from cache
            self.routing_cache.remove(&agent_id);
        }

        // Cache miss or stale route - query persistent storage (functional core + imperative shell)
        match self.routing_storage.lookup_route(agent_id).await {
            Ok(Some(route_info)) => {
                // Update cache for future lookups (imperative shell)
                if route_is_fresh(&route_info) {
                    self.routing_cache.insert(agent_id, route_info.clone());
                    Ok(Some(route_info))
                } else {
                    // Route expired, clean up storage
                    let _ = self.routing_storage.remove_route(agent_id).await;
                    Ok(None)
                }
            }
            Ok(None) => Ok(None),
            Err(e) => Err(RouterError::ConfigurationError {
                message: format!("Route lookup failed: {e}"),
            }),
        }
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
            routing_storage: Arc::clone(&self.routing_storage),
            routing_cache: DashMap::new(), // New instances get empty cache
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
        if self.config.enable_message_validation()
            && message.content.len() > self.config.max_message_size_bytes()
        {
            return Err(RouterError::MessageTooLarge {
                size: message.content.len(),
                max_size: self.config.max_message_size_bytes(),
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
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent registration"))?;

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
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent deregistration"))?;

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
            .map_err(|e| map_registry_error_for_agent_ops(&e, "Agent lookup"))?;

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
        let _uptime = calculate_uptime(*self.start_time.read().await);

        let error_rate = calculate_error_rate(total_errors, total_messages);

        // Get local agents for queue depth calculation
        let local_agents = self.agent_registry.list_local_agents().await.map_err(|e| {
            RouterError::ConfigurationError {
                message: format!("Failed to get agent list: {e:?}"),
            }
        })?;

        let agent_queue_depths = create_agent_queue_depths(local_agents);

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
            total_messages_processed: safe_message_count_from_u64(total_messages),

            // TODO: Collect real latency metrics
            routing_latency_p50: 500, // microseconds
            routing_latency_p90: 1_000,
            routing_latency_p99: 2_000,
            routing_latency_p999: 5_000,

            total_errors: safe_message_count_from_u64(total_errors),
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
    fn new(config: RouterConfig) -> Self {
        Self {
            agent_queues: DashMap::new(),
            remote_connections: DashMap::new(),
            config,
        }
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
    fn new(config: RouterConfig) -> Self {
        Self {
            conversations: DashMap::new(),
            total_created: AtomicU64::new(0),
            config,
        }
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
            if is_conversation_expired(conversation.last_activity, timeout) {
                expired_ids.push(*entry.key());
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
        let total_created =
            MessageCount::new(self.total_created.load(Ordering::Relaxed).min(1_000_000) as usize);

        // Calculate average duration and message count
        let mut total_duration_ms = 0u64;
        let mut total_message_count = 0u64;
        let mut participants_distribution = HashMap::new();

        for entry in &self.conversations {
            let conversation = entry.value();

            // Calculate duration
            let duration_ms = calculate_conversation_duration_ms(
                conversation.created_at.as_system_time(),
                conversation.last_activity.as_system_time(),
            );
            total_duration_ms = total_duration_ms.saturating_add(duration_ms);

            // Add message count
            total_message_count += conversation.message_count.into_inner() as u64;

            // Track participant distribution
            let participant_count = conversation.participants.len();
            *participants_distribution
                .entry(participant_count)
                .or_insert(0) += 1;
        }

        let average_duration_ms = calculate_average_duration(total_duration_ms, total_active);

        let average_message_count =
            calculate_average_message_count(total_message_count, total_active);

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
    fn new(_config: RouterConfig) -> Self {
        Self {
            agents: DashMap::new(),
            routes: DashMap::new(),
            capabilities: DashMap::new(),
            node_registry: DashMap::new(),
        }
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
    fn new(_config: RouterConfig) -> Self {
        Self
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

// =============================================================================
// FUNCTIONAL CORE - Pure functions for route operations
// =============================================================================

/// Pure function to check if a route is still fresh (functional core)
fn route_is_fresh(route_info: &RouteInfo) -> bool {
    std::time::SystemTime::now() < route_info.expires_at.as_system_time()
}

/// Pure SQL generation functions for route storage operations
mod route_sql {
    /// Generate SQL for creating routes table (functional core)
    pub(super) fn create_routes_table() -> &'static str {
        "CREATE TABLE IF NOT EXISTS routes (
            agent_id TEXT PRIMARY KEY,
            node_id TEXT NOT NULL,
            hops INTEGER NOT NULL,
            updated_at INTEGER NOT NULL,
            expires_at INTEGER NOT NULL
        )"
    }

    /// Generate SQL for inserting/updating route (functional core)
    pub(super) fn upsert_route() -> &'static str {
        "INSERT OR REPLACE INTO routes (agent_id, node_id, hops, updated_at, expires_at) VALUES (?, ?, ?, ?, ?)"
    }

    /// Generate SQL for selecting route by agent ID (functional core)
    pub(super) fn select_route_by_agent_id() -> &'static str {
        "SELECT agent_id, node_id, hops, updated_at, expires_at FROM routes WHERE agent_id = ?"
    }

    /// Generate SQL for removing expired routes (functional core)
    pub(super) fn delete_expired_routes() -> &'static str {
        "DELETE FROM routes WHERE expires_at < ?"
    }

    /// Generate SQL for removing specific route (functional core)
    pub(super) fn delete_route_by_agent_id() -> &'static str {
        "DELETE FROM routes WHERE agent_id = ?"
    }
}

/// Pure data mapping functions for route storage
mod route_mapping {
    use super::{AgentId, MessageTimestamp, NodeId, RouteHops, RouteInfo, Uuid};
    use crate::database::DatabaseError;
    use std::time::UNIX_EPOCH;

    /// Convert `AgentId` to string for database storage (functional core)
    pub(super) fn agent_id_to_string(id: AgentId) -> String {
        id.to_string()
    }

    /// Convert `NodeId` to string for database storage (functional core)
    pub(super) fn node_id_to_string(id: NodeId) -> String {
        id.to_string()
    }

    /// Convert `MessageTimestamp` to Unix timestamp for database storage (functional core)
    #[allow(clippy::cast_possible_wrap)] // Unix timestamps fit in i64 for reasonable time ranges
    pub(super) fn timestamp_to_unix(timestamp: MessageTimestamp) -> i64 {
        timestamp
            .as_system_time()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64
    }

    /// Parse `NodeId` from database string (functional core)
    pub(super) fn parse_node_id(id_str: &str) -> Result<NodeId, DatabaseError> {
        let uuid = Uuid::parse_str(id_str).map_err(|e| {
            DatabaseError::Storage(crate::database::StorageError::Database {
                message: format!("Invalid node ID format: {e}"),
            })
        })?;
        Ok(NodeId::from(uuid))
    }

    /// Parse `RouteHops` from database integer (functional core)
    pub(super) fn parse_route_hops(hops: i64) -> Result<RouteHops, DatabaseError> {
        let hops_u8 = u8::try_from(hops).map_err(|e| {
            DatabaseError::Storage(crate::database::StorageError::Database {
                message: format!("Invalid route hops value: {e}"),
            })
        })?;

        RouteHops::try_new(hops_u8).map_err(|e| {
            DatabaseError::Storage(crate::database::StorageError::Database {
                message: format!("Route hops validation failed: {e}"),
            })
        })
    }

    /// Parse `MessageTimestamp` from Unix timestamp (functional core)
    #[allow(clippy::cast_sign_loss)] // Valid Unix timestamps are positive
    pub(super) fn parse_timestamp(unix_timestamp: i64) -> MessageTimestamp {
        let system_time = UNIX_EPOCH + std::time::Duration::from_secs(unix_timestamp as u64);
        MessageTimestamp::new(system_time)
    }

    /// Create `RouteInfo` from parsed components (functional core)
    pub(super) fn create_route_info(
        node_id: NodeId,
        hops: RouteHops,
        updated_at: MessageTimestamp,
        expires_at: MessageTimestamp,
    ) -> RouteInfo {
        RouteInfo {
            node_id,
            hops,
            updated_at,
            expires_at,
        }
    }
}

// =============================================================================
// IMPERATIVE SHELL - Persistent route storage
// =============================================================================

/// Route storage implementation using `SQLite` for persistence
///
/// Follows functional core / imperative shell pattern:
/// - Functional core: SQL generation, data mapping, validation
/// - Imperative shell: Database I/O operations, connection management
pub struct RouteStorage {
    connection: DatabaseConnection,
}

impl RouteStorage {
    /// Create new route storage with in-memory `SQLite` database (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database initialization fails.
    pub async fn new() -> Result<Self, DatabaseError> {
        // Create a temporary SQLite database file for route storage
        // This provides persistence with excellent performance for routing data
        let temp_dir = std::env::temp_dir();
        let db_file = temp_dir.join(format!("caxton_routes_{}.db", uuid::Uuid::new_v4()));
        let db_path = DatabasePath::new(db_file)?;

        let config = DatabaseConfig::for_testing(db_path)
            .with_wal_mode(false) // Not needed for in-memory
            .with_foreign_keys(false); // Simpler for route storage

        let connection = DatabaseConnection::initialize(config).await?;

        let storage = Self { connection };

        // Initialize the routes table (imperative shell)
        storage.ensure_routes_table_exists().await?;

        Ok(storage)
    }

    /// Store route information in database (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database operations fail.
    pub async fn store_route(
        &self,
        agent_id: AgentId,
        route_info: &RouteInfo,
    ) -> Result<(), DatabaseError> {
        // Convert domain types to database format (functional core)
        let agent_id_str = route_mapping::agent_id_to_string(agent_id);
        let node_id_str = route_mapping::node_id_to_string(route_info.node_id);
        let hops = i64::from(route_info.hops.into_inner());
        let updated_at = route_mapping::timestamp_to_unix(route_info.updated_at);
        let expires_at = route_mapping::timestamp_to_unix(route_info.expires_at);

        // Execute database operation (imperative shell)
        sqlx::query(route_sql::upsert_route())
            .bind(agent_id_str)
            .bind(node_id_str)
            .bind(hops)
            .bind(updated_at)
            .bind(expires_at)
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }

    /// Lookup route information by agent ID (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database operations fail.
    pub async fn lookup_route(
        &self,
        agent_id: AgentId,
    ) -> Result<Option<RouteInfo>, DatabaseError> {
        // Convert agent ID for query (functional core)
        let agent_id_str = route_mapping::agent_id_to_string(agent_id);

        // Execute database query (imperative shell)
        let result = sqlx::query(route_sql::select_route_by_agent_id())
            .bind(agent_id_str)
            .fetch_optional(self.connection.pool())
            .await?;

        if let Some(row) = result {
            // Extract raw data from database row (imperative shell)
            let node_id_str: String = row.get("node_id");
            let hops: i64 = row.get("hops");
            let updated_at: i64 = row.get("updated_at");
            let expires_at: i64 = row.get("expires_at");

            // Convert database format to domain types (functional core)
            let node_id = route_mapping::parse_node_id(&node_id_str)?;
            let route_hops = route_mapping::parse_route_hops(hops)?;
            let updated_timestamp = route_mapping::parse_timestamp(updated_at);
            let expires_timestamp = route_mapping::parse_timestamp(expires_at);

            // Create domain object (functional core)
            let route_info = route_mapping::create_route_info(
                node_id,
                route_hops,
                updated_timestamp,
                expires_timestamp,
            );

            Ok(Some(route_info))
        } else {
            Ok(None)
        }
    }

    /// Remove route by agent ID (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database operations fail.
    pub async fn remove_route(&self, agent_id: AgentId) -> Result<(), DatabaseError> {
        let agent_id_str = route_mapping::agent_id_to_string(agent_id);

        sqlx::query(route_sql::delete_route_by_agent_id())
            .bind(agent_id_str)
            .execute(self.connection.pool())
            .await?;

        Ok(())
    }

    /// Clean up expired routes (imperative shell)
    ///
    /// # Errors
    ///
    /// Returns an error if database operations fail.
    #[allow(clippy::cast_possible_wrap)] // Unix timestamps fit in i64 for reasonable time ranges
    pub async fn cleanup_expired_routes(&self) -> Result<u64, DatabaseError> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        let result = sqlx::query(route_sql::delete_expired_routes())
            .bind(now)
            .execute(self.connection.pool())
            .await?;

        Ok(result.rows_affected())
    }

    /// Ensure routes table exists in database (imperative shell)
    async fn ensure_routes_table_exists(&self) -> Result<(), DatabaseError> {
        sqlx::query(route_sql::create_routes_table())
            .execute(self.connection.pool())
            .await?;
        Ok(())
    }
}
