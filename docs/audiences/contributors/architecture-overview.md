---
title: "System Architecture Overview"
description: "Comprehensive architecture guide for Caxton contributors"
audience: contributors
categories: [Architecture, Development]
layout: page
---

## Executive Summary

Caxton is a production-ready multi-agent orchestration server that
provides **configuration-driven agents** as the primary user experience,
with optional WebAssembly isolation for advanced use cases. The system
offers capability-based messaging, embedded memory capabilities, and
comprehensive observability.

## Core Architectural Principles

### 1. Configuration-First Design (ADR-0028)

**Primary UX**: 90% of agents are created through YAML+Markdown files
in 5-10 minutes, eliminating compilation complexity.

```yaml
---
name: DataAnalyzer
capabilities: [data-analysis, report-generation]
tools: [http_client, csv_parser, chart_generator]
memory: { enabled: true, scope: workspace }
---
# DataAnalyzer Agent

This agent specializes in data analysis and can fetch data from HTTP
endpoints, parse various formats, and generate visualizations.
```

### 2. Hybrid Runtime Architecture

**Two Agent Types:**

- **Configuration Agents** (90% of use cases): YAML+Markdown with LLM
  orchestration
- **WASM Agents** (10% of use cases): Compiled WebAssembly for custom
  algorithms

### 3. Type-Driven Domain Design

Following Scott Wlaschin's "Make Illegal States Unrepresentable"
philosophy:

```rust
// Agent state machine - impossible to have invalid transitions
pub struct Agent<State> { /* ... */ }
pub struct Unloaded;
pub struct Loaded;
pub struct Running;

impl Agent<Unloaded> {
    pub fn load(self) -> Result<Agent<Loaded>, LoadError>
}

impl Agent<Loaded> {
    pub fn start(self) -> Result<Agent<Running>, StartError>
}

// Only running agents can process messages
impl Agent<Running> {
    pub fn handle_message(
        &self,
        msg: AgentMessage
    ) -> Result<(), ProcessingError>
}
```

### 4. Zero Dependencies by Default (ADR-0030)

**Embedded Backend**: SQLite + Candle provides complete working system
out of the box:

- **Memory System**: Entity-relationship storage with semantic search
- **Embedding Model**: All-MiniLM-L6-v2 (~23MB) for local embeddings
- **Scaling Path**: Optional migration to Neo4j/Qdrant for 100K+ entities

### 5. Security Through Isolation

**Hybrid Security Model:**

- **Configuration Agents**: Run in host runtime (orchestration only)
- **MCP Tools**: Execute in WASM sandboxes (actual system access)
- **WASM Agents**: Full isolation with resource limits

## High-Level System Architecture

