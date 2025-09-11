---
title: "For Contributors: Working on Caxton Codebase"
date: 2025-01-15
layout: page
categories: [Audiences, Contributors]
audience: contributors
description: "Contribute to Caxton's Rust codebase with comprehensive development setup,
  architecture deep dives, and contribution guidelines."
---

## Welcome, Contributor

You want to contribute to **Caxton's codebase itself** - the Rust server,
WebAssembly runtime, message router, embedded memory system, or documentation.
This path provides everything you need to understand the architecture, set up
your development environment, and make meaningful contributions.

## What You'll Learn

- ✅ Complete development environment setup with Nix
- ✅ Architecture deep dives with domain modeling principles
- ✅ Type-driven development patterns and testing strategies
- ✅ SPARC workflow for systematic feature development
- ✅ Code quality gates and contribution guidelines
- ✅ Performance optimization and benchmarking
- ✅ Documentation writing and maintenance

## Contributor Journey

### 🏗️ Foundation Setup (45 minutes)

Get your development environment running and understand the codebase.

1. **[Architecture Overview](../../ARCHITECTURE.md)** (15 min)
   - Hybrid agent architecture (config + WASM)
   - Domain modeling philosophy (Scott Wlaschin approach)
   - Core components and their interactions

2. **Development Environment Setup** (30 min)
   - Nix development shell with all tools
   - Bacon continuous testing integration
   - Pre-commit hooks and code quality gates
   - SPARC workflow tooling

### 🧠 Architecture Deep Dive (2 hours)

Master the technical architecture and design principles.

1. **[Domain-Driven Design Patterns](../../docs/domain-types.md)** (30 min)
   - "Make illegal states unrepresentable" philosophy
   - Smart constructors and validation patterns
   - Type-driven API design

2. **[ADR Documentation](../../docs/_adrs/)** (60 min)
   - **[ADR-28: Configuration Agents](../../docs/_adrs/0028-configuration-driven-agent-architecture.md)**
     - Primary UX
   - **[ADR-29: Agent Messaging](../../docs/_adrs/0029-agent-messaging.md)**
     - Capability routing
   - **[ADR-30: Embedded Memory](../../docs/_adrs/0030-embedded-memory-system.md)**
     - Zero-dependency backend

3. **[Security Architecture](../../docs/_adrs/0016-security-architecture.md)**
   (30 min)
   - Hybrid security model: host runtime + WASM sandboxes
   - MCP tool isolation and capability allowlists
   - Resource limits and denial-of-service protection

### 🛠️ Development Workflow (1.5 hours)

Learn the tools, patterns, and processes for contributing code.

1. **[Testing Strategy](../../docs/development/testing-strategy.md)** (30 min)
   - Bacon continuous testing workflow (MANDATORY)
   - Property-based testing with proptest
   - Integration testing patterns

