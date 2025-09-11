---
title: "Testing Strategy and Practices"
description: "Comprehensive guide to testing in Caxton development"
audience: contributors
categories: [Testing, Development, Quality]
layout: page
---

## Testing Philosophy

Caxton follows a comprehensive testing strategy that emphasizes:

- **Type-driven testing**: Domain types eliminate many test failure
  scenarios
- **Test-driven development**: Red→Green→Refactor cycles for all new
  features
- **Property-based testing**: Generate inputs to verify invariants
- **Integration testing**: Test complete workflows with realistic
  scenarios

## Core Testing Tools

### Nextest (Mandatory)

**CRITICAL**: Always use `cargo nextest run` instead of `cargo test`.

```bash
# Install nextest if not available
cargo install cargo-nextest --locked

# Run all tests
cargo nextest run

# Why nextest?
# - Better parallelization and performance
# - Clearer output and error reporting
# - Improved test isolation
# - Better integration with CI/CD
```

### Property-Based Testing (Proptest)

Use proptest to verify invariants across input ranges:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn agent_name_roundtrip_serialization(
        name in "[a-zA-Z0-9_-]{1,64}"
    ) {
        let agent_name = AgentName::try_new(name.clone()).unwrap();
        let serialized = serde_json::to_string(&agent_name).unwrap();
        let deserialized: AgentName = serde_json::from_str(&serialized).unwrap();
        assert_eq!(agent_name, deserialized);
    }
}
```

### WASM Test Fixtures

Integration tests use compiled WASM modules in `tests/fixtures/`:

```rust
#[tokio::test]
async fn test_agent_message_handling() {
    let wasm_bytes = include_bytes!("fixtures/test_agent.wasm");
    let runtime = WasmRuntime::new(test_config()).await?;

    let agent_id = runtime.deploy_agent(
        AgentConfig::new("test-agent", wasm_bytes)
    ).await?;

    let message = AgentMessage::new(
        MessageId::generate(),
        Performative::Request,
        AgentId::system(),
        agent_id,
        ConversationId::generate(),
        MessageContent::text("test message")
    );

    let response = runtime.route_message(message).await?;
    assert!(response.is_ok());
}
```

## Test Structure Overview

```text
Testing Architecture:
├── Unit Tests (99 tests)           # In #[cfg(test)] modules
│   ├── Domain Types (automatic)    # nutype validation tests
│   ├── Business Logic             # Core functionality
│   ├── Resource Management        # Memory, CPU limits
│   └── Security Policies          # Validation rules
├── Integration Tests (47 tests)    # In tests/ directory
│   ├── Runtime Integration        # End-to-end workflows
│   ├── Message Routing           # Agent message handling
│   ├── Performance Benchmarks    # Scaling and timing
│   └── Error Scenarios           # Failure mode testing
└── Property Tests (12 tests)      # Generative testing
    ├── Domain Type Properties     # Invariant verification
    ├── Serialization Roundtrips   # Data consistency
    └── Boundary Validation        # Edge case coverage
```

## Test Categories Deep Dive

### Unit Tests (Fast, Focused)

**Location**: `#[cfg(test)]` modules within source files

**Characteristics**:

- **Execution time**: < 1ms each
- **Isolation**: Test individual functions and types
- **Coverage**: Business logic and domain validation

**Example**: Domain type validation

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_name_validation_accepts_valid_names() {
        let valid_names = vec![
            "web-scraper",
            "data_analyzer",
            "chat-bot-v2",
            "a",  // minimum length
            "a".repeat(64)  // maximum length
        ];

        for name in valid_names {
            assert!(
                AgentName::try_new(name.clone()).is_ok(),
                "Should accept valid name: {}",
                name
            );
        }
    }

    #[test]
    fn agent_name_validation_rejects_invalid_names() {
        let invalid_names = vec![
            "",  // empty string
            " ",  // whitespace only
            "a".repeat(65),  // too long
            "agent with spaces",  // spaces
            "agent@domain",  // special characters
        ];

        for name in invalid_names {
            assert!(
                AgentName::try_new(name.clone()).is_err(),
                "Should reject invalid name: {}",
                name
            );
        }
    }

    #[test]
    fn memory_bytes_arithmetic_operations() {
        let base = MemoryBytes::from_mb(10).unwrap();
        let additional = MemoryBytes::from_mb(5).unwrap();

        let sum = base.saturating_add(additional);
        assert_eq!(sum.as_usize(), 15 * 1024 * 1024);

        let diff = base.saturating_sub(additional);
        assert_eq!(diff.as_usize(), 5 * 1024 * 1024);
    }
}
```

### Integration Tests (Realistic Workflows)

**Location**: `tests/` directory

**Characteristics**:

- **Execution time**: 10ms - 1s each
- **Scope**: Complete workflows and system interactions
- **Environment**: Use WASM fixtures and realistic scenarios

**Example**: Message routing integration

```rust
// tests/message_routing_test.rs
use caxton::{WasmRuntime, AgentMessage, AgentConfig};

