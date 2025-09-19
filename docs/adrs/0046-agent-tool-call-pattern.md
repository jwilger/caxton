---
title: "ADR-0046: Agent Tool Call Pattern for External System Interaction"
date: 2025-09-19
status: proposed
layout: adr
categories: [Architecture, Agents, External Systems]
---

## Status

Proposed

## Context

During Phase 6 domain modeling implementation, a critical architectural question emerged about how agents should interact with external systems while maintaining the pure domain function architecture established in ADR-0045.

### The Agent-External System Interaction Problem

Configuration-driven agents in Caxton need to interact with two distinct categories of external systems:

1. **Built-in Caxton Platform Capabilities**:
   - Memory system operations (search, create, traverse relationships)
   - Agent lifecycle management
   - Workspace management
   - Internal observability and metrics

2. **User-Deployed Tool Servers**:
   - Custom business logic implementations
   - External API integrations (OpenAI, AWS, databases)
   - Domain-specific computational tools
   - Third-party service connectors

The challenge is enabling agents to request external operations while preserving the pure function architecture from ADR-0045, where domain functions cannot perform I/O operations or have side effects.

### Current Architecture Constraints

**ADR-0045 Pure Domain Functions**: Domain layer functions must be pure - accepting all needed data as parameters with no I/O operations or side effects.

**ADR-0028 Configuration-Driven Agents**: Agents are TOML configurations that define capabilities and behavior, not compiled code.

**ADR-0044 On-Demand Agent Execution**: Agents are stateless configurations loaded on-demand, not persistent processes.

**ADR-0002 WebAssembly Isolation**: MCP tool servers run in WebAssembly sandboxes for security.

### The Continuation Problem

Pure domain functions cannot make external calls, but agents often need external data to continue processing:

- **Memory Operations**: "Search for similar past decisions" → continue with context
- **Tool Calls**: "Calculate risk score via external API" → continue with score
- **State Queries**: "Load current workspace configuration" → continue with settings

The question is: How can pure agent functions request external operations and resume execution with results?

## Decision

We will implement a **continuation-based tool call pattern** where agents can request external operations and resume execution with results, following standard LLM agent tool call conventions while maintaining pure domain function architecture.

### Core Design Pattern

Agent functions return `Result<AgentAction, ExecutionError>` where `AgentAction` enums either complete execution or request external operations with continuation state:

```rust
pub enum AgentAction {
    /// Agent has completed processing
    Complete(AgentResponse),

    /// Request built-in Caxton platform operations
    RequestMemorySearch {
        query: String,
        continuation_state: AgentState
    },
    RequestMemoryCreate {
        entity: String,
        content: String,
        continuation_state: AgentState
    },
    RequestMemoryTraverse {
        from: String,
        relation: String,
        continuation_state: AgentState
    },

    /// Generic tool calls for user-deployed MCP servers
    RequestToolCall {
        tool_name: String,
        parameters: serde_json::Value,
        continuation_state: AgentState,
    },
}
```

### Execution Flow Architecture

The orchestrator manages the agent execution lifecycle through a continuation loop:

1. **Load Configuration**: The orchestrator loads and validates the agent configuration for the specified agent name
2. **Initialize State**: Create initial agent state from the incoming request
3. **Execute with Continuation**: Call the pure domain function with current state
4. **Handle Actions**: Based on returned AgentAction:
   - **Complete**: Return the final response to the user
   - **Request Operations**: Execute the requested external operation (memory, tools)
   - **Resume**: Update state with operation results and continue execution
5. **Repeat**: Continue the loop until completion

The domain function signature supports tool calls through step-based execution:

```rust
pub fn execute_agent_step(
    config: AgentConfig,           // Pre-validated configuration
    state: AgentState,             // Current execution state
) -> Result<AgentAction, ExecutionError>
```

This pattern separates pure business logic (domain) from external system coordination (application layer), while maintaining the current `execute_agent_request` workflow signature that returns final results.

### Agent State Management