2. **[SPARC Workflow](../../CLAUDE.md#sparc-coordinator-role-critical)**
   (45 min)
   - Research → Plan → Implement → Expert review process
   - Type-driven development with domain modeling
   - TDD discipline: Red → Green → Refactor cycles

3. **[Code Quality Gates](../../CLAUDE.md#code-quality-enforcement---critical)**
   (15 min)
   - Zero tolerance for `allow` attributes without approval
   - Pre-commit hook requirements
   - Clippy rules and formatting standards

### 🚀 Advanced Topics (2+ hours)

Deep technical areas for specialized contributions.

1. **[WebAssembly Runtime](../../docs/wasm-runtime-architecture.md)** (45 min)
   - Wasmtime integration and resource management
   - Host function registry and security boundaries
   - Agent lifecycle and state management

2. **[Message Router Architecture](../../docs/architecture/message-router.md)**
   (45 min)
    - Agent messaging protocol implementation
    - Capability-based routing algorithms
    - Conversation state management

3. **[Memory System Implementation](../../docs/memory-system/embedded-backend.md)**
   (45 min)
    - SQLite + Candle integration
    - Semantic search with All-MiniLM-L6-v2
    - Migration patterns for external backends

4. **[Performance Optimization](../../docs/performance_and_safety_metrics.md)**
   (45 min)
    - Benchmarking methodology
    - Memory usage patterns
    - Async runtime optimization

## Development Environment Setup

### Prerequisites

```bash
# Install Nix (required for development environment)
curl -L https://nixos.org/nix/install | sh

# Clone the repository
git clone https://github.com/caxton/caxton.git
cd caxton

# Enter Nix development shell (provides all tools)
nix develop

# Verify environment
rustc --version    # Latest stable Rust
cargo --version
bacon --version    # Continuous testing
just --version     # Task runner (optional)
```

### Required Development Tools

The Nix shell provides:

- **Rust toolchain**: stable with clippy, rustfmt, rust-analyzer
- **Cargo extensions**: nextest, expand, edit, watch
- **Bacon**: Continuous testing (MANDATORY - see CLAUDE.md)
- **Development tools**: git, curl, jq, sqlite

### MANDATORY: Bacon Integration

**CRITICAL**: All development MUST use bacon for continuous testing.

```bash
# Start bacon in background (required for all work)
bacon --headless &

# Verify bacon is monitoring your changes
echo "Making a test change..."
# Edit any Rust file and watch bacon output

# NEVER use manual test commands
# ❌ Don't: cargo test
# ❌ Don't: cargo nextest run
# ✅ Do: Let bacon handle all testing automatically
```

Why bacon is mandatory:

- **Real-time feedback**: See test results as you type
- **TDD discipline**: Enforces proper Red → Green → Refactor cycles
- **Prevents broken commits**: Catches issues immediately
- **Performance**: Only runs affected tests

## Core Architecture Concepts

### Domain-Driven Type System

Caxton follows Scott Wlaschin's "Domain Modeling Made Functional" approach:

```rust
// Make illegal states unrepresentable
#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 64),
    derive(Clone, Debug, Eq, PartialEq, Display, Hash)
)]
pub struct AgentName(String);

// Smart constructors with validation
impl AgentName {
    pub fn new(name: impl AsRef<str>) -> Result<Self, ValidationError> {
        Self::try_new(name.as_ref())
    }
}

// State machines with phantom types
pub struct Agent<State> {
    id: AgentId,
    name: AgentName,
    _state: PhantomData<State>,
}

// Only loaded agents can be started
impl Agent<Loaded> {
    pub fn start(self) -> Result<Agent<Running>, StartError> {
        // Implementation that makes invalid transitions impossible
    }
}
```

### Type-Driven API Design

```rust
// APIs designed around domain types
pub trait AgentLifecycleManager {
    async fn deploy_agent(
        &self,
        config: AgentConfig,
        strategy: DeploymentStrategy,
    ) -> Result<DeploymentId, DeploymentError>;

    async fn get_agent_status(
        &self,
        id: AgentId,
    ) -> Result<AgentStatus, StatusError>;
}

// Error types that guide handling
#[derive(Debug, Error)]
pub enum DeploymentError {
    #[error("Agent {name} already exists")]
    AgentExists { name: AgentName },

    #[error("Invalid WASM module: {reason}")]
    InvalidWasm { reason: String },

    #[error("Resource limit exceeded: {limit_type}")]
    ResourceLimitExceeded { limit_type: ResourceType },
}
```

### Railway-Oriented Programming

```rust
// Chain operations with comprehensive error handling
pub async fn deploy_configuration_agent(
    agent_file: &Path,
    deployment_config: DeploymentConfig,
) -> CaxtonResult<AgentId> {
    parse_agent_configuration(agent_file)
        .and_then(|config| validate_agent_config(config))
        .and_then(|config| check_capability_conflicts(config))
        .and_then_async(|config| register_agent_capabilities(config))
        .and_then_async(|config| deploy_to_runtime(config, deployment_config))
        .map_err(|e| {
            tracing::error!(
                error = %e,
                agent_file = %agent_file.display(),
                "Configuration agent deployment failed"
            );
            e
        })
}
```

## Development Workflows

### Feature Development with SPARC

The SPARC (Systematic Planning and Review for Code) workflow ensures
high-quality contributions:

```bash
# 1. Start bacon continuous testing (MANDATORY)
bacon --headless &

# 2. Research phase - understand the problem
# Use /sparc/model for pure domain exploration
# Use /sparc for full implementation workflows

# 3. Domain modeling (if needed)
# Create types that make illegal states impossible
# Define workflows as function signatures

# 4. TDD implementation
# Write failing test → Make it pass → Refactor → Repeat

# 5. Expert review
# Code quality, performance, security review
```

### Testing Patterns

#### Property-Based Testing

```rust
use proptest::prelude::*;

#[cfg(test)]
mod tests {
    use super::*;

    proptest! {
        #[test]
        fn agent_name_validation_properties(
            name in "\\PC{1,64}"  // Valid unicode, 1-64 chars
        ) {
            let result = AgentName::new(&name);
            prop_assert!(result.is_ok());
            prop_assert_eq!(result.unwrap().as_str(), name.trim());
        }

        #[test]
        fn agent_name_rejects_invalid_input(
            name in "\\PC{65,1000}|^\\s*$"  // Too long or empty/whitespace
        ) {
            let result = AgentName::new(&name);
            prop_assert!(result.is_err());
        }
    }
}
```

#### Integration Testing

```rust
// tests/integration_tests.rs
use caxton::test_utils::*;

#[tokio::test]
async fn test_agent_deployment_lifecycle() {
    let test_runtime = create_test_runtime().await;

    // Deploy configuration agent
    let agent_config = create_test_config_agent("test-agent");
    let agent_id = test_runtime.deploy_agent(agent_config).await?;

    // Verify agent is running
    let status = test_runtime.get_agent_status(agent_id).await?;
    assert_eq!(status, AgentStatus::Running);

    // Test message handling
    let test_message = create_test_fipa_message(agent_id);
    let response = test_runtime.send_message(test_message).await?;

    assert!(response.is_some());
    assert_eq!(response.unwrap().performative, Performative::Inform);
}
```

### Code Quality Standards

#### Mandatory Requirements

```rust
// ❌ NEVER do this without team approval
#[allow(clippy::unwrap_used)]
pub fn bad_code() {
    let value = risky_operation().unwrap();  // Panics on failure
}

// ✅ Always do this instead
pub fn good_code() -> CaxtonResult<ProcessedValue> {
    let value = risky_operation()
        .map_err(|e| CaxtonError::ProcessingFailed(e))?;

    Ok(ProcessedValue::new(value))
}
```

#### Required Patterns

1. **All functions return `Result` types** (no panics in production)
2. **Instrument async functions** with `#[tracing::instrument]`
3. **Use domain types** instead of primitive obsession
4. **Validate at boundaries** with smart constructors
5. **Comprehensive error types** that guide handling

### Contribution Guidelines

#### Pull Request Requirements

1. **Branch naming**: `story-{id}-{slug}` (e.g., `story-052-wasm-runtime-fix`)
2. **Commit messages**: Conventional commits format
3. **Testing**: All tests pass with bacon
4. **Documentation**: Update docs for new features
5. **ADRs**: Document architectural decisions

#### Code Review Process

1. **Automated checks**: Clippy, formatting, tests must pass
2. **Domain modeling review**: Type safety and illegal state prevention
3. **Performance review**: Benchmarks for performance-critical changes
4. **Security review**: For changes affecting sandboxing or resource limits
5. **Documentation review**: Ensure completeness and accuracy

## Specialized Contribution Areas

### WebAssembly Runtime Development

**Focus**: Agent sandboxing, resource management, host function integration

**Key files**:

- `src/wasm_runtime/` - Core WASM execution
- `src/resource_manager.rs` - CPU/memory limits
- `src/host_functions.rs` - Safe host function registry

**Testing approach**:

- Property-based testing for resource limits
- Integration tests with real WASM modules
- Security testing for sandbox escape attempts

### Memory System Development

**Focus**: Embedded SQLite+Candle backend, semantic search, external backends

**Key files**:

- `src/memory/` - Memory system implementation
- `src/memory/embedded.rs` - SQLite + Candle integration
- `src/memory/migration.rs` - External backend migration

**Testing approach**:

- Performance benchmarks for search latency
- Data integrity tests for SQLite operations
- Migration testing between backends

### Message Router Development

**Focus**: Agent messaging protocol, capability routing, conversation management

**Key files**:

- `src/message_router/` - Core routing logic
- `src/messaging/` - Agent messaging protocol implementation
- `src/capabilities.rs` - Capability-based routing

**Testing approach**:

- Protocol compliance testing
- Load testing for high-throughput scenarios
- Conversation state consistency testing

### Configuration Agent Runtime

**Focus**: YAML parsing, LLM integration, hot-reload

**Key files**:

- `src/config_agent/` - Configuration agent runtime
- `src/config_agent/parser.rs` - YAML schema validation
- `src/config_agent/llm.rs` - LLM provider integration

**Testing approach**:

- Schema validation with invalid YAML
- Hot-reload consistency testing
- LLM integration mocking

## Performance and Benchmarking

### Benchmark Setup

```rust
// benches/memory_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use caxton::memory::EmbeddedMemorySystem;

fn semantic_search_benchmark(c: &mut Criterion) {
    let memory_system = setup_test_memory_system();

    c.bench_function("semantic_search_1000_entities", |b| {
        b.iter(|| {
            memory_system.semantic_search(
                black_box("data analysis trends"),
                black_box(10),
                black_box(0.7),
            )
        })
    });
}

criterion_group!(benches, semantic_search_benchmark);
criterion_main!(benches);
```

### Performance Targets

| Component | Target | Measurement |
|-----------|---------|------------|
| Semantic search | <50ms P99 | 100K entities |
| WASM agent startup | <100ms P99 | Cold start |
| Config agent reload | <10ms P99 | Hot reload |
| Message routing | <1ms P99 | Local delivery |
| Memory store | <20ms P99 | Entity with embeddings |

## Documentation Contributions

### Documentation Standards

1. **Audience-specific**: Target specific user personas
2. **Implementation status**: Clear disclaimers for planned features
3. **Markdownlint compliance**: All markdown must pass linting
4. **YAML frontmatter**: Proper metadata for Jekyll processing

### Documentation Types

- **ADRs**: Architectural decision records with rationale
- **User guides**: Step-by-step instructions for common tasks
- **API reference**: Complete function and type documentation
- **Runbooks**: Operational procedures for production use

## Community and Communication

### Getting Started with Contributions

1. **Browse good first issues**: Labels in GitHub repository
2. **Join Discord**: Real-time discussion with maintainers
3. **Attend office hours**: Weekly contributor Q&A sessions
4. **Read recent PRs**: Understand current development patterns

### Development Coordination

- **Weekly planning**: Roadmap review and story prioritization
- **Architecture discussions**: Major design decisions
- **Performance reviews**: Benchmark results and optimization
- **Security audits**: Regular security and code quality reviews

### Contribution Recognition

- **Contributor wall**: Public recognition for contributions
- **Core contributor status**: Voting rights on architectural decisions
- **Conference speaking**: Opportunities to represent the project
- **Mentorship program**: Help onboard new contributors

---

**Ready to contribute?** Start with the **[Architecture Overview](../../ARCHITECTURE.md)**
to understand the system design, then set up your development environment and
dive into the codebase!