#[tokio::test]
async fn test_capability_based_routing() {
    // Setup runtime with test configuration
    let runtime = WasmRuntime::new(test_config()).await?;

    // Deploy agents with different capabilities
    let analyzer_id = runtime.deploy_agent(
        AgentConfig::new("data-analyzer", ANALYZER_WASM)
            .with_capability("data-analysis")
    ).await?;

    let reporter_id = runtime.deploy_agent(
        AgentConfig::new("report-generator", REPORTER_WASM)
            .with_capability("report-generation")
    ).await?;

    // Send message targeting capability, not specific agent
    let message = AgentMessage::new(
        MessageId::generate(),
        Performative::Request,
        AgentId::system(),
        Capability::new("data-analysis"),  // Capability-based routing
        ConversationId::generate(),
        MessageContent::json(json!({
            "task": "analyze_csv",
            "data_url": "https://example.com/data.csv"
        }))
    );

    // Verify message routes to correct agent
    let result = runtime.route_message(message).await?;
    assert!(result.routed_to_agent(analyzer_id));
    assert!(!result.routed_to_agent(reporter_id));
}

#[tokio::test]
async fn test_resource_limit_enforcement() {
    let runtime = WasmRuntime::new(
        test_config().with_memory_limit(MemoryBytes::from_mb(1))
    ).await?;

    let agent_id = runtime.deploy_agent(
        AgentConfig::new("memory-hog", MEMORY_HOG_WASM)
    ).await?;

    // Agent should fail when exceeding memory limit
    let message = FipaMessage::request(
        AgentId::system(),
        agent_id,
        MessageContent::text("allocate_large_buffer")
    );

    let result = runtime.route_message(message).await;
    assert!(matches!(result.unwrap_err(),
        RoutingError::ResourceLimitExceeded { .. }));
}
```

### Property-Based Tests (Invariant Verification)

**Location**: Dedicated test modules or integration tests

**Characteristics**:

- **Input generation**: Proptest generates thousands of test cases
- **Invariant checking**: Verify properties hold across input space
- **Edge case discovery**: Find corner cases manual testing misses

**Example**: Domain type properties

```rust
// tests/property_tests.rs
use proptest::prelude::*;
use caxton::domain_types::*;

proptest! {
    #[test]
    fn cpu_fuel_operations_never_underflow(
        initial in 0u64..1_000_000,
        consumed in 0u64..1_000_000
    ) {
        let fuel = CpuFuel::try_new(initial).unwrap();
        let consumption = CpuFuelAmount::try_new(consumed).unwrap();

        // saturating_subtract should never panic or underflow
        let remaining = fuel.saturating_subtract(consumption);
        assert!(remaining.as_u64() <= initial);

        // subtract should return error for underflow, not panic
        if consumed > initial {
            assert!(fuel.subtract(consumption).is_err());
        } else {
            assert!(fuel.subtract(consumption).is_ok());
        }
    }

    #[test]
    fn agent_id_parsing_roundtrip(
        id in 1u64..u64::MAX
    ) {
        let original_id = AgentId::from_u64(id);
        let id_string = original_id.to_string();
        let parsed_id = AgentId::parse(&id_string).unwrap();
        assert_eq!(original_id, parsed_id);
    }

    #[test]
    fn memory_bytes_conversion_consistency(
        mb in 1usize..1024
    ) {
        let from_mb = MemoryBytes::from_mb(mb).unwrap();
        let from_bytes = MemoryBytes::try_new(mb * 1024 * 1024).unwrap();
        assert_eq!(from_mb, from_bytes);
        assert_eq!(from_mb.as_usize(), mb * 1024 * 1024);
    }
}
```

## Running Tests

### Basic Test Commands

```bash
# Run all tests (mandatory: use nextest)
cargo nextest run