```text
┌─────────────────────────────────────────────────────────────┐
│                  Caxton Server Process                      │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │              Management API Layer                       │ │
│  │    • Authentication • Authorization                     │ │
│  │    • Agent Deployment • Configuration                   │ │
│  └─────────────────┬───────────────────────────────────────┘ │
│                    │                                         │
│  ┌─────────────────▼───────────────────────────────────────┐ │
│  │         Hybrid Agent Runtime Environment               │ │
│  │                                                         │ │
│  │ Configuration Agents (Primary UX - 5-10 min setup)     │ │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐       │ │
│  │  │Config A    │  │Config B    │  │Config C    │       │ │
│  │  │(YAML+MD)   │  │(YAML+MD)   │  │(YAML+MD)   │       │ │
│  │  │Host Runtime│  │Host Runtime│  │Host Runtime│       │ │
│  │  └──────┬─────┘  └──────┬─────┘  └──────┬─────┘       │ │
│  │         │               │               │             │ │
│  │ WASM Agents (Advanced Use Cases)                       │ │
│  │  ┌─────────┐            │            ┌─────────┐      │ │
│  │  │Agent X  │            │            │Agent Z  │      │ │
│  │  │Sandbox  │            │            │Sandbox  │      │ │
│  │  └────┬────┘            │            └────┬────┘      │ │
│  │       └─────────────────┼─────────────────┘           │ │
│  │                         │                             │ │
│  │  ┌─────────────────────▼───────────────────────────┐  │ │
│  │  │      Capability-Based Agent Message Router       │  │ │
│  │  │  • Capability Routing • Conversation Mgmt       │  │ │
│  │  │  • Protocol Handling  • Error Recovery          │  │ │
│  │  └─────────────────────────────────────────────────┘  │ │
│  │                                                         │ │
│  │  ┌─────────────────────────────────────────────────┐   │ │
│  │  │           MCP Tool Sandboxes (WASM)            │   │ │
│  │  │  • HTTP Client    • CSV Parser                 │   │ │
│  │  │  • Chart Gen      • File System                │   │ │
│  │  │  • Database       • Custom Tools               │   │ │
│  │  └─────────────────────────────────────────────────┘   │ │
│  └─────────────────────────────────────────────────────────┘ │
│                                                               │
│  ┌─────────────────────────────────────────────────────────┐ │
│  │           Embedded Memory System                       │ │
│  │  • SQLite + Candle (Default)                          │ │
│  │  • Entity-Relationship Storage                        │ │
│  │  • Semantic Search (All-MiniLM-L6-v2)                │ │
│  │  • Optional External Backends                         │ │
│  └─────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Core Components Deep Dive

### Agent Runtime Environment

**Responsibilities:**

- **Lifecycle Management**: Agent deployment, startup, shutdown
- **Resource Allocation**: Memory, CPU, and execution limits
- **Message Routing**: Capability-based routing between agents
- **Security Enforcement**: Sandbox policies and isolation

**Key Implementation:**

```rust
pub struct AgentRuntime {
    config_agents: HashMap<AgentId, Arc<ConfigAgent>>,
    wasm_agents: HashMap<AgentId, Arc<WasmAgent<Running>>>,
    message_router: Arc<CapabilityRouter>,
    resource_manager: Arc<ResourceManager>,
    observability: Arc<ObservabilityLayer>,
}

impl AgentRuntime {
    #[instrument(skip(self))]
    pub async fn spawn_config_agent(
        &mut self,
        config: ConfigAgentDefinition
    ) -> Result<AgentId, SpawnError> {
        // 1. Validate configuration
        let validated_config = self.validate_config_agent(&config).await?;

        // 2. Create agent instance
        let agent = ConfigAgent::from_definition(validated_config)?;

        // 3. Register capabilities with router
        self.message_router
            .register_capabilities(agent.id(), agent.capabilities())
            .await?;

        // 4. Store running agent
        let agent_id = agent.id();
        self.config_agents.insert(agent_id, Arc::new(agent));

        Ok(agent_id)
    }
}
```

### Agent Message Router with Capability-Based Routing

**Core Innovation**: Messages target capabilities, not specific agents.

```rust
#[derive(Debug, Clone)]
pub struct AgentMessage {
    pub id: MessageId,
    pub performative: Performative,
    pub sender: AgentId,
    pub target_capability: Capability,  // Key innovation
    pub conversation_id: ConversationId,
    pub content: MessageContent,
    // Observability context
    pub trace_id: TraceId,
    pub span_id: SpanId,
}

impl CapabilityRouter {
    pub async fn route_by_capability(
        &self,
        message: AgentMessage
    ) -> Result<(), RoutingError> {
        // 1. Find agents providing target capability
        let capable_agents = self.capability_registry
            .find_agents_with_capability(&message.target_capability)
            .await?;

        // 2. Select routing strategy by message type
        let selected_agents = match message.performative {
            Performative::Request => {
                vec![self.select_best_agent(&capable_agents).await?]
            },
            Performative::Inform => capable_agents, // Broadcast
            _ => capable_agents,
        };

        // 3. Route to appropriate runtime
        for agent_id in selected_agents {
            if self.config_agent_runtime.has_agent(agent_id).await? {
                self.route_to_config_agent(message.clone()).await?;
            } else {
                self.route_to_wasm_agent(message.clone()).await?;
            }
        }

        Ok(())
    }
}
```

### Embedded Memory System Architecture

**Default: SQLite + Candle Implementation:**

```rust
pub struct EmbeddedMemorySystem {
    db: Arc<SqlitePool>,
    embeddings: Arc<CandleEmbeddings>,
    entity_store: Arc<EntityStore>,
    relation_store: Arc<RelationStore>,
    semantic_search: Arc<SemanticSearchEngine>,
}

