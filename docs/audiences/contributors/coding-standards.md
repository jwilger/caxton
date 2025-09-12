---
title: "Coding Standards and Guidelines"
description: "Code quality standards and contribution guidelines"
audience: contributors
categories: [Development, Standards, Quality]
layout: page
---

## Code Quality Philosophy

Caxton prioritizes code quality through:

- **Type Safety First**: Use Rust's type system to prevent bugs
- **Zero Tolerance for Technical Debt**: Fix issues rather than suppress them
- **Comprehensive Testing**: Every feature includes tests
- **Clear Documentation**: Code should be self-documenting through
  types and names
- **Performance Consciousness**: Optimize thoughtfully with measurement

## Critical Code Quality Rules

### 1. Never Add Allow Attributes (ZERO TOLERANCE)

**This is a hard rule with zero exceptions without explicit team approval.**

```rust
// ❌ NEVER do this without explicit team approval
#![allow(clippy::some_warning)]
#[allow(clippy::some_warning)]
```

**Why this rule exists:**

- Suppressing warnings hides potential issues
- Technical debt accumulates quickly
- Warning suppressions often outlive their usefulness
- Quality standards must be maintained consistently

**What to do instead:**

1. **Fix the underlying issue** causing the warning
2. **Refactor code** to eliminate the warning condition
3. **Create a GitHub issue** for systematic cleanup if many warnings exist
4. **Get explicit team approval** if suppression is truly necessary

### 2. Pre-commit Hooks are Mandatory

Pre-commit hooks ensure code quality and consistency:

```bash
# ❌ Only bypass in genuine emergencies with team notification
git commit --no-verify

# ✅ Normal workflow - let hooks check your code
git commit -m "Your commit message"
```

**Pre-commit hook checks:**

- `cargo fmt` - Code formatting
- `cargo clippy` - Linting with strict rules
- Markdown linting for documentation
- Import organization and unused import removal
- Line ending consistency

**If pre-commit hooks fail:**

1. **Fix the reported issues** instead of bypassing
2. **Use automatic fixes** where available: `cargo fmt`, `cargo clippy --fix`
3. **Address linting violations** systematically
4. **Only bypass in genuine emergencies** with immediate team notification

## Code Formatting Standards

### Automatic Formatting with rustfmt

All code must be formatted with `cargo fmt`:

```bash
# Format your code before committing
cargo fmt

# Check if code is properly formatted
cargo fmt --check
```

**Key formatting rules:**

- **Line length**: 100 characters maximum
- **Indentation**: 4 spaces (no tabs)
- **Import organization**: Automatic grouping and sorting
- **Trailing commas**: Used in multi-line constructs
- **Brace style**: Rust standard (opening brace on same line)

### Documentation Formatting

```rust
/// Brief description of the function.
///
/// Longer description explaining the purpose, behavior, and any important
/// details about the function.
///
/// # Arguments
///
/// * `agent_id` - Unique identifier for the agent
/// * `message` - Agent message to process
///
/// # Returns
///
/// Returns `Ok(())` on successful processing, or `RoutingError` if message
/// routing fails.
///
/// # Errors
///
/// This function returns an error if:
/// - The agent is not found
/// - The message format is invalid
/// - Resource limits are exceeded
///
/// # Examples
///
/// ```rust
/// let agent_id = AgentId::generate();
/// let message = AgentMessage::new(/* ... */);
/// runtime.route_message(agent_id, message)?;
/// ```
pub async fn route_message(
    &self,
    agent_id: AgentId,
    message: AgentMessage
) -> Result<(), RoutingError> {
    // Implementation
}
```

## Linting Standards with Clippy

### Mandatory Clippy Compliance

All clippy warnings must be resolved:

```bash
# Run clippy with project's strict configuration
cargo clippy

# Auto-fix issues where possible
cargo clippy --fix

# Check specific linting categories
cargo clippy -- -W clippy::all -W clippy::pedantic
```

### Custom Clippy Configuration

The project uses a `.clippy.toml` file placed in the repository root
(alongside `Cargo.toml`) to configure clippy rules. This file integrates
automatically with the development setup when running `cargo clippy` or using
the bacon continuous testing system.

Project `.clippy.toml` includes strict rules:

```toml
# Enable additional strictness
cognitive-complexity-threshold = 10
too-many-arguments-threshold = 5
type-complexity-threshold = 50

# Documentation requirements
missing-docs-in-private-items = true
```

### Common Clippy Issues and Solutions

### Issue: Complex function signatures

```rust
// ❌ Too many parameters
fn complex_function(
    a: String,
    b: usize,
    c: bool,
    d: Vec<u8>,
    e: HashMap<String, String>
) -> Result<(), Error>