# Run only unit tests
cargo nextest run --lib

# Run only integration tests
cargo nextest run --tests

# Run specific test module
cargo nextest run sandbox::tests
cargo nextest run message_routing_test

# Run single test function
cargo nextest run test_agent_lifecycle_state_machine
```

### Advanced Test Options

```bash
# Show test output (for debugging)
cargo nextest run --nocapture

# Run with backtrace on failure
RUST_BACKTRACE=1 cargo nextest run

# Continue running after first failure
cargo nextest run --no-fail-fast

# Run tests with timing information
cargo nextest run --verbose

# Run only tests matching pattern
cargo nextest run --run-ignored
cargo nextest run --ignored
```

### Performance and Benchmarking

```bash
# Run performance benchmarks (if implemented)
cargo nextest run bench::

# Profile test execution
cargo nextest run --profile ci

# Memory usage testing
RUST_TEST_TIME_INTEGRATION=60 cargo nextest run --tests
```

## Test-Driven Development (TDD)

### Red→Green→Refactor Cycle

**1. Red Phase (Write Failing Test):**

```rust
#[test]
fn agent_should_handle_malformed_messages() {
    let runtime = test_runtime();
    let agent_id = runtime.deploy_simple_agent().await.unwrap();

    // Send malformed message
    let malformed_message = FipaMessage::new(
        MessageId::generate(),
        Performative::Request,
        AgentId::system(),
        agent_id,
        ConversationId::generate(),
        MessageContent::invalid_json("{ invalid json }")
    );

    // Should handle gracefully with appropriate error
    let result = runtime.route_message(malformed_message).await;
    assert!(matches!(result.unwrap_err(),
        RoutingError::InvalidMessageFormat { .. }));
}
```

**2. Green Phase (Make Test Pass):**

```rust
impl Agent<Running> {
    pub fn handle_message(&self, message: AgentMessage) -> Result<(), ProcessingError> {
        // Add validation to handle malformed messages
        if let Err(e) = self.validate_message_format(&message) {
            return Err(ProcessingError::InvalidMessageFormat {
                message_id: message.id,
                reason: e.to_string()
            });
        }

        // Process valid message
        self.process_valid_message(message)
    }
}
```

**3. Refactor Phase (Improve Structure):**

```rust
impl Agent<Running> {
    pub fn handle_message(&self, message: AgentMessage) -> Result<(), ProcessingError> {
        // Extract validation into clear method
        let validated_message = self.validate_and_parse_message(message)?;

        // Extract processing into focused method
        self.execute_message_handler(validated_message)
    }

    fn validate_and_parse_message(
        &self,
        message: AgentMessage
    ) -> Result<ValidatedMessage, ProcessingError> {
        // Centralized validation logic
        ValidatedMessage::try_from(message)
    }
}
```

### Domain-First TDD

When building new features:

1. **Model the domain types** that make illegal states impossible
2. **Write tests for domain invariants** using property-based testing
3. **Implement minimal domain logic** to satisfy type requirements
4. **Add integration tests** for complete workflows
5. **Refactor** to improve clarity and maintainability

## Testing Domain Types

### Automatic nutype Tests

Domain types created with `nutype` include built-in validation tests:

```rust
#[nutype(
    sanitize(trim),
    validate(len(min = 1, max = 64)),
    derive(Clone, Debug, Eq, PartialEq, Display, Serialize, Deserialize)
)]
pub struct AgentName(String);

// nutype automatically generates:
// - Boundary validation tests (empty string, max length)
// - Serialization/deserialization tests
// - Default value tests where applicable
```

### Custom Domain Logic Tests

Add business-specific tests for domain operations:

```rust
#[cfg(test)]
mod agent_name_tests {
    use super::*;

    #[test]
    fn agent_name_normalization() {
        let name = AgentName::try_new("  My-Agent  ".to_string()).unwrap();
        // sanitize(trim) should remove whitespace
        assert_eq!(name.as_str(), "My-Agent");
    }