Agent state captures all execution context necessary for continuation, including:

- **Original Request**: The user's initial input and context
- **Execution Phase**: Current stage in the agent's processing workflow
- **Accumulated Context**: Results from previous memory operations and tool calls
- **Intermediate Decisions**: Agent's reasoning and decision history
- **Continuation Data**: Serializable state needed to resume processing

State must be completely serializable to support suspension and resumption across process boundaries, enabling natural checkpointing and replay capabilities.

## Decision Drivers

### Alignment with Pure Function Architecture (ADR-0045)

**Maintains Purity**: Domain functions remain pure by accepting all state as parameters and returning action requests rather than performing I/O.

**Explicit Dependencies**: All external data requirements are explicitly represented in the AgentAction return values.

**Testability**: Pure agent functions can be unit tested by providing mock AgentState and verifying returned AgentAction values.

### Standard LLM Agent Pattern Compatibility

**Industry Standard**: Follows established LLM agent tool call patterns used by OpenAI, Anthropic, and agent frameworks.

**Familiar Mental Model**: Developers understand tool call request/response cycles from existing agent experience.

**Tooling Compatibility**: Enables integration with existing agent development tools and observability systems.

### Consistency with Caxton Architecture

**Configuration-Driven**: Agents remain TOML configurations that define available tools and capabilities.

**Stateless Execution**: Each agent execution is independent with all state passed explicitly.

**MCP Integration**: Seamlessly integrates with existing MCP tool server architecture.

**Security Model**: Maintains WebAssembly sandboxing for user tools while allowing direct access to built-in capabilities.

### Observable and Debuggable

**Tool Call Tracing**: Every external operation is explicitly logged as a tool call request/response.

**State Inspection**: Agent state can be inspected at any continuation point for debugging.

**Replay Capability**: Execution can be replayed with different tool responses for testing.

**Performance Monitoring**: Tool call latency and frequency can be monitored per agent type.

## Alternatives Considered

### Dependency Injection with Trait Objects

```rust
pub trait MemorySystem {
    fn search(&self, query: &str) -> Result<Vec<MemoryResult>, MemoryError>;
}

pub fn execute_agent(
    config: AgentConfig,
    memory: &dyn MemorySystem,
    tools: &dyn ToolRegistry,
) -> Result<AgentResponse, ExecutionError> {
    // Would violate pure function principle
}
```

**Rejected**: Violates ADR-0045 pure function architecture by introducing side effects and I/O operations in domain layer.

### Pre-loading All Possible Data

```rust
pub fn execute_agent(
    config: AgentConfig,
    all_memory_data: CompleteMemorySnapshot,
    all_tool_results: PrecomputedToolResults,
) -> Result<AgentResponse, ExecutionError> {
    // Would require impossible data pre-loading
}
```

**Rejected**: Impossible for dynamic memory operations and prohibitively expensive for large datasets. Doesn't support conditional tool calls based on intermediate results.

### Async Domain Functions

```rust
pub async fn execute_agent(
    config: AgentConfig,
    request: AgentRequest,
) -> Result<AgentResponse, ExecutionError> {
    let memory_results = call_memory_search("query").await; // I/O in domain
    // Business logic
}
```

**Rejected**: Introduces I/O operations directly in domain layer, violating separation of concerns and making testing complex.

### State Machine with External Orchestration

```rust
// External orchestrator drives state machine
// Domain functions only handle state transitions
```

**Considered**: Would maintain purity but creates complex orchestration logic and unclear responsibility boundaries.

**Rejected**: More complex than continuation pattern without significant benefits.

## Consequences

### Positive Outcomes

**Pure Domain Functions Preserved**: Agent logic remains testable and side-effect-free while enabling external interactions.

**Standard Tool Call Pattern**: Developers can leverage existing agent development knowledge and tooling.

**Flexible Tool Integration**: Supports both built-in Caxton capabilities and arbitrary user-deployed MCP tools.

**Natural Checkpointing**: Agent state at each continuation point provides natural resume/retry capabilities.

