---
title: "ADR-0045: Domain Layer Dependency Management"
date: 2025-09-19
status: proposed
layout: adr
categories: [Architecture, Domain Model]
---

## Status

Proposed

## Context

During Phase 6 domain modeling implementation, the team identified a critical architectural decision about how domain layer functions should handle external dependencies. The question arose: Should domain workflow functions perform I/O operations and dependency injection internally, or should they be pure functions that accept all needed data as parameters?

### Current Architectural Layers

The Caxton system follows a layered architecture:

- **Infrastructure Layer**: Handles I/O operations, file loading, external APIs, database connections
- **Application Layer**: Orchestrates workflows by coordinating infrastructure and domain layers
- **Domain Layer**: Contains pure business logic and domain rules

### The Dependency Problem

Domain workflows often need external data to execute business logic. For example, an agent execution workflow might need:

- Agent configuration loaded from TOML files
- Memory system connections for context retrieval
- LLM provider credentials and settings
- Tool server configurations

The architectural question is whether these dependencies should be:

1. **Injected/Loaded Within Domain Functions**: Domain functions handle their own I/O and dependency resolution
2. **Pre-loaded and Passed as Parameters**: Infrastructure/application layers handle loading, domain functions receive data as parameters

### Analysis of Alternative Approaches

#### Option 1: Domain Functions with Dependency Injection

```rust
// Domain function handles its own dependency loading
pub fn execute_agent_request(
    agent_name: AgentName,
    request: AgentRequest,
) -> Result<AgentResponse, ExecutionError> {
    // Would need to load config somehow - violates pure function principle
    let config = load_agent_config(&agent_name)?; // I/O operation in domain
    let memory = connect_to_memory()?; // I/O operation in domain
    // Business logic here
}
```

**Problems Identified**:

- Violates separation of concerns (mixing I/O with business logic)
- Makes functions impure and harder to test
- Creates tight coupling between domain and infrastructure
- Introduces side effects that make reasoning difficult
- Complicates error handling and composability

#### Option 2: Pure Functions with Pre-loaded Dependencies

```rust
// Pure function accepting all needed data
pub fn execute_agent_request(
    config: AgentConfig,      // Pre-loaded by infrastructure/application layer
    memory_context: MemoryContext, // Pre-loaded context
    request: AgentRequest,
) -> Result<AgentResponse, ExecutionError> {
    // Pure business logic only - no I/O operations
}
```

**Advantages Identified**:

- Clear separation of concerns
- Pure functions are easier to test and reason about
- No side effects in domain layer
- Better composability and reusability
- Explicit dependencies make data flow clear

## Decision

We will adopt **pure domain functions that accept all needed data as parameters**. Domain layer functions must not perform I/O operations, dependency injection, or any side effects.

### Core Principle

**Domain workflows shall be pure functions that accept ALL needed data as parameters. No dependency injection, no I/O operations in the domain layer.**

### Architectural Pattern

The system will follow this layered responsibility model:

#### Infrastructure Layer Responsibilities

- File system operations (loading TOML configs, reading templates)
- Database connections and queries
- External API calls (LLM providers, external services)
- Network I/O and resource management
- Error translation from external systems to domain errors

#### Application Layer Responsibilities

- Orchestrating "lookup → load → execute" workflows
- Coordinating between infrastructure and domain layers
- Managing transaction boundaries and consistency
- Converting infrastructure data to domain types
- Handling application-level error recovery

#### Domain Layer Responsibilities

- Pure business logic with pre-loaded inputs
- Domain rule validation and enforcement
- Business process orchestration (without I/O)
- Domain error handling and business exceptions
- Type-safe domain operations

### Implementation Pattern

```rust
// Infrastructure Layer
impl ConfigLoader {
    pub fn load_agent_config(&self, name: &AgentName) -> Result<AgentConfig, LoadError> {
        // File I/O operations here
    }
}

// Application Layer
impl AgentOrchestrator {
    pub fn execute_agent_request(
        &self,
        agent_name: AgentName,
        request: AgentRequest,
    ) -> Result<AgentResponse, OrchestrationError> {
        // 1. Load dependencies
        let config = self.config_loader.load_agent_config(&agent_name)?;
        let memory_context = self.memory_loader.load_context(&request)?;

        // 2. Execute pure domain logic
        let response = domain::execute_agent_request(config, memory_context, request)?;

        // 3. Handle persistence if needed
        self.response_persister.save_response(&response)?;

        Ok(response)
    }
}

// Domain Layer
pub fn execute_agent_request(
    config: AgentConfig,        // Pre-loaded
    memory_context: MemoryContext, // Pre-loaded
    request: AgentRequest,
) -> Result<AgentResponse, ExecutionError> {
    // Pure business logic only - no I/O
}
```

## Decision Drivers

### Testability and Maintainability

**Pure Function Benefits**: Domain functions become trivial to unit test since all inputs are explicit parameters. No mocking of file systems, databases, or external services required for domain logic testing.