// ✅ Use configuration struct
struct ComplexConfig {
    name: String,
    size: usize,
    enabled: bool,
    data: Vec<u8>,
    metadata: HashMap<String, String>,
}

fn complex_function(config: ComplexConfig) -> Result<(), Error>
```

### Issue: Unnecessary cloning

```rust
// ❌ Unnecessary clone
fn process_name(name: String) -> String {
    format!("Agent: {}", name.clone())  // name consumed anyway
}

// ✅ Use owned value directly
fn process_name(name: String) -> String {
    format!("Agent: {}", name)
}
```

### Issue: Pattern matching improvements

```rust
// ❌ Verbose pattern matching
match result {
    Ok(value) => Some(value),
    Err(_) => None,
}

// ✅ Use Result::ok()
result.ok()
```

## Error Handling Standards

### Use Domain-Specific Error Types

```rust
// ❌ Generic error handling
fn process_agent(id: u64) -> Result<(), Box<dyn std::error::Error>> {
    // Hard to understand what can go wrong
}

// ✅ Domain-specific error types
fn process_agent(id: AgentId) -> Result<(), AgentError> {
    // Clear error contract
}
```

### Comprehensive Error Context

```rust
#[derive(Debug, Error)]
pub enum AgentError {
    #[error("Agent {agent_id} not found")]
    NotFound { agent_id: AgentId },

    #[error(
        "Agent {agent_id} in invalid state {current_state} \
         for operation {operation}"
    )]
    InvalidState {
        agent_id: AgentId,
        current_state: String,
        operation: String,
    },

    #[error(
        "Resource limit exceeded for agent {agent_id}: \
         {resource_type} ({available}/{requested})"
    )]
    ResourceLimitExceeded {
        agent_id: AgentId,
        resource_type: String,
        available: String,
        requested: String,
    },
}
```

### Error Propagation Patterns

```rust
// Use ? operator for error propagation
pub async fn deploy_agent(
    &self,
    config: AgentConfig
) -> Result<AgentId, DeploymentError> {
    let validated_config = self.validate_config(config)?;
    let wasm_module = self.load_wasm_module(&validated_config.wasm_bytes)?;
    let agent = Agent::new(validated_config.name)
        .load(wasm_module)?
        .start()?;

    let agent_id = agent.id();
    self.agents.insert(agent_id, agent);

    Ok(agent_id)
}

// Add context where helpful
impl CaxtonResultExt<T> for Result<T, E>
where E: Into<CaxtonError>
{
    fn with_agent_context(self, agent_id: AgentId) -> CaxtonResult<T> {
        self.map_err(|e| {
            let error = e.into();
            tracing::error!(%agent_id, %error, "Operation failed for agent");
            error
        })
    }
}
```

## Testing Standards

### Test Organization and Naming

```rust
#[cfg(test)]
mod agent_lifecycle_tests {
    use super::*;

    mod deployment_tests {
        use super::*;

        #[test]
        fn deploys_valid_agent_successfully() {
            // Test valid deployment path
        }

        #[test]
        fn rejects_invalid_wasm_module() {
            // Test validation failure
        }
    }

    mod state_transition_tests {
        use super::*;

        #[test]
        fn transitions_from_unloaded_to_loaded_with_valid_module() {
            // Test valid state transition
        }

        #[test]
        fn prevents_invalid_state_transitions() {
            // Test compile-time prevention
        }
    }
}
```

### Test Quality Standards

**Test Naming Convention:**

- Use descriptive names that explain the scenario
- Format: `operation_condition_expected_outcome`
- Examples: `deploys_agent_successfully_with_valid_config`

**Test Structure (AAA Pattern):**

```rust
#[test]
fn calculates_fuel_remaining_after_consumption() {
    // Arrange
    let initial_fuel = CpuFuel::try_new(1000).unwrap();
    let consumption = CpuFuelAmount::try_new(300).unwrap();

    // Act
    let remaining = initial_fuel.subtract(consumption).unwrap();

    // Assert
    assert_eq!(remaining.as_u64(), 700);
}
```

**Property-Based Testing:**

```rust
proptest! {
    #[test]
    fn fuel_operations_never_underflow(
        initial in 0u64..1_000_000,
        consumed in 0u64..1_000_000
    ) {
        let fuel = CpuFuel::try_new(initial).unwrap();
        let consumption = CpuFuelAmount::try_new(consumed).unwrap();

        let remaining = fuel.saturating_subtract(consumption);
        assert!(remaining.as_u64() <= initial);
    }
}
```

### Mandatory Testing Requirements

- **Unit tests**: All business logic must have unit tests
- **Integration tests**: All major workflows must be tested end-to-end
- **Property tests**: All domain types must have property-based tests
- **Error path testing**: All error conditions must be tested
- **Performance tests**: Critical paths must have performance tests

## Performance Standards

### Measurement Before Optimization

```rust
// Use criterion for benchmarking
#[bench]
fn bench_message_routing(b: &mut Bencher) {
    let runtime = create_test_runtime();
    let message = create_test_message();

    b.iter(|| {
        black_box(runtime.route_message(message.clone()))
    });
}
```

### Performance Guidelines

**Resource Allocation:**

- **Minimize allocations** in hot paths
- **Use object pools** for frequently created/destroyed objects
- **Prefer `&str` over `String`** when ownership not needed
- **Use `Cow<str>`** when sometimes owned, sometimes borrowed

**Async Patterns:**

- **Avoid blocking** in async contexts
- **Use `tokio::spawn`** for CPU-intensive work
- **Batch operations** where possible
- **Use channels** for producer-consumer patterns

```rust
// ❌ Blocking in async context
pub async fn process_data(data: &[u8]) -> Result<ProcessedData, Error> {
    // This blocks the async runtime
    expensive_cpu_operation(data)
}