    #[test]
    fn agent_name_case_sensitivity() {
        let name1 = AgentName::try_new("agent".to_string()).unwrap();
        let name2 = AgentName::try_new("Agent".to_string()).unwrap();
        // Names should be case-sensitive
        assert_ne!(name1, name2);
    }
}
```

## Performance Testing

### Benchmarking with Criterion

```rust
// benches/message_routing_bench.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use caxton::{WasmRuntime, AgentMessage};

fn benchmark_message_routing(c: &mut Criterion) {
    let rt = tokio::runtime::Runtime::new().unwrap();
    let runtime = rt.block_on(async {
        let runtime = WasmRuntime::new(test_config()).await.unwrap();
        runtime.deploy_agent(test_agent_config()).await.unwrap();
        runtime
    });

    c.bench_function("route_simple_message", |b| {
        b.to_async(&rt).iter(|| async {
            let message = create_test_message();
            black_box(runtime.route_message(message).await.unwrap());
        });
    });
}

criterion_group!(benches, benchmark_message_routing);
criterion_main!(benches);
```

### Load Testing Integration Tests

```rust
#[tokio::test]
async fn test_concurrent_message_processing() {
    let runtime = WasmRuntime::new(test_config()).await?;
    let agent_id = runtime.deploy_agent(test_agent_config()).await?;

    // Generate 1000 concurrent messages
    let messages: Vec<_> = (0..1000).map(|i| {
        AgentMessage::request(
            AgentId::system(),
            agent_id,
            MessageContent::text(format!("message_{}", i))
        )
    }).collect();

    // Process all messages concurrently
    let start = std::time::Instant::now();
    let results = futures::future::join_all(
        messages.into_iter().map(|msg| runtime.route_message(msg))
    ).await;
    let duration = start.elapsed();

    // All messages should succeed
    assert!(results.iter().all(|r| r.is_ok()));

    // Performance requirement: < 100ms for 1000 messages
    assert!(duration < std::time::Duration::from_millis(100));
}
```

## Error Testing Patterns

### Testing Domain Validation

```rust
#[test]
fn resource_limits_validation() {
    // Test boundary conditions
    assert!(MemoryBytes::try_new(0).is_ok());  // Minimum
    assert!(MemoryBytes::try_new(1_073_741_824).is_ok());  // 1GB max
    assert!(MemoryBytes::try_new(1_073_741_825).is_err());  // Over limit

    // Test helper methods
    assert!(MemoryBytes::from_mb(1024).is_ok());  // 1GB in MB
    assert!(MemoryBytes::from_mb(1025).is_err());  // Over limit
}
```

### Testing Error Propagation

```rust
#[tokio::test]
async fn test_error_propagation_chain() {
    let runtime = WasmRuntime::new(test_config()).await?;

    // Deploy agent that will fail on message processing
    let agent_id = runtime.deploy_agent(
        AgentConfig::new("failing-agent", FAILING_AGENT_WASM)
    ).await?;

    let message = FipaMessage::request(
        AgentId::system(),
        agent_id,
        MessageContent::text("trigger_failure")
    );

    // Verify error types propagate correctly through the system
    match runtime.route_message(message).await {
        Err(RoutingError::DeliveryFailed { reason, .. }) => {
            assert!(reason.contains("Agent processing failed"));
        }
        other => panic!("Expected DeliveryFailed error, got: {:?}", other),
    }
}
```

## Continuous Testing

### Development Workflow

Use cargo-watch for continuous feedback:

```bash
# Auto-run tests on file changes
cargo watch -x 'nextest run'

# Auto-run specific test module
cargo watch -x 'nextest run sandbox::tests'

# Run tests and linting on changes
cargo watch -x 'nextest run' -x clippy
```

### Test Organization

```rust
// Organize tests by functionality, not by test type
#[cfg(test)]
mod agent_lifecycle_tests {
    use super::*;

    mod deployment_tests {
        #[test] fn test_valid_deployment() { /* ... */ }
        #[test] fn test_invalid_wasm_module() { /* ... */ }
    }

    mod state_transition_tests {
        #[test] fn test_unloaded_to_loaded() { /* ... */ }
        #[test] fn test_invalid_state_transition() { /* ... */ }
    }

