# Caxton System Architecture

**Version**: 1.0
**Date**: 2025-08-04
**Status**: Design Phase

## Executive Summary

Caxton is a production-ready multi-agent orchestration server that provides WebAssembly-based agent isolation, FIPA-compliant messaging, and comprehensive observability. This document defines the complete system architecture, domain model, and implementation patterns following type-driven development principles.

## Table of Contents

1. [System Overview](#system-overview)
2. [Domain Model](#domain-model)
3. [Component Architecture](#component-architecture)
4. [Agent Lifecycle Management](#agent-lifecycle-management)
5. [FIPA Message Flow](#fipa-message-flow)
6. [Security Architecture](#security-architecture)
7. [Observability Integration](#observability-integration)
8. [Performance & Scalability](#performance--scalability)
9. [Type Safety & Error Handling](#type-safety--error-handling)
10. [Deployment Architecture](#deployment-architecture)

## System Overview

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     Management Layer                        │
├─────────────────┬───────────────────┬───────────────────────┤
│   CLI Tool      │   Web Dashboard   │    gRPC/REST API     │
│   (caxton)      │   (Future)        │   (Management)       │
└─────────────────┴───────────────────┴───────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                  Caxton Server Process                      │
│  ┌───────────────────────────────────────────────────────┐ │
│  │              Management API Layer                     │ │
│  │    • Authentication • Authorization                   │ │
│  │    • Agent Deployment • Configuration                 │ │
│  └─────────────────┬─────────────────────────────────────┘ │
│                    │                                       │
│  ┌─────────────────▼─────────────────────────────────────┐ │
│  │            Agent Runtime Environment                  │ │
│  │                                                       │ │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐ │ │
│  │  │Agent A  │  │Agent B  │  │Agent C  │  │Agent N  │ │ │
│  │  │(WASM)   │  │(WASM)   │  │(WASM)   │  │(WASM)   │ │ │
│  │  │Sandbox  │  │Sandbox  │  │Sandbox  │  │Sandbox  │ │ │
│  │  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘ │ │
│  │       └────────────┼────────────┼────────────┘      │ │
│  │                    │            │                   │ │
│  │  ┌─────────────────▼────────────▼─────────────────┐ │ │
│  │  │           FIPA Message Bus                    │ │ │
│  │  │  • Message Routing  • Protocol Handling      │ │ │
│  │  │  • Conversation Mgmt • Error Recovery        │ │ │
│  │  └───────────────────────────────────────────────┘ │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              Observability Layer                   │ │
│  │  • Structured Logging (tracing crate)             │ │
│  │  • Metrics (Prometheus)                           │ │
│  │  • Distributed Tracing (OpenTelemetry)           │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │           Coordination Layer                       │ │
│  │  • Cluster Membership (SWIM Protocol)            │ │
│  │  • Agent Registry (Gossip)                       │ │
│  │  • Local State (SQLite)                          │ │
│  │  • Partition Detection & Handling                │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Type-Driven Design**: All illegal states are unrepresentable through the type system
2. **Observability First**: Every operation is traced, logged, and measured
3. **WebAssembly Isolation**: Complete sandboxing between agents
4. **FIPA Compliance**: Standard agent communication protocols
5. **Minimal Core**: Small, focused core with rich extension ecosystem
6. **Coordination Over State**: Use lightweight protocols instead of shared databases (see [ADR-0014](docs/adr/0014-coordination-first-architecture.md))

### Protocol Layering

- **SWIM Protocol (Infrastructure)**: Handles cluster membership, failure detection, and routing information dissemination
- **FIPA Protocol (Application)**: Manages semantic agent-to-agent messaging with conversation tracking
- **Clear Separation**: These protocols operate at different layers and complement each other (see [ADR-0015](docs/adr/0015-distributed-protocol-architecture.md))

## Domain Model

### Core Domain Types

```rust
// Agent Identity and Lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AgentId(NonZeroU64);

#[derive(Debug, Clone)]
pub struct Agent<State> {
    id: AgentId,
    name: AgentName,
    wasm_module: WasmModule,
    resources: ResourceLimits,
    _state: PhantomData<State>,
}

// Agent States - Make illegal state transitions impossible
pub struct Unloaded;
pub struct Loaded;
pub struct Running;
pub struct Draining;
pub struct Failed;

impl Agent<Unloaded> {
    pub fn load(self, module: WasmModule) -> Result<Agent<Loaded>, LoadError> {
        // Only unloaded agents can be loaded
    }
}

impl Agent<Loaded> {
    pub fn start(self) -> Result<Agent<Running>, StartError> {
        // Only loaded agents can start
    }
}

impl Agent<Running> {
    pub fn drain(self) -> Result<Agent<Draining>, DrainError> {
        // Only running agents can be drained
    }

    pub fn handle_message(&self, msg: FipaMessage) -> Result<(), ProcessingError> {
        // Only running agents can process messages
    }
}

// FIPA Message Domain Model
#[derive(Debug, Clone)]
pub struct FipaMessage {
    pub id: MessageId,
    pub performative: Performative,
    pub sender: AgentId,
    pub receiver: AgentId,
    pub conversation_id: ConversationId,
    pub reply_with: Option<ReplyWith>,
    pub in_reply_to: Option<InReplyTo>,
    pub content: MessageContent,
    pub ontology: Option<Ontology>,
    pub language: Option<Language>,
    // Observability context
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Performative {
    // Information exchange
    Inform,
    Query,
    Request,

    // Negotiation
    Propose,
    AcceptProposal,
    RejectProposal,

    // Contract Net Protocol
    Cfp,  // Call for Proposals

    // Error handling
    NotUnderstood,
    Failure,

    // Conversation management
    Cancel,
}

// Resource Management
#[derive(Debug, Clone)]
pub struct ResourceLimits {
    pub max_memory_bytes: ByteSize,
    pub max_cpu_millis: CpuMillis,
    pub max_execution_time: Duration,
    pub max_message_size: ByteSize,
}

// Deployment and Versioning
#[derive(Debug, Clone)]
pub struct Deployment {
    pub id: DeploymentId,
    pub agent_id: AgentId,
    pub version: Version,
    pub strategy: DeploymentStrategy,
    pub state: DeploymentState,
    pub created_at: SystemTime,
}

#[derive(Debug, Clone)]
pub enum DeploymentStrategy {
    Direct,
    BlueGreen { warm_up_duration: Duration },
    Canary {
        stages: Vec<CanaryStage>,
        rollback_conditions: RollbackConditions,
    },
    Shadow {
        duration: Duration,
        comparison_metrics: Vec<MetricComparison>,
    },
}

#[derive(Debug, Clone)]
pub struct CanaryStage {
    pub percentage: Percentage,
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentState {
    Validating,
    Deploying,
    Running,
    Draining,
    Failed { error: DeploymentError },
    RolledBack { reason: RollbackReason },
}
```

### Message Flow State Machine

```rust
// Conversation State Management
#[derive(Debug, Clone)]
pub struct Conversation {
    pub id: ConversationId,
    pub initiator: AgentId,
    pub participants: NonEmpty<AgentId>,
    pub protocol: InteractionProtocol,
    pub state: ConversationState,
    pub messages: Vec<FipaMessage>,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InteractionProtocol {
    RequestReply,
    ContractNet,
    Auction,
    Negotiation,
    Subscribe,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConversationState {
    Initiated,
    InProgress,
    Completed { outcome: ConversationOutcome },
    Failed { error: ConversationError },
    Expired,
}
```

## Component Architecture

### Agent Runtime Environment

```rust
pub struct AgentRuntime {
    agents: HashMap<AgentId, Arc<Agent<Running>>>,
    wasm_engine: WasmEngine,
    message_router: Arc<MessageRouter>,
    resource_manager: Arc<ResourceManager>,
    observability: Arc<ObservabilityLayer>,
}

impl AgentRuntime {
    pub async fn spawn_agent(
        &mut self,
        config: AgentConfig
    ) -> Result<AgentId, SpawnError> {
        let span = tracing::info_span!("agent_spawn", %config.name);
        let _enter = span.enter();

        // 1. Validate WASM module
        let module = self.validate_wasm_module(&config.wasm_bytes).await?;

        // 2. Create agent instance
        let agent = Agent::new(config.name, module)
            .load(config.wasm_bytes)?
            .start()?;

        // 3. Register with router
        self.message_router.register_agent(agent.id(), &agent).await?;

        // 4. Track resources
        self.resource_manager.track_agent(agent.id(), config.resources)?;

        // 5. Store running agent
        let agent_id = agent.id();
        self.agents.insert(agent_id, Arc::new(agent));

        // 6. Emit telemetry
        self.observability.record_agent_spawned(agent_id);

        Ok(agent_id)
    }

    #[instrument(skip(self, message))]
    pub async fn route_message(
        &self,
        message: FipaMessage
    ) -> Result<(), RoutingError> {
        // Message routing with full observability
        let agent = self.agents.get(&message.receiver)
            .ok_or(RoutingError::AgentNotFound(message.receiver))?;

        // Create child span for message processing
        let span = tracing::info_span!(
            "message_process",
            agent_id = %message.receiver,
            message_id = %message.id,
            performative = ?message.performative,
        );

        agent.handle_message(message).instrument(span).await
    }
}
```

### Message Router

```rust
#[async_trait]
pub trait MessageRouter: Send + Sync {
    async fn route(&self, message: FipaMessage) -> Result<(), RoutingError>;
    async fn register_agent(&self, id: AgentId, agent: &Agent<Running>) -> Result<(), RegistrationError>;
    async fn unregister_agent(&self, id: AgentId) -> Result<(), RegistrationError>;
}

pub struct FipaMessageRouter {
    routing_table: Arc<RwLock<HashMap<AgentId, Arc<Agent<Running>>>>>,
    conversation_manager: Arc<ConversationManager>,
    message_store: Arc<dyn MessageStore>,
    observability: Arc<ObservabilityLayer>,
}

impl FipaMessageRouter {
    #[instrument(skip(self, message))]
    pub async fn route(&self, message: FipaMessage) -> Result<(), RoutingError> {
        // 1. Validate message
        self.validate_message(&message)?;

        // 2. Store for persistence
        self.message_store.store_message(&message).await?;

        // 3. Update conversation state
        self.conversation_manager
            .update_conversation(&message)
            .await?;

        // 4. Find target agent
        let agents = self.routing_table.read().await;
        let target_agent = agents.get(&message.receiver)
            .ok_or(RoutingError::AgentNotFound(message.receiver))?;

        // 5. Deliver message
        target_agent.handle_message(message).await?;

        // 6. Record metrics
        self.observability.record_message_routed(&message);

        Ok(())
    }
}
```

## Agent Lifecycle Management

### Lifecycle State Machine

```rust
pub struct AgentLifecycleManager {
    state_store: Arc<dyn StateStore>,
    deployment_manager: Arc<DeploymentManager>,
    resource_manager: Arc<ResourceManager>,
    observability: Arc<ObservabilityLayer>,
}

impl AgentLifecycleManager {
    pub async fn deploy_agent(
        &self,
        config: DeploymentConfig
    ) -> Result<DeploymentId, DeploymentError> {
        let deployment_id = DeploymentId::new();
        let span = tracing::info_span!(
            "agent_deploy",
            %deployment_id,
            agent_name = %config.agent_name,
            strategy = ?config.strategy
        );
        let _enter = span.enter();

        // Create deployment record
        let deployment = Deployment {
            id: deployment_id,
            agent_id: config.agent_id,
            version: config.version,
            strategy: config.strategy.clone(),
            state: DeploymentState::Validating,
            created_at: SystemTime::now(),
        };

        // Store deployment
        self.state_store.store_deployment(&deployment).await?;

        // Execute deployment strategy
        match config.strategy {
            DeploymentStrategy::Direct => {
                self.execute_direct_deployment(deployment_id, config).await
            },
            DeploymentStrategy::BlueGreen { .. } => {
                self.execute_blue_green_deployment(deployment_id, config).await
            },
            DeploymentStrategy::Canary { .. } => {
                self.execute_canary_deployment(deployment_id, config).await
            },
            DeploymentStrategy::Shadow { .. } => {
                self.execute_shadow_deployment(deployment_id, config).await
            },
        }
    }

    #[instrument(skip(self, config))]
    async fn execute_canary_deployment(
        &self,
        deployment_id: DeploymentId,
        config: DeploymentConfig
    ) -> Result<DeploymentId, DeploymentError> {
        // 1. Validation phase
        self.update_deployment_state(deployment_id, DeploymentState::Validating).await?;
        let validation_result = self.validate_agent(&config).await?;

        if !validation_result.passed {
            return Err(DeploymentError::ValidationFailed(validation_result.errors));
        }

        // 2. Begin canary deployment
        self.update_deployment_state(deployment_id, DeploymentState::Deploying).await?;

        if let DeploymentStrategy::Canary { stages, rollback_conditions } = &config.strategy {
            for stage in stages {
                // Deploy to percentage of traffic
                self.deploy_canary_stage(deployment_id, stage).await?;

                // Monitor for rollback conditions
                let monitoring_result = self.monitor_canary_stage(
                    deployment_id,
                    stage,
                    rollback_conditions
                ).await?;

                if monitoring_result.should_rollback {
                    self.rollback_deployment(deployment_id, monitoring_result.reason).await?;
                    return Err(DeploymentError::RolledBack(monitoring_result.reason));
                }
            }
        }

        // 3. Complete deployment
        self.update_deployment_state(deployment_id, DeploymentState::Running).await?;
        self.observability.record_deployment_completed(deployment_id);

        Ok(deployment_id)
    }
}
```

## FIPA Message Flow

### Protocol Implementation

```rust
pub struct ContractNetProtocol {
    conversation_manager: Arc<ConversationManager>,
    message_router: Arc<dyn MessageRouter>,
    timeout_manager: Arc<TimeoutManager>,
}

impl ContractNetProtocol {
    #[instrument(skip(self, task))]
    pub async fn initiate_contract_net(
        &self,
        initiator: AgentId,
        task: Task,
        participants: NonEmpty<AgentId>
    ) -> Result<ContractResult, ProtocolError> {
        let conversation_id = ConversationId::new();
        let span = tracing::info_span!(
            "contract_net_protocol",
            %conversation_id,
            %initiator,
            participant_count = participants.len()
        );
        let _enter = span.enter();

        // 1. Create conversation
        let conversation = Conversation {
            id: conversation_id,
            initiator,
            participants: participants.clone(),
            protocol: InteractionProtocol::ContractNet,
            state: ConversationState::Initiated,
            messages: Vec::new(),
            created_at: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(300)),
        };

        self.conversation_manager.create_conversation(conversation).await?;

        // 2. Send Call for Proposals
        let cfp_message = FipaMessage {
            id: MessageId::new(),
            performative: Performative::Cfp,
            sender: initiator,
            receiver: AgentId::broadcast(), // Special broadcast ID
            conversation_id,
            reply_with: Some(ReplyWith::new()),
            in_reply_to: None,
            content: MessageContent::Task(task),
            ontology: Some(Ontology::ContractNet),
            language: Some(Language::Json),
            trace_id: Span::current().id().unwrap_or_default(),
            span_id: SpanId::new(),
            timestamp: SystemTime::now(),
        };

        // Broadcast to all participants
        for participant in &participants {
            let mut participant_message = cfp_message.clone();
            participant_message.receiver = *participant;
            self.message_router.route(participant_message).await?;
        }

        // 3. Collect proposals with timeout
        let proposals = self.collect_proposals(
            conversation_id,
            participants.len(),
            Duration::from_secs(30)
        ).await?;

        // 4. Evaluate and select winner
        let winning_proposal = self.evaluate_proposals(proposals)?;

        // 5. Send accept-proposal to winner
        let accept_message = FipaMessage {
            id: MessageId::new(),
            performative: Performative::AcceptProposal,
            sender: initiator,
            receiver: winning_proposal.sender,
            conversation_id,
            reply_with: Some(ReplyWith::new()),
            in_reply_to: Some(InReplyTo::from(winning_proposal.id)),
            content: MessageContent::Acceptance(winning_proposal.content.clone()),
            ontology: Some(Ontology::ContractNet),
            language: Some(Language::Json),
            trace_id: Span::current().id().unwrap_or_default(),
            span_id: SpanId::new(),
            timestamp: SystemTime::now(),
        };

        self.message_router.route(accept_message).await?;

        // 6. Send reject-proposal to others
        for proposal in &proposals {
            if proposal.sender != winning_proposal.sender {
                let reject_message = FipaMessage {
                    id: MessageId::new(),
                    performative: Performative::RejectProposal,
                    sender: initiator,
                    receiver: proposal.sender,
                    conversation_id,
                    reply_with: None,
                    in_reply_to: Some(InReplyTo::from(proposal.id)),
                    content: MessageContent::Rejection("Proposal not selected".to_string()),
                    ontology: Some(Ontology::ContractNet),
                    language: Some(Language::Json),
                    trace_id: Span::current().id().unwrap_or_default(),
                    span_id: SpanId::new(),
                    timestamp: SystemTime::now(),
                };

                self.message_router.route(reject_message).await?;
            }
        }

        // 7. Complete conversation
        self.conversation_manager.complete_conversation(
            conversation_id,
            ConversationOutcome::ContractAwarded {
                winner: winning_proposal.sender,
                task: task.clone(),
            }
        ).await?;

        Ok(ContractResult {
            conversation_id,
            winner: winning_proposal.sender,
            proposal: winning_proposal,
        })
    }
}
```

## Security Architecture

### WebAssembly Sandboxing

```rust
pub struct WasmSandbox {
    engine: wasmtime::Engine,
    store: wasmtime::Store<WasmContext>,
    instance: wasmtime::Instance,
    resource_limiter: ResourceLimiter,
}

#[derive(Debug)]
pub struct WasmContext {
    agent_id: AgentId,
    resource_limits: ResourceLimits,
    mcp_tools: Arc<McpToolRegistry>,
    observability: Arc<ObservabilityLayer>,
}

impl WasmSandbox {
    pub fn new(
        agent_id: AgentId,
        wasm_bytes: &[u8],
        resource_limits: ResourceLimits
    ) -> Result<Self, SandboxError> {
        // Create engine with security configurations
        let mut config = wasmtime::Config::new();
        config.wasm_simd(false);  // Disable SIMD for security
        config.wasm_reference_types(false);  // Disable ref types
        config.wasm_bulk_memory(false);  // Disable bulk memory
        config.consume_fuel(true);  // Enable fuel for CPU limiting

        let engine = wasmtime::Engine::new(&config)?;

        // Create store with resource limits
        let context = WasmContext {
            agent_id,
            resource_limits: resource_limits.clone(),
            mcp_tools: Arc::new(McpToolRegistry::new()),
            observability: Arc::new(ObservabilityLayer::new()),
        };

        let mut store = wasmtime::Store::new(&engine, context);
        store.limiter(|ctx| &mut ctx.resource_limits);

        // Set fuel limit for CPU control
        store.set_fuel(resource_limits.max_cpu_millis.into())?;

        // Load and instantiate module
        let module = wasmtime::Module::new(&engine, wasm_bytes)?;
        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

        Ok(Self {
            engine,
            store,
            instance,
            resource_limiter: ResourceLimiter::new(resource_limits),
        })
    }

    #[instrument(skip(self, message))]
    pub async fn handle_message(
        &mut self,
        message: FipaMessage
    ) -> Result<Option<FipaMessage>, SandboxError> {
        // Serialize message for WASM
        let message_bytes = serde_json::to_vec(&message)?;

        // Get WASM function
        let handle_message = self.instance
            .get_typed_func::<(i32, i32), i32>(&mut self.store, "handle_message")?;

        // Allocate memory in WASM instance
        let memory = self.instance.get_memory(&mut self.store, "memory")
            .ok_or(SandboxError::NoMemoryExport)?;

        let message_ptr = self.allocate_in_wasm(memory, &message_bytes)?;

        // Set fuel before execution
        self.store.set_fuel(self.resource_limiter.remaining_cpu())?;

        // Call WASM function with timeout
        let result = tokio::time::timeout(
            self.resource_limiter.max_execution_time(),
            async {
                handle_message.call_async(
                    &mut self.store,
                    (message_ptr, message_bytes.len() as i32)
                ).await
            }
        ).await??;

        // Check remaining fuel
        let consumed_fuel = self.resource_limiter.max_cpu_millis() - self.store.get_fuel()?;
        self.resource_limiter.consume_cpu(consumed_fuel)?;

        // Handle result
        match result {
            0 => Ok(None), // No response
            1 => {
                // Agent produced a response
                let response_bytes = self.get_response_from_wasm(memory)?;
                let response_message: FipaMessage = serde_json::from_slice(&response_bytes)?;
                Ok(Some(response_message))
            },
            error_code => Err(SandboxError::AgentError(error_code)),
        }
    }
}
```

### Resource Management

```rust
pub struct ResourceLimiter {
    max_memory_bytes: ByteSize,
    max_cpu_millis: CpuMillis,
    max_execution_time: Duration,

    consumed_memory: AtomicU64,
    consumed_cpu: AtomicU64,
    start_time: SystemTime,
}

impl ResourceLimiter {
    pub fn consume_memory(&self, bytes: u64) -> Result<(), ResourceError> {
        let current = self.consumed_memory.fetch_add(bytes, Ordering::SeqCst);
        if current + bytes > self.max_memory_bytes.as_u64() {
            Err(ResourceError::MemoryLimitExceeded {
                limit: self.max_memory_bytes,
                attempted: ByteSize::from(current + bytes),
            })
        } else {
            Ok(())
        }
    }

    pub fn consume_cpu(&self, millis: u64) -> Result<(), ResourceError> {
        let current = self.consumed_cpu.fetch_add(millis, Ordering::SeqCst);
        if current + millis > self.max_cpu_millis.as_u64() {
            Err(ResourceError::CpuLimitExceeded {
                limit: self.max_cpu_millis,
                attempted: CpuMillis::from(current + millis),
            })
        } else {
            Ok(())
        }
    }
}
```

## Observability Integration

### Structured Logging and Tracing

```rust
pub struct ObservabilityLayer {
    tracer: opentelemetry::global::Tracer,
    metrics_registry: Arc<MetricsRegistry>,
    event_store: Arc<dyn EventStore>,
}

impl ObservabilityLayer {
    pub fn record_agent_event<T>(&self, event: AgentEvent<T>)
    where T: Serialize + Send + Sync + 'static {
        // Structured logging
        tracing::info!(
            agent_id = %event.agent_id,
            event_type = std::any::type_name::<T>(),
            timestamp = ?event.timestamp,
            trace_id = %event.trace_id,
            "Agent event recorded"
        );

        // Store event for analysis
        self.event_store.store_event(event);

        // Update metrics
        self.metrics_registry.increment_counter(
            "caxton_agent_events_total",
            &[("agent_id", &event.agent_id.to_string())]
        );
    }

    pub fn record_message_metrics(&self, message: &FipaMessage, duration: Duration) {
        // Histogram for message processing time
        self.metrics_registry.record_histogram(
            "caxton_message_processing_duration_seconds",
            duration.as_secs_f64(),
            &[
                ("performative", &format!("{:?}", message.performative)),
                ("sender", &message.sender.to_string()),
                ("receiver", &message.receiver.to_string()),
            ]
        );

        // Counter for message throughput
        self.metrics_registry.increment_counter(
            "caxton_messages_processed_total",
            &[
                ("performative", &format!("{:?}", message.performative)),
                ("status", "success"),
            ]
        );
    }
}

// Agent Event Types for Observability
#[derive(Debug, Serialize)]
pub struct AgentEvent<T> {
    pub agent_id: AgentId,
    pub timestamp: SystemTime,
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub event_data: T,
}

#[derive(Debug, Serialize)]
pub struct LifecycleEvent {
    pub old_state: Option<String>,
    pub new_state: String,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CommunicationEvent {
    pub message_id: MessageId,
    pub conversation_id: ConversationId,
    pub performative: Performative,
    pub direction: MessageDirection,
}

#[derive(Debug, Serialize)]
pub struct PerformanceEvent {
    pub cpu_usage: f64,
    pub memory_usage: ByteSize,
    pub message_queue_size: usize,
    pub active_conversations: usize,
}
```

## Performance & Scalability

### Scalability Targets

- **Single Instance**: 1,000+ concurrent agents
- **Message Throughput**: 100,000+ messages/second
- **Latency**: < 1ms p99 for local message routing
- **Memory Efficiency**: < 1MB per idle agent
- **Startup Time**: < 100ms for new agent deployment

### Performance Optimizations

```rust
pub struct PerformanceOptimizedRuntime {
    // Agent pool for reusing WASM instances
    agent_pool: Arc<AgentPool>,

    // Message batching for throughput
    message_batcher: Arc<MessageBatcher>,

    // Lock-free message queue
    message_queue: Arc<crossbeam::queue::SegQueue<FipaMessage>>,

    // CPU-intensive work executor
    cpu_executor: Arc<tokio::task::JoinSet<()>>,

    // I/O bound work executor
    io_executor: Arc<tokio::task::JoinSet<()>>,
}

impl PerformanceOptimizedRuntime {
    pub async fn process_message_batch(&self) -> Result<(), ProcessingError> {
        let batch = self.message_batcher.get_batch().await?;

        // Process messages in parallel
        let mut tasks = Vec::new();
        for message in batch {
            let agent_pool = Arc::clone(&self.agent_pool);
            let task = tokio::spawn(async move {
                let agent = agent_pool.get_agent(message.receiver).await?;
                agent.handle_message(message).await
            });
            tasks.push(task);
        }

        // Wait for all messages to complete
        futures::future::try_join_all(tasks).await?;

        Ok(())
    }
}

// Agent Pool for Instance Reuse
pub struct AgentPool {
    available_agents: Arc<Mutex<HashMap<AgentId, VecDeque<Agent<Running>>>>>,
    max_pool_size: usize,
}

impl AgentPool {
    pub async fn get_agent(&self, agent_id: AgentId) -> Result<Agent<Running>, PoolError> {
        let mut pool = self.available_agents.lock().await;

        if let Some(agents) = pool.get_mut(&agent_id) {
            if let Some(agent) = agents.pop_front() {
                return Ok(agent);
            }
        }

        // No pooled instance available, create new one
        self.create_fresh_agent(agent_id).await
    }

    pub async fn return_agent(&self, agent: Agent<Running>) -> Result<(), PoolError> {
        let mut pool = self.available_agents.lock().await;
        let agent_id = agent.id();

        let agents = pool.entry(agent_id).or_insert_with(VecDeque::new);

        if agents.len() < self.max_pool_size {
            agents.push_back(agent);
        }
        // Otherwise, drop the agent to reclaim memory

        Ok(())
    }
}
```

## Type Safety & Error Handling

### Comprehensive Error Model

```rust
// Top-level Error Type
#[derive(Debug, Error)]
pub enum CaxtonError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("Message routing error: {0}")]
    Routing(#[from] RoutingError),

    #[error("Deployment error: {0}")]
    Deployment(#[from] DeploymentError),

    #[error("Resource error: {0}")]
    Resource(#[from] ResourceError),

    #[error("Security error: {0}")]
    Security(#[from] SecurityError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),

    #[error("Observability error: {0}")]
    Observability(#[from] ObservabilityError),
}

// Domain-specific Error Types
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent {agent_id} not found")]
    NotFound { agent_id: AgentId },

    #[error("Agent {agent_id} is in state {current_state}, cannot perform {operation}")]
    InvalidState {
        agent_id: AgentId,
        current_state: String,
        operation: String,
    },

    #[error("WASM execution failed: {reason}")]
    WasmExecutionFailed { reason: String },

    #[error("Agent {agent_id} exceeded resource limit: {limit_type}")]
    ResourceLimitExceeded {
        agent_id: AgentId,
        limit_type: String,
    },
}

#[derive(Debug, Error)]
pub enum RoutingError {
    #[error("No route found for agent {agent_id}")]
    NoRoute { agent_id: AgentId },

    #[error("Message {message_id} delivery failed: {reason}")]
    DeliveryFailed {
        message_id: MessageId,
        reason: String,
    },

    #[error("Invalid message format: {details}")]
    InvalidMessageFormat { details: String },

    #[error("Conversation {conversation_id} not found")]
    ConversationNotFound { conversation_id: ConversationId },
}

// Railway-Oriented Programming Pattern
pub type CaxtonResult<T> = Result<T, CaxtonError>;

pub trait CaxtonResultExt<T> {
    fn with_context<F>(self, f: F) -> CaxtonResult<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> CaxtonResultExt<T> for Result<T, E>
where
    E: Into<CaxtonError>
{
    fn with_context<F>(self, f: F) -> CaxtonResult<T>
    where
        F: FnOnce() -> String
    {
        self.map_err(|e| {
            let context = f();
            tracing::error!(error = %e.into(), context = %context, "Operation failed");
            e.into()
        })
    }
}
```

### Smart Constructors for Type Safety

```rust
// Smart constructor pattern ensures validation
impl AgentId {
    pub fn new() -> Self {
        Self(NonZeroU64::new(rand::random()).expect("random u64 is non-zero"))
    }

    // Parse from string with validation
    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let id = s.parse::<u64>()
            .map_err(|_| ParseError::InvalidFormat)?;

        let non_zero_id = NonZeroU64::new(id)
            .ok_or(ParseError::ZeroId)?;

        Ok(Self(non_zero_id))
    }
}

impl MessageId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, ParseError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ParseError::InvalidUuid)?;
        Ok(Self(uuid))
    }
}

// Percentage type that ensures valid range
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Percentage(f64);

impl Percentage {
    pub fn new(value: f64) -> Result<Self, ValidationError> {
        if !(0.0..=100.0).contains(&value) {
            return Err(ValidationError::InvalidPercentage { value });
        }
        Ok(Self(value))
    }

    pub fn from_ratio(ratio: f64) -> Result<Self, ValidationError> {
        if !(0.0..=1.0).contains(&ratio) {
            return Err(ValidationError::InvalidRatio { ratio });
        }
        Ok(Self(ratio * 100.0))
    }

    pub fn as_ratio(&self) -> f64 {
        self.0 / 100.0
    }

    pub fn as_percentage(&self) -> f64 {
        self.0
    }
}
```

## Deployment Architecture

### Production Deployment Pattern

```yaml
# docker-compose.yml for production deployment
version: '3.8'

services:
  caxton-server:
    image: caxton/caxton:latest
    ports:
      - "8080:8080"    # Management API
      - "9090:9090"    # Metrics endpoint
    environment:
      - CAXTON_CONFIG_PATH=/etc/caxton/config.yaml
      - RUST_LOG=info
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:14268
    volumes:
      - ./config/caxton.yaml:/etc/caxton/config.yaml:ro
      - caxton-data:/var/lib/caxton
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped
    depends_on:
      - postgres
      - jaeger

  postgres:
    image: postgres:15-alpine
    environment:
      - POSTGRES_DB=caxton
      - POSTGRES_USER=caxton
      - POSTGRES_PASSWORD=secure_password
    volumes:
      - postgres-data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # UI
      - "14268:14268"  # HTTP collector
    environment:
      - COLLECTOR_OTLP_ENABLED=true

volumes:
  caxton-data:
  postgres-data:
```

### Kubernetes Deployment

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: caxton-server
  labels:
    app: caxton
spec:
  serviceName: caxton
  replicas: 3
  selector:
    matchLabels:
      app: caxton
  template:
    metadata:
      labels:
        app: caxton
    spec:
      containers:
      - name: caxton
        image: caxton/caxton:latest
        ports:
        - containerPort: 8080
          name: api
        - containerPort: 9090
          name: metrics
        env:
        - name: CAXTON_CONFIG_PATH
          value: /etc/caxton/config.yaml
        - name: RUST_LOG
          value: info
        volumeMounts:
        - name: config
          mountPath: /etc/caxton
        - name: data
          mountPath: /var/lib/caxton
        livenessProbe:
          httpGet:
            path: /health
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
        readinessProbe:
          httpGet:
            path: /ready
            port: 8080
          initialDelaySeconds: 5
          periodSeconds: 5
        resources:
          requests:
            memory: "512Mi"
            cpu: "250m"
          limits:
            memory: "2Gi"
            cpu: "1000m"
      volumes:
      - name: config
        configMap:
          name: caxton-config
  volumeClaimTemplates:
  - metadata:
      name: data
    spec:
      accessModes: [ "ReadWriteOnce" ]
      resources:
        requests:
          storage: 10Gi
```

## Summary

This architecture provides:

1. **Type Safety**: Comprehensive domain modeling with phantom types and smart constructors
2. **Observability**: Built-in tracing, metrics, and structured logging
3. **Security**: WebAssembly sandboxing with resource limits
4. **Scalability**: Performance-optimized runtime with agent pooling
5. **Reliability**: Comprehensive error handling and fault tolerance
6. **Production Ready**: Full deployment and operational patterns

The architecture follows type-driven development principles, making illegal states unrepresentable while providing comprehensive observability and security for production multi-agent systems.