// ✅ Spawn blocking work
pub async fn process_data(data: &[u8]) -> Result<ProcessedData, Error> {
    let data = data.to_owned();
    tokio::task::spawn_blocking(move || {
        expensive_cpu_operation(&data)
    }).await?
}
```

## Security Standards

### Input Validation

All external inputs must be validated using domain types:

```rust
// ❌ Direct use of untrusted input
pub fn create_agent(
    raw_name: String,
    raw_memory: usize
) -> Result<Agent, Error> {
    // Raw inputs can contain invalid data
    if raw_name.is_empty() || raw_memory > MAX_MEMORY {
        return Err(Error::InvalidInput);
    }
    // Validation logic scattered everywhere
}

// ✅ Parse at boundaries
pub fn create_agent(
    raw_name: String,
    raw_memory: usize
) -> Result<Agent, Error> {
    let name = AgentName::try_new(raw_name)?;  // Validation in one place
    let memory = MemoryBytes::try_new(raw_memory)?;  // Type guarantees validity
    Agent::new(name, memory)  // Safe to use validated types
}
```

### Safe WebAssembly Execution

```rust
pub struct WasmSandbox {
    engine: wasmtime::Engine,
    store: wasmtime::Store<WasmContext>,
    resource_limiter: ResourceLimiter,
}

impl WasmSandbox {
    pub fn new(
        wasm_bytes: &[u8],
        limits: ResourceLimits
    ) -> Result<Self, SandboxError> {
        // Security-hardened WASM configuration
        let mut config = wasmtime::Config::new();
        config.wasm_simd(false);           // Disable SIMD for security
        config.wasm_reference_types(false); // Disable ref types
        config.wasm_bulk_memory(false);    // Disable bulk memory operations
        config.consume_fuel(true);         // Enable CPU limiting

        let engine = wasmtime::Engine::new(&config)?;
        // ... rest of secure initialization
    }
}
```

### Audit Trail and Logging

All security-relevant operations must be logged:

```rust
#[instrument(skip(self, wasm_bytes))]
pub async fn deploy_agent(
    &self,
    name: AgentName,
    wasm_bytes: &[u8]
) -> Result<AgentId, DeploymentError> {
    // Structured logging for audit trail
    tracing::info!(
        agent_name = %name,
        wasm_size = wasm_bytes.len(),
        "Beginning agent deployment"
    );

    // Security validation
    self.validate_wasm_module(wasm_bytes)?;

    tracing::info!(
        agent_name = %name,
        "WASM module validation passed"
    );

    // ... rest of deployment
}
```

## Documentation Standards

### Code Documentation

**Public APIs require comprehensive documentation:**

```rust
/// Routes an agent message to the appropriate agent based on capability.
///
/// This function implements capability-based routing where messages target
/// specific capabilities rather than agent addresses, enabling loose coupling
/// and dynamic load balancing.
///
/// # Arguments
///
/// * `message` - The agent message to route, must have a valid
///   target capability
///
/// # Returns
///
/// Returns `Ok(())` if message was successfully routed and delivered to at
/// least one capable agent. Returns `RoutingError` if no agents provide the
/// requested capability or if message delivery fails.
///
/// # Errors
///
/// * `RoutingError::NoCapabilityProviders` - No agents registered
///   for the capability
/// * `RoutingError::DeliveryFailed` - Message delivery to agent failed
/// * `RoutingError::InvalidMessageFormat` - Message format validation failed
///
/// # Examples
///
/// ```rust
/// let router = CapabilityRouter::new();
/// let message = AgentMessage::request(
///     AgentId::system(),
///     Capability::new("data-analysis"),
///     MessageContent::json(analysis_request)
/// );
///
/// router.route_message(message).await?;
/// ```
pub async fn route_message(
    &self,
    message: AgentMessage
) -> Result<(), RoutingError> {
    // Implementation...
}
```

### Architecture Documentation

All architectural decisions must be documented in ADRs:

```markdown
# ADR-NNNN: Title of Decision