**Reasoning Clarity**: Pure functions make data dependencies explicit, improving code comprehension and reducing cognitive load.

### Separation of Concerns

**Single Responsibility**: Each layer has a clear, focused responsibility without overlapping concerns.

**Dependency Isolation**: Changes to infrastructure (file formats, storage systems) don't affect domain logic.

### Composability and Reusability

**Function Composition**: Pure domain functions can be easily composed into larger workflows without side effect concerns.

**Context Independence**: Domain functions work regardless of how dependencies were loaded (files, databases, network, tests).

### Alignment with Domain-Driven Design

**Domain Purity**: Keeps business logic separate from technical infrastructure concerns.

**Explicit Dependencies**: Makes domain requirements clear through function signatures.

## Alternatives Considered

### Dependency Injection Pattern

- **Advantages**: Familiar pattern, reduces parameter passing
- **Rejected**: Introduces impurity and side effects to domain layer, complicates testing, creates implicit dependencies

### Reader Monad Pattern

- **Advantages**: Functional approach to dependency injection, maintains purity
- **Rejected**: Adds complexity without significant benefits over explicit parameters in this context

### Service Locator Pattern

- **Advantages**: Centralized dependency resolution
- **Rejected**: Creates hidden dependencies, violates explicit dependency principle, anti-pattern in modern design

### Hybrid Approach (Some I/O in Domain)

- **Advantages**: Flexibility for different function types
- **Rejected**: Creates inconsistent patterns, makes architectural boundaries unclear

## Consequences

### Positive Outcomes

**Enhanced Testability**: Domain logic can be unit tested with simple parameter passing, no complex mocking required.

**Improved Reasoning**: Pure functions make data flow explicit and eliminate hidden side effects.

**Better Separation of Concerns**: Clear boundaries between I/O operations and business logic.

**Increased Composability**: Pure functions can be easily combined and reused in different contexts.

**Simplified Debugging**: No hidden state changes or side effects within domain functions.

### Implementation Requirements

**Explicit Parameter Passing**: Application layer must load all dependencies before calling domain functions.

**Data Transformation**: Infrastructure layer must convert external data formats to domain types.

**Error Translation**: Infrastructure errors must be translated to domain-appropriate error types.

**Dependency Coordination**: Application layer becomes responsible for orchestrating multi-step workflows.

### Acceptable Trade-offs

**Parameter Proliferation**: Domain functions may have more parameters, but this makes dependencies explicit.

**Application Layer Complexity**: Application layer becomes more complex as it handles orchestration, but this is appropriate separation.

**Potential Performance Overhead**: Pre-loading all dependencies may load more data than needed, but benefits outweigh costs for clarity and testability.

### Risk Mitigation

**Parameter Management**: Use domain types and builder patterns to manage complex parameter sets.

**Performance Monitoring**: Monitor application layer performance to ensure pre-loading doesn't create bottlenecks.

**Documentation Standards**: Clearly document the responsibilities of each layer to prevent architectural drift.

## Implementation Strategy

### Phase 1: Domain Function Design

1. Identify all domain workflow functions that need external data
2. Define explicit parameter types for each dependency
3. Create pure function signatures without I/O operations

### Phase 2: Infrastructure Layer Implementation

1. Implement data loading functions in infrastructure layer
2. Create error translation between infrastructure and domain
3. Ensure infrastructure functions are focused on I/O only

### Phase 3: Application Layer Orchestration

1. Implement application layer coordinators that load dependencies
2. Create workflows that combine infrastructure loading with domain execution
3. Add error handling and transaction management

### Phase 4: Integration and Testing

1. Verify pure domain functions can be unit tested without mocks
2. Test application layer orchestration with real infrastructure
3. Validate clear separation of concerns across all layers

## Alignment with Strategic Goals

**ADR-0018 Domain Types**: Complements strong typing with pure function architecture for complete domain safety.

**ADR-0020 Parse Don't Validate**: Aligns with parsing external data at boundaries and using valid types in domain.

**ADR-0044 On-Demand Agent Execution**: Supports stateless execution model where all dependencies are loaded fresh per request.

**ADR-0028 Configuration-Driven Agents**: Enables clean separation between configuration loading and agent logic execution.

## Measurement Criteria

**Purity Success**: 100% of domain functions are pure (no I/O operations, no side effects).

**Testability Success**: Domain functions can be unit tested without mocking external systems.

**Separation Success**: Clear boundaries maintained between infrastructure, application, and domain layers.

**Maintainability Success**: Changes to data sources don't require domain function modifications.

## References

- [Functional Core, Imperative Shell](https://www.destroyallsoftware.com/screencasts/catalog/functional-core-imperative-shell)
- [Parse Don't Validate](https://lexi-lambda.github.io/blog/2019/11/05/parse-don-t-validate/)
- [Domain-Driven Design Layered Architecture](https://martinfowler.com/bliki/PresentationDomainDataLayering.html)
- [Pure Functions in Domain Logic](https://enterprisecraftsmanship.com/posts/functional-c-primitive-obsession/)