    mod message_handling_tests {
        #[test] fn test_running_agent_processes_messages() { /* ... */ }
        #[test] fn test_stopped_agent_rejects_messages() { /* ... */ }
    }
}
```

## Test Data Management

### Test Fixtures

Organize test data for reusability:

```rust
// tests/common/mod.rs
pub mod fixtures {
    pub fn test_agent_config() -> AgentConfig {
        AgentConfig::new("test-agent", include_bytes!("../fixtures/test_agent.wasm"))
            .with_memory_limit(MemoryBytes::from_mb(10).unwrap())
            .with_cpu_fuel(CpuFuel::try_new(100_000).unwrap())
    }

    pub fn simple_request_message(receiver: AgentId) -> AgentMessage {
        FipaMessage::new(
            MessageId::generate(),
            Performative::Request,
            AgentId::system(),
            receiver,
            ConversationId::generate(),
            MessageContent::text("test request")
        )
    }
}
```

### Database Testing

For integration tests requiring persistence:

```rust
#[tokio::test]
async fn test_agent_state_persistence() {
    let temp_dir = tempfile::tempdir()?;
    let config = test_config()
        .with_database_path(temp_dir.path().join("test.db"));

    let runtime = WasmRuntime::new(config).await?;
    let agent_id = runtime.deploy_agent(test_agent_config()).await?;

    // Verify state persists across runtime restarts
    runtime.shutdown().await?;
    let runtime2 = WasmRuntime::new(config).await?;

    assert!(runtime2.get_agent(agent_id).await?.is_some());
}
```

## Test Coverage and Quality

### Coverage Goals

- **Unit test coverage**: > 90% of business logic
- **Integration test coverage**: All major workflows
- **Property test coverage**: All domain type invariants
- **Error path coverage**: All error conditions

### Quality Metrics

```bash
# Generate coverage report (requires tarpaulin)
cargo install cargo-tarpaulin
cargo tarpaulin --out html

# Mutation testing (optional)
cargo install cargo-mutagen
cargo mutagen
```

### Test Quality Indicators

- **Fast feedback**: Unit tests complete in < 100ms total
- **Clear failure messages**: Tests explain what went wrong
- **Independent**: Tests don't depend on execution order
- **Repeatable**: Tests produce consistent results
- **Focused**: Each test verifies one specific behavior

## Debugging Test Failures

### Common Debugging Techniques

```bash
# Run single failing test with output
cargo nextest run test_name --nocapture

# Add timing information
cargo nextest run test_name --verbose

# Run with backtrace
RUST_BACKTRACE=full cargo nextest run test_name

# Debug with logging output
RUST_LOG=debug cargo nextest run test_name
```

### Test-Specific Debugging

```rust
#[tokio::test]
async fn debug_message_routing() {
    // Enable test-specific logging
    let _guard = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_test_writer()
        .try_init();

    let runtime = WasmRuntime::new(test_config()).await?;

    // Add debug assertions
    let agent_id = runtime.deploy_agent(test_agent_config()).await?;
    tracing::debug!(?agent_id, "Agent deployed successfully");

    // Test with detailed error context
    let result = runtime.route_message(test_message(agent_id)).await;
    match result {
        Ok(response) => {
            tracing::debug!(?response, "Message routed successfully");
            assert!(response.is_success());
        }
        Err(e) => {
            tracing::error!(?e, "Message routing failed");
            panic!("Routing failed: {}", e);
        }
    }
}
```

## Best Practices Summary

### Test Organization

1. **Group related tests** in modules by functionality
2. **Use descriptive test names** that explain the scenario
3. **Follow AAA pattern**: Arrange, Act, Assert
4. **Keep tests focused** on single behaviors
5. **Make tests independent** of execution order

### Test Data

1. **Use domain types** in tests, not primitives
2. **Create reusable fixtures** for common test scenarios
3. **Generate test data** with property-based testing
4. **Use temporary directories** for file system tests
5. **Clean up resources** in test teardown

### Performance

1. **Keep unit tests fast** (< 1ms each)
2. **Use `#[ignore]` for slow tests** and run separately
3. **Parallelize integration tests** where possible
4. **Mock external dependencies** in unit tests
5. **Use realistic data sizes** in performance tests

### Error Testing

1. **Test both success and failure paths** for all operations
2. **Verify specific error types** are returned
3. **Test error propagation** through system layers
4. **Check error message quality** for debugging
5. **Test recovery scenarios** where applicable

This comprehensive testing approach ensures Caxton maintains high
quality, reliability, and performance while supporting rapid
development through strong type safety and automated validation.