**Date:** YYYY-MM-DD **Status:** [Proposed|Accepted|Deprecated|Superseded]

## Context

Describe the forces at play and the environment in which the decision
is being made.

## Decision

State the decision clearly and concisely.

## Rationale

Explain why this decision was made, including alternatives considered.

## Consequences

Describe the positive and negative consequences of this decision.

## Implementation Notes

Provide specific guidance for implementing this decision.
```

## Contribution Workflow

### Branch Management

```bash
# Create feature branch from main
git checkout main
git pull origin main
git checkout -b feature/agent-lifecycle-improvements

# Regular commits with clear messages
git commit -m "feat: add domain type for agent state transitions"
git commit -m "test: add property tests for state machine invariants"
git commit -m "docs: update architecture overview with state management"

# Push feature branch
git push origin feature/agent-lifecycle-improvements
```

### Commit Message Standards

Use conventional commits format:

```text
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

**Types:**

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `test`: Adding or fixing tests
- `refactor`: Code refactoring without feature changes
- `perf`: Performance improvements
- `chore`: Build process or auxiliary tool changes

**Examples:**

```text
feat(router): implement capability-based message routing

Add support for routing messages based on agent capabilities rather
than specific agent addresses. This enables loose coupling and dynamic
load balancing.

Closes #123

test: add property tests for domain type validation

Add comprehensive proptest coverage for all nutype domain types to
verify invariants hold across input ranges.

docs(adr): add ADR-0029 for agent messaging architecture

Document the decision to use a simplified agent messaging system
optimized for configuration-driven agents.
```

### Pull Request Standards

**PR Title Format:**

```text
feat: brief description of changes
```

**PR Description Template:**

```markdown
## Summary

Brief description of what this PR accomplishes.

## Changes

- List of specific changes made
- Include new files created
- Note any breaking changes

## Testing

- Describe testing approach
- Note any new tests added
- Confirm all tests pass

## Documentation

- List documentation updates
- Note any ADRs created or updated
- Confirm examples work correctly

## Checklist

- [ ] All tests pass locally
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] Breaking changes documented
```

## Code Review Standards

### Review Checklist

**Functionality:**

- [ ] Code accomplishes stated requirements
- [ ] Edge cases are handled appropriately
- [ ] Error handling is comprehensive
- [ ] Performance implications considered

**Code Quality:**

- [ ] Follows project coding standards
- [ ] Uses appropriate domain types
- [ ] No unnecessary complexity
- [ ] Clear naming and structure

**Testing:**

- [ ] Adequate test coverage
- [ ] Tests are clear and focused
- [ ] Property tests for domain types
- [ ] Error paths tested

**Documentation:**

- [ ] Public APIs documented
- [ ] Complex logic explained
- [ ] Examples provided where helpful
- [ ] ADRs updated if needed

### Review Process

1. **Self-review first**: Review your own code before requesting review
2. **Small PRs**: Keep pull requests focused and reviewable
3. **Clear descriptions**: Explain what and why, not just what
4. **Address feedback**: Respond to all review comments
5. **Test locally**: Ensure all tests pass before requesting review

## Maintaining Code Quality

### Regular Maintenance

**Weekly:**

- Run full test suite: `cargo nextest run`
- Check for new clippy warnings: `cargo clippy`
- Update dependencies: `cargo upgrade`

**Monthly:**

- Review and update documentation
- Analyze test coverage reports
- Performance profiling of critical paths

**Before Releases:**

- Comprehensive integration testing
- Security audit of changes
- Documentation completeness review

### Technical Debt Management

**Identify Technical Debt:**

- Clippy warnings that accumulate
- TODO comments in code
- Test coverage gaps
- Performance bottlenecks

**Address Systematically:**

1. **Create GitHub issues** for technical debt items
2. **Prioritize by impact** on development velocity
3. **Allocate time** in sprint planning for debt reduction
4. **Track progress** and celebrate improvements

### Quality Metrics

**Code Quality Indicators:**

- Zero clippy warnings (mandatory)
- Test coverage > 90% (unit tests)
- Build time < 2 minutes (incremental)
- Documentation coverage for public APIs

**Process Quality Indicators:**

- Pre-commit hook success rate > 95%
- Code review turnaround < 24 hours
- CI/CD pipeline success rate > 98%
- Zero security vulnerabilities in dependencies

These standards ensure Caxton maintains high code quality, security,
and maintainability while supporting rapid development through strong
type safety and comprehensive testing.