**Observable Execution**: Every external interaction is explicitly visible in the AgentAction flow.

**Composable Operations**: Complex agent workflows can be built by composing tool call sequences.

### Implementation Requirements

**State Serialization**: AgentState must be completely serializable to support continuation across process boundaries.

**Tool Call Infrastructure**: Application layer must implement tool call routing to appropriate systems (memory, MCP servers).

**Error Recovery**: Tool call failures must be handled gracefully with appropriate fallback strategies.

**Performance Optimization**: Orchestrator must minimize overhead for simple agents that don't need tool calls.

### Acceptable Trade-offs

**Execution Complexity**: Multi-step agent execution requires orchestrator loop management vs simple function calls.

**State Management Overhead**: Agent state must capture all necessary continuation context, increasing memory usage.

**Latency Introduction**: Tool calls introduce network latency vs pre-loaded data approaches.

**Mental Model Complexity**: Developers must understand continuation-based execution vs traditional procedural flows.

### Risk Mitigation

**State Size Limits**: Implement maximum AgentState size limits to prevent memory exhaustion.

**Tool Call Timeouts**: Apply timeouts to all external tool calls to prevent hanging executions.

**Cycle Detection**: Detect and prevent infinite tool call loops in agent configurations.

**Tool Availability**: Graceful degradation when requested tools are unavailable.

## Implementation Strategy

### Phase 1: Core Agent Action Types

1. Define `AgentAction` enum with built-in platform operations
2. Implement `AgentState` serialization and continuation support
3. Create agent orchestrator loop for handling tool calls
4. Build memory system integration for built-in operations

### Phase 2: MCP Tool Integration

1. Extend `AgentAction` with generic tool call support
2. Implement MCP client integration in application layer
3. Add tool call routing and error handling
4. Create tool availability validation and fallback logic

### Phase 3: Agent Configuration Enhancement

1. Update TOML agent configurations to declare tool dependencies
2. Implement tool permission and access control systems
3. Add tool call observability and monitoring
4. Create agent development tooling for tool call debugging

### Phase 4: Advanced Features

1. Implement parallel tool call execution for independent operations
2. Add tool call result caching and optimization
3. Create tool call replay and testing infrastructure
4. Build agent workflow composition tools

## Alignment with Strategic Goals

**ADR-0045 Pure Domain Functions**: Maintains pure function architecture while enabling external system interaction.

**ADR-0028 Configuration-Driven Agents**: Supports TOML-based agent definition with tool call specifications.

**ADR-0044 On-Demand Agent Execution**: Compatible with stateless agent loading and execution model.

**ADR-0002 WebAssembly Isolation**: Preserves security model for user-deployed tool servers.

**ADR-0030 Embedded Memory System**: Provides efficient access to built-in Caxton memory operations.

## Measurement Criteria

**Purity Compliance**: 100% of agent domain functions remain pure with no I/O operations.

**Tool Call Success Rate**: >99% of valid tool calls complete successfully or fail gracefully.

**Performance Overhead**: Tool call orchestration adds <10ms overhead vs direct function calls.

**Developer Experience**: Agent development time reduced vs traditional dependency injection patterns.

**Observability Coverage**: 100% of external interactions captured in tool call tracing.

## Future Considerations

**Parallel Tool Execution**: Support for concurrent tool calls when operations are independent.

**Tool Call Batching**: Optimize multiple similar tool calls into batch operations.

**Smart Caching**: Cache tool call results based on agent configuration and input patterns.

**Tool Call Composition**: Enable agents to compose complex workflows from simpler tool call primitives.

## References

- [ADR-0045: Domain Layer Dependency Management](./0045-domain-layer-dependency-management.md)
- [OpenAI Function Calling Documentation](https://platform.openai.com/docs/guides/function-calling)
- [LangChain Agent Tool Pattern](https://python.langchain.com/docs/modules/agents/tools/)
- [Functional Core, Imperative Shell Pattern](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)
