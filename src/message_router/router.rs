//! Main message router implementation
//!
//! This module provides the core `MessageRouterImpl` struct that coordinates message routing
//! between agents using dependency injection and a coordination-first architecture.
//!
//! # Architecture
//!
//! The router follows a dependency injection pattern where all major components are
//! injected as traits:
//! - **`DeliveryEngine`**: Handles actual message delivery
//! - **`ConversationManager`**: Manages conversation state and context
//! - **`AgentRegistry`**: Provides O(1) agent lookup and registration
//! - **`FailureHandler`**: Handles retries, circuit breaking, and dead-lettering
//! - **`MetricsCollector`**: Collects performance and operational metrics
//!
//! This design enables easy testing, configuration, and customization of router behavior
//! without tight coupling to specific implementations.

use crate::{
    database::DatabaseError,
    message_router::{
        config::RouterConfig,
        domain_types::{AgentId, FipaMessage, RouteInfo},
        traits::{
            AgentRegistry, ConversationManager, DeliveryEngine, FailureHandler, MetricsCollector,
            RouterError,
        },
    },
};
use dashmap::DashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use tokio::sync::{RwLock, Semaphore, mpsc};
use tokio::time::{Duration, Instant};
use tracing::info;

/// Core message router that orchestrates agent communication
///
/// The router acts as a coordination hub, delegating specific responsibilities
/// to injected components while maintaining overall message flow control.
///
/// # Design Principles
///
/// - **Separation of concerns**: Each injected component has a single responsibility
/// - **Testability**: All dependencies are traits that can be mocked
/// - **Performance**: Asynchronous processing with configurable concurrency limits
/// - **Reliability**: Built-in error handling, retries, and circuit breaking
/// - **Observability**: Comprehensive metrics and tracing support
///
/// # Lifecycle
///
/// 1. Create with `new()` by injecting all required components
/// 2. Start with `start()` to begin processing messages
/// 3. The router handles incoming messages asynchronously
/// 4. Shutdown gracefully when needed
#[allow(dead_code)] // Many fields unused after implementation extraction
pub struct MessageRouterImpl {
    pub(crate) config: RouterConfig,

    // Core components (injected)
    delivery_engine: Arc<dyn DeliveryEngine>,
    pub(crate) conversation_manager: Arc<dyn ConversationManager>,
    pub(crate) agent_registry: Arc<dyn AgentRegistry>,
    failure_handler: Arc<dyn FailureHandler>,

    // Internal state
    pub(crate) is_running: AtomicBool,
    pub(crate) is_shutdown: AtomicBool,
    start_time: RwLock<Option<Instant>>,

    // Performance tracking
    pub(crate) message_counter: AtomicU64,
    error_counter: AtomicU64,
    pub(crate) throughput_tracker: Arc<ThroughputTracker>,

    // Queue management
    pub(crate) inbound_queue: mpsc::Sender<RoutingTask>,
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
#[allow(dead_code)] // Fields unused after implementation extraction
pub(crate) struct RoutingTask {
    #[allow(dead_code)]
    pub message: FipaMessage,
    #[allow(dead_code)]
    pub attempt_count: u8,
    #[allow(dead_code)]
    pub created_at: Instant,
    #[allow(dead_code)]
    pub span: tracing::Span,
}

/// Lightweight throughput tracker for performance monitoring
///
/// Tracks message processing rates over a configurable time window.
/// Currently simplified after implementation extraction but maintains
/// the same interface for future enhancements.
#[allow(dead_code)] // Field unused in current minimal implementation
pub(crate) struct ThroughputTracker {
    /// Time window for rate calculations
    _window_size: Duration,
}

impl ThroughputTracker {
    /// Create a new throughput tracker with the specified window size
    ///
    /// # Arguments
    ///
    /// * `window_size` - Time window for calculating message rates
    fn new(window_size: Duration) -> Self {
        Self {
            _window_size: window_size,
        }
    }
}

impl MessageRouterImpl {
    /// Create a new message router with dependency injection
    ///
    /// # Errors
    ///
    /// Returns `RouterError::ConfigurationError` if route storage initialization fails.
    pub fn new(
        config: RouterConfig,
        delivery_engine: Arc<dyn DeliveryEngine>,
        conversation_manager: Arc<dyn ConversationManager>,
        agent_registry: Arc<dyn AgentRegistry>,
        failure_handler: Arc<dyn FailureHandler>,
        metrics_collector: Option<Arc<dyn MetricsCollector>>,
    ) -> Result<Self, RouterError> {
        let queue_size = config.inbound_queue_size.as_usize();
        let worker_thread_count = config.worker_thread_count.as_usize();

        let (inbound_sender, inbound_receiver) = mpsc::channel(queue_size);
        let routing_storage =
            Arc::new(
                RouteStorage::new().map_err(|e| RouterError::ConfigurationError {
                    message: format!("Failed to create route storage: {e:?}"),
                })?,
            );

        let throughput_tracker = Arc::new(ThroughputTracker::new(Duration::from_secs(60)));

        Ok(Self {
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
            routing_semaphore: Arc::new(Semaphore::new(worker_thread_count)),
            metrics_collector,
            routing_storage,
            routing_cache: DashMap::new(),
        })
    }

    /// Start the message router
    ///
    /// # Errors
    ///
    /// Returns `RouterError::ConfigurationError` if router is already running.
    pub async fn start(&self) -> Result<(), RouterError> {
        if self.is_running.load(Ordering::SeqCst) {
            return Err(RouterError::ConfigurationError {
                message: "Router is already running".to_string(),
            });
        }

        *self.start_time.write().await = Some(Instant::now());
        self.is_running.store(true, Ordering::SeqCst);

        info!("Message router started successfully");
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

/// Minimal route storage placeholder - move to separate module
pub struct RouteStorage;

impl RouteStorage {
    /// Creates a new route storage instance
    ///
    /// # Errors
    ///
    /// Currently returns `Ok(Self)` as placeholder implementation.
    pub fn new() -> Result<Self, DatabaseError> {
        Ok(Self)
    }
}