impl EmbeddedMemorySystem {
    pub async fn semantic_search(
        &self,
        query: &str,
        limit: usize,
        min_similarity: f32
    ) -> Result<Vec<EntityMatch>, MemoryError> {
        // Generate query embedding using All-MiniLM-L6-v2
        let query_embedding = self.embeddings.encode_text(query).await?;

        // Perform vector similarity search in SQLite
        let matches = self.semantic_search
            .find_similar(query_embedding, limit, min_similarity)
            .await?;

        Ok(matches)
    }
}
```

### Security Architecture

**Three-Layer Security Model:**

1. **Configuration Agent Security**: Minimal, orchestration-only
2. **MCP Tool Sandboxing**: Primary security boundary
3. **WASM Agent Isolation**: Full sandboxing for custom code

```rust
pub struct McpToolSandbox {
    engine: wasmtime::Engine,
    store: wasmtime::Store<McpContext>,
    resource_limits: ResourceLimits,
    capability_allowlist: CapabilityAllowlist,
}

impl McpToolSandbox {
    pub fn new(
        tool_id: ToolId,
        wasm_bytes: &[u8]
    ) -> Result<Self, SandboxError> {
        // Create engine with strict security config
        let mut config = wasmtime::Config::new();
        config.wasm_simd(false);          // Disable SIMD
        config.wasm_reference_types(false); // Disable ref types
        config.consume_fuel(true);         // Enable CPU limiting

        // Load module with resource limits
        let engine = wasmtime::Engine::new(&config)?;
        let store = wasmtime::Store::new(&engine, context);
        store.limiter(|ctx| &mut ctx.resource_limits);

        Ok(Self { engine, store, /* ... */ })
    }
}
```

## Agent Lifecycle State Machine

```text
Configuration Agents:
   Unloaded → Validated → Registered → Running ← Message Processing
                                         ↓
                                     Shutdown

WASM Agents:
   Unloaded → Loaded → Started → Running ← Message Processing
                                   ↓
                               Draining → Stopped
```

**Implementation with Phantom Types:**

```rust
// State transitions enforced at compile time
impl Agent<Unloaded> {
    pub fn load(self, module: WasmModule) -> Result<Agent<Loaded>, LoadError>
}

impl Agent<Loaded> {
    pub fn start(self) -> Result<Agent<Running>, StartError>
}

impl Agent<Running> {
    pub fn drain(self) -> Result<Agent<Draining>, DrainError>
}

