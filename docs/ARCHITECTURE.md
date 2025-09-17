# Caxton System Architecture

**Version**: 2.0 **Date**: 2025-09-10 **Status**: Design Phase

> **🚧 Implementation Status**
>
> This architecture document serves as the technical specification and
> acceptance criteria for development. The system design reflects ADRs 28-30
> architectural decisions and represents the target implementation goals.
>
> **Current State**: Type system and domain modeling foundation
> **Implementation Progress**: Core domain types and architectural
> patterns being established
>
> All features and components described represent planned functionality aligned
> with the hybrid agent architecture vision.

## Executive Summary

Caxton is a production-ready multi-agent orchestration server that provides
**configuration-driven agents** as the primary user experience. The system
offers
simple agent communication, embedded memory capabilities, and comprehensive
observability. WebAssembly is used for deployable MCP servers that provide tools
to agents. This document defines the complete architecture, domain model, and
implementation patterns following type-driven development principles.

## Table of Contents

1.  [System Overview](#system-overview)
2.  [Domain Model](#domain-model)
3.  [Component Architecture](#component-architecture)
4.  [Agent Lifecycle Management](#agent-lifecycle-management)
5.  [FIPA Message Flow](#fipa-message-flow)
6.  [Security Architecture](#security-architecture)
7.  [Observability Integration](#observability-integration)
8.  [Performance & Scalability](#performance--scalability)
9.  [Type Safety & Error Handling](#type-safety--error-handling)
10. [Deployment Architecture](#deployment-architecture)

## System Overview

### High-Level Hybrid Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                     Management Layer                        │
├─────────────────┬───────────────────┬───────────────────────┤
│   CLI Tool      │   Web Dashboard   │    REST/HTTP API     │
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
│  │            Agent Runtime Environment                 │ │
│  │                                                       │ │
│  │ Simple Agents (Primary UX - 5-10 min setup)          │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐     │ │
│  │  │Agent A     │  │Agent B     │  │Agent C     │     │ │
│  │  │(TOML)      │  │(TOML)      │  │(TOML)      │     │ │
│  │  │Host Runtime│  │Host Runtime│  │Host Runtime│     │ │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘     │ │
│  │         └───────────────┼───────────────┘           │ │
│  │                         │                           │ │
│  │  ┌─────────────────────▼─────────────────────────┐  │ │
│  │  │         Simple Message Router                 │  │ │
│  │  │  • Message Routing • Conversation Mgmt        │  │ │
│  │  │  • Protocol Handling  • Error Recovery        │  │ │
│  │  └─────────────────────────────────────────────────┘  │ │
│  │                                                       │ │
│  │  ┌─────────────────────────────────────────────────┐  │ │
│  │  │         Deployable MCP Servers (WASM)          │  │ │
│  │  │  • HTTP Client    • CSV Parser                 │  │ │
│  │  │  • Chart Gen      • File System                │  │ │
│  │  │  • Database       • Custom Tools                │  │ │
│  │  └─────────────────────────────────────────────────┘  │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │           Embedded Memory System                   │ │
│  │  • SQLite + Candle (Default)                      │ │
│  │  • Entity-Relationship Storage                    │ │
│  │  • Semantic Search (All-MiniLM-L6-v2)            │ │
│  │  • Optional External Backends                     │ │
│  └─────────────────────────────────────────────────────┘ │
│                                                           │
│  ┌─────────────────────────────────────────────────────┐ │
│  │              Observability Layer                   │ │
│  │  • Structured Logging (tracing crate)             │ │
│  │  • Metrics (Prometheus)                           │ │
│  │  • Distributed Tracing (OpenTelemetry)           │ │
│  └─────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

### Core Principles

1. **Configuration First**: 5-10 minute agent creation through TOML
   configuration files (ADR-0032)
2. **Simple Agent Runtime**: Configuration agents provide simple orchestration
   without complex sandboxing
3. **Type-Driven Design**: All illegal states are unrepresentable through the
   type system
4. **Observability First**: Every operation is traced, logged, and measured
5. **Zero Dependencies by Default**: Zero external dependencies by default, with
   pluggable external backends available for scaling (ADR-0030)
6. **Simple Messaging**: Lightweight agent communication patterns
7. **MCP Server Deployment**: WebAssembly used for deployable MCP servers that
   provide tools to agents
8. **Pluggable LLM Integration**: Users can integrate any LLM/SLM API through
   configurable provider system. OpenAI chat completion provided as default
   reference implementation

### Agent Types and User Journey

> **🚧 Planned Feature**
> These agent types represent the target user experience designed in ADR-28.
> Implementation is in progress following type-driven development principles.

**Simple Agents (Primary use cases)**:

- **Definition**: TOML configuration files with embedded documentation
- **Capabilities**: Declare what they can do (e.g., "data-analysis")
- **Tools**: Allowlist of MCP tools they can access
- **Runtime**: Executed in host process with LLM orchestration
- **Setup Time**: 5-10 minutes from idea to working agent
- **Security**: Simple agents focus on orchestration; complex operations
  delegated to MCP servers

**Deployable MCP Servers (Tool providers)**:

- **Definition**: WebAssembly modules that provide tools to agents
- **Use Cases**: File operations, HTTP requests, data processing, custom
  algorithms
- **Languages**: Rust, JavaScript, Python, Go, or any WASM-compatible language
- **Runtime**: Executed in isolated WASM sandboxes with resource limits
- **Setup Time**: Variable depending on complexity

## Domain Model

### Simple Agent Domain Model

```toml
# Example Configuration Agent Definition
name = "DataAnalyzer"
version = "1.0.0"
capabilities = ["data-analysis", "report-generation"]
tools = ["http_client", "csv_parser", "chart_generator"]

[memory]
enabled = true
scope = "workspace"

[parameters]
max_file_size = "10MB"
supported_formats = ["csv", "json", "xlsx"]

system_prompt = '''
You are a data analysis expert who helps users understand their data.
You can fetch data from URLs, parse various formats, and create
visualizations.
'''

user_prompt_template = '''
Analyze the following data request: {{request}}

Available data: {{context}}
User requirements: {{requirements}}
'''

documentation = '''
# DataAnalyzer Agent

This agent specializes in data analysis tasks and can:
- Fetch data from HTTP endpoints
- Parse CSV, JSON, and Excel files
- Generate charts and visualizations
- Provide statistical summaries
'''
```

### MCP Server Domain Model (Tool Providers)

```rust
// MCP Server Identity and Lifecycle
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct McpServerId(NonZeroU64);

#[derive(Debug, Clone)]
pub struct McpServer<State> {
    id: McpServerId,
    name: McpServerName,
    wasm_module: WasmModule,
    tools: Vec<ToolDefinition>,
    resources: ResourceLimits,
    _state: PhantomData<State>,
}

// MCP Server States - Make illegal state transitions impossible
pub struct Unloaded;
pub struct Loaded;
pub struct Running;
pub struct Draining;
pub struct Failed;

impl McpServer<Unloaded> {
    pub fn load(self, module: WasmModule)
        -> Result<McpServer<Loaded>, LoadError> {
        // Only unloaded servers can be loaded
    }
}

impl McpServer<Loaded> {
    pub fn start(self) -> Result<McpServer<Running>, StartError> {
        // Only loaded servers can start
    }
}

impl McpServer<Running> {
    pub fn drain(self) -> Result<McpServer<Draining>, DrainError> {
        // Only running servers can be drained
    }

    pub fn handle_tool_call(
        &self,
        tool_call: ToolCall
    ) -> Result<ToolResult, ProcessingError> {
        // Only running servers can process tool calls
    }
}

// Simple Message Domain Model
#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub message_type: MessageType,
    pub sender: AgentId,
    pub receiver: Option<AgentId>,
    pub conversation_id: ConversationId,
    pub reply_to: Option<MessageId>,
    pub content: MessageContent,
    // Observability context
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    // Basic communication
    Request,
    Response,
    Notification,

    // Error handling
    Error,
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
    async fn register_agent(
        &self,
        id: AgentId,
        agent: &Agent<Running>
    ) -> Result<(), RegistrationError>;
    async fn unregister_agent(
        &self,
        id: AgentId
    ) -> Result<(), RegistrationError>;
}

pub struct FipaMessageRouter {
    routing_table: Arc<RwLock<HashMap<AgentId, Arc<Agent<Running>>>>>,
    conversation_manager: Arc<ConversationManager>,
    message_store: Arc<dyn MessageStore>,
    observability: Arc<ObservabilityLayer>,
}

impl FipaMessageRouter {
    #[instrument(skip(self, message))]
    pub async fn route(
        &self,
        message: FipaMessage
    ) -> Result<(), RoutingError> {
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
        self.update_deployment_state(
            deployment_id,
            DeploymentState::Validating
        ).await?;
        let validation_result = self.validate_agent(&config).await?;

        if !validation_result.passed {
            return Err(DeploymentError::ValidationFailed(
                validation_result.errors
            ));
        }

        // 2. Begin canary deployment
        self.update_deployment_state(
            deployment_id,
            DeploymentState::Deploying
        ).await?;

        if let DeploymentStrategy::Canary {
            stages,
            rollback_conditions
        } = &config.strategy {
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
                    self.rollback_deployment(
                        deployment_id,
                        monitoring_result.reason
                    ).await?;
                    return Err(DeploymentError::RolledBack(
                        monitoring_result.reason
                    ));
                }
            }
        }

        // 3. Complete deployment
        self.update_deployment_state(
            deployment_id,
            DeploymentState::Running
        ).await?;
        self.observability.record_deployment_completed(deployment_id);

        Ok(deployment_id)
    }
}
```

## Agent Communication

### Simple Message Routing

The Caxton messaging system implements simple agent communication
optimized for configuration-driven agents with straightforward routing.

#### Message Structure

```rust
#[derive(Debug, Clone)]
pub struct Message {
    pub id: MessageId,
    pub message_type: MessageType,
    pub sender: AgentId,
    pub receiver: Option<AgentId>,
    pub conversation_id: ConversationId,
    pub reply_to: Option<MessageId>,
    pub content: MessageContent,

    // Observability context
    pub trace_id: TraceId,
    pub span_id: SpanId,
    pub timestamp: SystemTime,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    // Basic Communication
    Request,
    Response,
    Notification,

    // Error Handling
    Error,
}
```

#### Simple Message Routing

Messages are routed between agents based on direct addressing or simple
patterns:

```rust
pub struct SimpleRouter {
    agent_registry: Arc<AgentRegistry>,
    conversation_manager: Arc<ConversationManager>,
    agent_runtime: Arc<AgentRuntime>,
}

impl SimpleRouter {
    #[instrument(skip(self, message))]
    pub async fn route_message(
        &self,
        message: Message
    ) -> Result<(), RoutingError> {
        // 1. Find target agent
        let target_agent_id = match message.receiver {
            Some(agent_id) => agent_id,
            None => {
                return Err(RoutingError::NoReceiver {
                    message_id: message.id,
                });
            }
        };

        // 2. Route to agent
        if self.agent_runtime.has_agent(target_agent_id).await? {
            self.route_to_agent(message).await?;
        } else {
            return Err(RoutingError::AgentNotFound(target_agent_id));
        }

        Ok(())
    }

    async fn route_to_agent(
        &self,
        message: Message
    ) -> Result<(), RoutingError> {
        // Convert message to natural language prompt for agent
        let prompt_context = self.format_message_as_prompt(&message).await?;

        // Execute agent with message context
        let agent_response = self.agent_runtime
            .execute_agent(message.receiver.unwrap(), prompt_context)
            .await?;

        // Parse agent response for follow-up messages
        if let Some(response_message) = self.parse_agent_response(
            &agent_response,
            &message
        ).await? {
            self.route_message(response_message).await?;
        }

        Ok(())
    }

    async fn format_message_as_prompt(
        &self,
        message: &Message
    ) -> Result<String, RoutingError> {
        match message.message_type {
            MessageType::Request => {
                format!(
                    "A user is requesting: {}\n\nPlease help them.",
                    message.content.as_text()
                )
            },
            MessageType::Notification => {
                format!(
                    "Information update: {}\n\n\
                     Please process this information.",
                    message.content.as_text()
                )
            },
            _ => {
                format!(
                    "Message: {}\n\nPlease respond appropriately.",
                    message.content.as_text()
                )
            },
        }
    }
}
```

## Security Architecture

### Simple Security Model

The Caxton security model focuses on practical isolation where it matters most -
at the MCP server level rather than agent level for simple agents.

#### Simple Agent Security

Simple agents run in the host runtime without WebAssembly isolation because:

- They contain only orchestration logic (no direct system access)
- All dangerous operations are delegated to MCP servers
- They operate through LLM calls that are naturally constrained
- Rapid development requires minimal friction

#### MCP Server Sandboxing (Primary Security Boundary)

All actual system access happens through MCP (Model Context Protocol) servers
that run in isolated WebAssembly sandboxes:

```rust
pub struct McpServerSandbox {
    engine: wasmtime::Engine,
    store: wasmtime::Store<McpContext>,
    instance: wasmtime::Instance,
    resource_limiter: ResourceLimiter,
    capability_allowlist: CapabilityAllowlist,
}

#[derive(Debug)]
pub struct McpContext {
    server_id: McpServerId,
    requesting_agent: AgentId,
    resource_limits: ResourceLimits,
    allowed_capabilities: CapabilityAllowlist,
    observability: Arc<ObservabilityLayer>,
}

impl McpServerSandbox {
    pub fn new(
        server_id: McpServerId,
        wasm_bytes: &[u8],
        capabilities: CapabilityAllowlist,
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
        let context = McpContext {
            server_id: tool_id,
            requesting_agent: AgentId::system(),
            resource_limits: resource_limits.clone(),
            allowed_capabilities: capabilities,
            observability: Arc::new(ObservabilityLayer::new()),
        };

        let mut store = wasmtime::Store::new(&engine, context);
        store.limiter(|ctx| &mut ctx.resource_limits);
        store.set_fuel(resource_limits.max_cpu_millis.into())?;

        // Load and instantiate module with host function restrictions
        let module = wasmtime::Module::new(&engine, wasm_bytes)?;
        let instance = wasmtime::Instance::new(&mut store, &module, &[])?;

        Ok(Self {
            engine,
            store,
            instance,
            resource_limiter: ResourceLimiter::new(resource_limits),
            capability_allowlist: capabilities,
        })
    }
}

impl McpServerSandbox {
    #[instrument(skip(self, tool_call))]
    pub async fn handle_tool_call(
        &mut self,
        tool_call: ToolCall
    ) -> Result<ToolResult, SandboxError> {
        // Serialize tool call for WASM
        let call_bytes = serde_json::to_vec(&tool_call)?;

        // Get WASM function
        let handle_call = self.instance
            .get_typed_func::<(i32, i32), i32>(
                &mut self.store,
                "handle_tool_call"
            )?;

        // Allocate memory in WASM instance
        let memory = self.instance.get_memory(&mut self.store, "memory")
            .ok_or(SandboxError::NoMemoryExport)?;

        let call_ptr = self.allocate_in_wasm(memory, &call_bytes)?;

        // Set fuel before execution
        self.store.set_fuel(self.resource_limiter.remaining_cpu())?;

        // Call WASM function with timeout
        let result = tokio::time::timeout(
            self.resource_limiter.max_execution_time(),
            async {
                handle_call.call_async(
                    &mut self.store,
                    (call_ptr, call_bytes.len() as i32)
                ).await
            }
        ).await??;

        // Check remaining fuel
        let consumed_fuel = self.resource_limiter.max_cpu_millis()
            - self.store.get_fuel()?;
        self.resource_limiter.consume_cpu(consumed_fuel)?;

        // Handle result
        match result {
            0 => {
                // Get tool result from WASM
                let result_bytes = self.get_response_from_wasm(memory)?;
                let tool_result: ToolResult =
                    serde_json::from_slice(&result_bytes)?;
                Ok(tool_result)
            },
            error_code => Err(SandboxError::ToolError(error_code)),
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

## Embedded Memory System (ADR-0030)

> **🚧 Planned Feature**
> The embedded memory system represents a key architectural component from
> ADR-30. SQLite + Candle implementation is being developed as part of the core
> platform functionality.

### Zero-Configuration Memory Architecture

Caxton provides an embedded memory system that works immediately without
external dependencies by default. Embedded backend scales to 100K+ entities,
with optional migration to external backends (Neo4j, Qdrant) for larger
deployments.

#### Default: SQLite + Candle Implementation

The embedded backend combines SQLite for structured storage with local
embedding models for semantic search:

```rust
pub struct EmbeddedMemorySystem {
    db: Arc<SqlitePool>,
    embeddings: Arc<CandleEmbeddings>,
    entity_store: Arc<EntityStore>,
    relation_store: Arc<RelationStore>,
    semantic_search: Arc<SemanticSearchEngine>,
}

pub struct CandleEmbeddings {
    model: SentenceEmbeddingsModel,  // All-MiniLM-L6-v2
    tokenizer: Arc<Tokenizer>,
    device: Device,
}

impl EmbeddedMemorySystem {
    pub fn new(db_path: &Path) -> Result<Self, MemoryError> {
        // Initialize SQLite with vector extension
        let db = SqlitePool::connect(&format!("sqlite://{}", db_path.display()))
            .await?;

        // Load All-MiniLM-L6-v2 embedding model (~23MB)
        let embeddings = CandleEmbeddings::load_model("all-MiniLM-L6-v2")?;

        // Initialize storage layers
        let entity_store = Arc::new(EntityStore::new(db.clone()));
        let relation_store = Arc::new(RelationStore::new(db.clone()));
        let semantic_search = Arc::new(
            SemanticSearchEngine::new(db.clone(), embeddings.clone())
        );

        Ok(Self {
            db,
            embeddings,
            entity_store,
            relation_store,
            semantic_search,
        })
    }

    #[instrument(skip(self))]
    pub async fn store_entity(
        &self,
        entity: Entity
    ) -> Result<EntityId, MemoryError> {
        // Generate embeddings for entity observations
        let embedding = self.embeddings
            .encode_observations(&entity.observations)
            .await?;

        // Store entity with embedding
        let entity_id = self.entity_store
            .create_entity(&entity, embedding)
            .await?;

        // Update semantic search index
        self.semantic_search
            .index_entity(entity_id, &entity, embedding)
            .await?;

        Ok(entity_id)
    }

    #[instrument(skip(self))]
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        min_similarity: f32
    ) -> Result<Vec<EntityMatch>, MemoryError> {
        // Generate query embedding
        let query_embedding = self.embeddings.encode_text(query).await?;

        // Perform vector similarity search
        let matches = self.semantic_search
            .find_similar(query_embedding, limit, min_similarity)
            .await?;

        Ok(matches)
    }
}
```

#### Agent Memory Integration

Configuration agents can enable memory through their TOML configuration:

```toml
name = "DataAnalyzer"

[memory]
enabled = true
scope = "workspace"  # agent-only, workspace, or global
auto_store = true    # Automatically store successful interactions
search_limit = 10
min_similarity = 0.7
```

#### Memory Scopes and Isolation

- **Agent-only**: Private memory per agent instance (isolated namespace)
- **Workspace**: Shared memory within a project/workspace context
- **Global**: System-wide shared knowledge base across all agents

#### Performance Characteristics

**Embedded Backend Performance**:

- **Semantic search**: 10-50ms for 100K entities
- **Graph traversal**: 5-20ms for typical relationship queries
- **Memory usage**: ~200MB baseline (embedding model + cache)
- **Storage**: ~2.5KB per entity (including 384-dim embedding)
- **Scaling limit**: Embedded backend scales to 100K+ entities, with optional
  migration to external backends for larger deployments

#### Pluggable Backend Architecture

For larger deployments requiring scale beyond 100K+ entities, external backends
can be configured as pluggable alternatives:

```rust
pub enum MemoryBackend {
    Embedded(EmbeddedMemorySystem),
    Neo4j(Neo4jBackend),
    Qdrant(QdrantBackend),
    Custom(Box<dyn MemoryBackend>),
}

pub trait MemoryBackend: Send + Sync {
    async fn store_entity(&self, entity: Entity)
        -> Result<EntityId, MemoryError>;
    async fn find_entities(&self, query: &EntityQuery)
        -> Result<Vec<Entity>, MemoryError>;
    async fn store_relation(&self, relation: Relation)
        -> Result<RelationId, MemoryError>;
    async fn semantic_search(&self, query: &str, limit: usize)
        -> Result<Vec<EntityMatch>, MemoryError>;
    async fn export_data(&self) -> Result<MemoryExport, MemoryError>;
    async fn import_data(&self, data: MemoryImport) -> Result<(), MemoryError>;
}
```

#### Migration and Data Portability

The memory system provides standard JSON export/import functionality:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct MemoryExport {
    pub entities: Vec<Entity>,
    pub relations: Vec<Relation>,
    pub metadata: ExportMetadata,
    pub version: String,
}

impl EmbeddedMemorySystem {
    pub async fn export_all(&self) -> Result<MemoryExport, MemoryError> {
        let entities = self.entity_store.get_all_entities().await?;
        let relations = self.relation_store.get_all_relations().await?;

        Ok(MemoryExport {
            entities,
            relations,
            metadata: ExportMetadata::new(),
            version: "1.0".to_string(),
        })
    }

    pub async fn import_from_export(&self, export: MemoryExport)
        -> Result<(), MemoryError> {
        // Validate version compatibility
        self.validate_export_version(&export.version)?;

        // Import entities with new embeddings
        for entity in export.entities {
            self.store_entity(entity).await?;
        }

        // Import relations
        for relation in export.relations {
            self.relation_store.create_relation(relation).await?;
        }

        Ok(())
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

    pub fn record_message_metrics(
        &self,
        message: &FipaMessage,
        duration: Duration
    ) {
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

### Hybrid Performance Characteristics

**Simple Agents (Primary UX)**:

- **Startup Time**: < 50ms (TOML parsing + prompt loading)
- **Memory Efficiency**: < 100KB per idle agent
- **Message Latency**: ~10-100ms (includes LLM orchestration)
- **Throughput**: 100-1,000 messages/second per agent (LLM dependent)
- **Concurrent Agents**: 10,000+ agents per instance

**MCP Servers (Tool Providers)**:

- **Startup Time**: < 100ms (WASM instantiation)
- **Memory Efficiency**: < 1MB per idle MCP server
- **Tool Call Latency**: < 1ms p99 for local processing
- **Throughput**: 100,000+ tool calls/second (native-like performance)
- **Concurrent Servers**: 1,000+ MCP servers per instance

**Memory System Performance**:

- **Semantic Search**: 10-50ms for 100K entities (embedded backend)
- **Entity Storage**: 5-20ms per entity with embeddings
- **Memory Baseline**: ~200MB (All-MiniLM-L6-v2 model)
- **Scaling Limit**: 100K entities recommended for embedded backend

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
    pub async fn get_agent(
        &self,
        agent_id: AgentId
    ) -> Result<Agent<Running>, PoolError> {
        let mut pool = self.available_agents.lock().await;

        if let Some(agents) = pool.get_mut(&agent_id) {
            if let Some(agent) = agents.pop_front() {
                return Ok(agent);
            }
        }

        // No pooled instance available, create new one
        self.create_fresh_agent(agent_id).await
    }

    pub async fn return_agent(
        &self,
        agent: Agent<Running>
    ) -> Result<(), PoolError> {
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

    #[error(
        "Agent {agent_id} is in state {current_state}, \
         cannot perform {operation}"
    )]
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
            tracing::error!(
                error = %e.into(),
                context = %context,
                "Operation failed"
            );
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

### Zero-Dependency Deployment (Default)

Caxton is designed for immediate deployment with zero external dependencies by
default, providing a complete working system out of the box:

```bash
# Simple deployment - works immediately
caxton server start

# Or with Docker
docker run -p 8080:8080 -v ./agents:/var/lib/caxton/agents caxton/caxton:latest
```

**What happens automatically**:

- SQLite database created in `/var/lib/caxton/memory.db`
- All-MiniLM-L6-v2 embedding model downloaded (~23MB)
- Agent registry initialized
- Memory system ready for use
- Configuration agents can be deployed immediately

### Configuration-First Deployment

```yaml
# docker-compose.yml for configuration-driven agents
version: "3.8"

services:
  caxton-server:
    image: caxton/caxton:latest
    ports:
      - "8080:8080" # REST Management API
      - "9090:9090" # Metrics endpoint
    environment:
      - CAXTON_CONFIG_PATH=/etc/caxton/config.yaml
      - RUST_LOG=info
      # Optional: External observability
      - OTEL_EXPORTER_OTLP_ENDPOINT=http://jaeger:14268
    volumes:
      - ./config/caxton.yaml:/etc/caxton/config.yaml:ro
      - ./agents:/var/lib/caxton/agents:ro # Configuration agents
      - caxton-data:/var/lib/caxton # Embedded SQLite + memory
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s
      timeout: 10s
      retries: 3
    restart: unless-stopped

  # Optional: External observability (not required)
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686" # UI
      - "14268:14268" # HTTP collector
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    profiles: ["observability"]

volumes:
  caxton-data: # Only volume needed - contains SQLite DB and embeddings
```

### Enterprise Deployment with Pluggable External Backends

For larger deployments requiring scale beyond the embedded backend's 100K+
entity capacity, external backends provide seamless migration paths:

```yaml
version: "3.8"

services:
  caxton-server:
    image: caxton/caxton:latest
    ports:
      - "8080:8080"
      - "9090:9090"
    environment:
      - CAXTON_MEMORY_BACKEND=neo4j
      - NEO4J_URI=bolt://neo4j:7687
      - NEO4J_USER=neo4j
      - NEO4J_PASSWORD=password
      # Or: CAXTON_MEMORY_BACKEND=qdrant
      # QDRANT_URL=http://qdrant:6333
    volumes:
      - ./agents:/var/lib/caxton/agents:ro
      - caxton-config:/var/lib/caxton
    depends_on:
      - neo4j

  neo4j:
    image: neo4j:5-community
    environment:
      - NEO4J_AUTH=neo4j/password
      - NEO4J_PLUGINS=["apoc"]
    ports:
      - "7474:7474" # Web interface
      - "7687:7687" # Bolt protocol
    volumes:
      - neo4j-data:/data

volumes:
  caxton-config:
  neo4j-data:
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
        accessModes: ["ReadWriteOnce"]
        resources:
          requests:
            storage: 10Gi
```

## Summary

The Caxton hybrid architecture delivers on the promise of **5-10 minute agent
creation** while maintaining production-grade capabilities:

### Core Value Propositions

1. **Configuration-First Experience**: 90% of agents are created through
   TOML configuration files, eliminating compilation complexity
2. **Zero Dependencies**: Embedded SQLite+Candle memory system works
   immediately without external setup
3. **Hybrid Runtime**: Simple config agents for most use cases, WASM for
   advanced algorithms
4. **Security Through Isolation**: MCP tools run in WASM sandboxes while
   config agents provide rapid development
5. **Capability-Based Messaging**: Lightweight FIPA messaging with capability
   routing instead of agent-specific addressing
6. **Production Ready**: Comprehensive observability, error handling, and
   deployment patterns

### When to Use Each Component

**Choose Simple Agents when**:

- Building orchestration workflows
- Combining prompts with tool calls
- Rapid prototyping and iteration needed
- No custom algorithms required
- 5-10 minute setup time acceptable

**Deploy MCP Servers when**:

- Custom tools or functionality needed
- Performance-critical processing required
- Language-agnostic development desired
- Full isolation and resource limits needed
- Reusable tools across multiple agents

### Architecture Alignment

This architecture aligns with modern agent platform trends:

- **Claude Code's success**: Proven configuration-driven approach with 100+
  community agents
- **Developer Experience**: Prioritizes accessibility over complexity
- **Type-Driven Design**: All illegal states remain unrepresentable through
  Rust's type system
- **Observability First**: Every operation traced, logged, and measured
- **Production Scale**: Supports growth from embedded to external backends

The simple approach enables rapid configuration-based development for agents,
with deployable WASM MCP servers providing powerful tools and functionality.