// Only running agents can process messages
impl Agent<Running> {
    pub fn handle_message(
        &self,
        msg: AgentMessage
    ) -> Result<(), ProcessingError>
}
```

## Performance Characteristics

### Configuration Agents

- **Startup Time**: < 50ms (YAML parsing + prompt loading)
- **Memory Overhead**: < 100KB per idle agent
- **Message Latency**: 10-100ms (includes LLM orchestration)
- **Throughput**: 100-1,000 messages/second per agent
- **Concurrent Agents**: 10,000+ per instance

### WASM Agents

- **Startup Time**: < 100ms (WASM instantiation)
- **Memory Overhead**: < 1MB per idle agent
- **Message Latency**: < 1ms p99 for local processing
- **Throughput**: 100,000+ messages/second
- **Concurrent Agents**: 1,000+ per instance

### Embedded Memory System

- **Semantic Search**: 10-50ms for 100K entities
- **Entity Storage**: 5-20ms per entity with embeddings
- **Memory Baseline**: ~200MB (embedding model + cache)
- **Scaling Limit**: 100K+ entities before external backend needed

## Observability Integration

**Comprehensive Instrumentation:**

```rust
#[instrument(skip(self, message))]
pub async fn route_message(
    &self,
    message: AgentMessage
) -> Result<(), RoutingError> {
    // Every operation creates structured logs and metrics
    let span = tracing::info_span!(
        "message_process",
        agent_id = %message.receiver,
        message_id = %message.id,
        performative = ?message.performative,
    );

    // Metrics collection
    self.metrics_registry.record_histogram(
        "caxton_message_processing_duration_seconds",
        duration.as_secs_f64(),
        &[("performative", &format!("{:?}", message.performative))]
    );
}
```

**Observability Stack:**

- **Structured Logging**: tracing crate with JSON output
- **Metrics**: Prometheus-compatible metrics
- **Distributed Tracing**: OpenTelemetry integration
- **Event Store**: Structured event storage for analysis

## Error Handling Architecture

**Railway-Oriented Programming Pattern:**

```rust
// Top-level error type with domain context
#[derive(Debug, Error)]
pub enum CaxtonError {
    #[error("Agent error: {0}")]
    Agent(#[from] AgentError),

    #[error("Message routing error: {0}")]
    Routing(#[from] RoutingError),

    #[error("Deployment error: {0}")]
    Deployment(#[from] DeploymentError),
}

// Domain-specific error types
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent {agent_id} not found")]
    NotFound { agent_id: AgentId },

    #[error("Agent {agent_id} in invalid state {current_state}")]
    InvalidState {
        agent_id: AgentId,
        current_state: String,
    },
}
```

## Deployment Architecture

### Zero-Dependency Deployment (Default)

```bash
# Works immediately - no external dependencies
caxton server start

# Automatic setup:
# - SQLite database in /var/lib/caxton/memory.db
# - All-MiniLM-L6-v2 model downloaded (~23MB)
# - Agent registry initialized
# - Ready for configuration agents
```

### Docker Deployment

```yaml
# docker-compose.yml - Single service needed
version: "3.8"
services:
  caxton-server:
    image: caxton/caxton:latest
    ports:
      - "8080:8080" # REST API
      - "9090:9090" # Metrics
    volumes:
      - ./agents:/var/lib/caxton/agents:ro
      - caxton-data:/var/lib/caxton # Only volume needed
    healthcheck:
      test: ["CMD", "caxton", "health"]
      interval: 30s

volumes:
  caxton-data: # Contains SQLite DB and embeddings
```

### Enterprise Scaling

For deployments requiring > 100K entities:

```yaml
environment:
  - CAXTON_MEMORY_BACKEND=neo4j # or qdrant
  - NEO4J_URI=bolt://neo4j:7687
  # Migration preserves data through JSON export/import
```

## Key Architectural Decisions (ADRs)

### ADR-0028: Configuration-Driven Agents

**Decision**: Make YAML+Markdown agents the primary user experience.

**Rationale**:

- 90% of agent use cases are orchestration, not custom algorithms
- 5-10 minute setup vs 2-4 hours for WASM compilation
- Removes Rust/WASM knowledge barrier

### ADR-0029: Capability-Based Messaging

**Decision**: Route messages by capability, not agent address.

**Rationale**:

- Enables loose coupling and dynamic routing
- Supports horizontal scaling and load balancing
- Simplifies agent discovery and coordination

### ADR-0030: Embedded Memory System

**Decision**: Default to SQLite+Candle, optional external backends.

**Rationale**:

- Zero setup friction for 90% of deployments
- Clear scaling path when needed
- Preserves data portability through standard formats

## Development Patterns

### Adding New Agent Types

1. **Define domain types** that make illegal states unrepresentable
2. **Create state machine** using phantom types
3. **Implement capability registration** with the router
4. **Add observability instrumentation** throughout lifecycle
5. **Write comprehensive tests** covering state transitions

### Message Flow Implementation

1. **Define message types** with clear semantics
2. **Implement routing strategy** for the message type
3. **Add conversation management** if multi-turn
4. **Ensure error handling** with specific error types
5. **Add distributed tracing** for debugging

### Memory Integration

1. **Define entity types** with clear relationships
2. **Implement storage layer** with validation
3. **Add semantic search** if needed for the domain
4. **Provide export/import** for data portability
5. **Add performance tests** for scaling validation

## Contributing Guidelines

### Code Organization

- **Domain logic in `src/domain/`**: Pure business logic
- **Infrastructure in root modules**: Framework and external concerns
- **Integration tests in `tests/`**: End-to-end scenarios
- **Unit tests in source files**: `#[cfg(test)]` modules

### Architecture Alignment

- **Type safety first**: Make invalid states impossible
- **Domain-driven**: Encode business rules in types
- **Observable**: Instrument all operations
- **Testable**: Support comprehensive testing
- **Scalable**: Consider performance implications

This architecture provides a foundation for reliable, observable, and
scalable multi-agent systems while maintaining ease of use through
configuration-driven development.
