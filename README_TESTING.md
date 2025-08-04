# Caxton Test Suite Documentation

## 🧪 Comprehensive Testing Strategy

This document describes the complete testing strategy implemented for the Caxton multi-agent orchestration platform, following type-driven development principles and SPARC methodology.

## 📋 Test Coverage Areas

### 1. **Property-Based Tests** (`tests/property/`)
- **Core Agent Types**: Invariants for AgentId, AgentMetadata, AgentType, AgentState
- **FIPA Message Properties**: Conversation consistency, serialization roundtrips
- **State Machine Validation**: Valid state transitions, illegal state prevention
- **Type Safety**: Smart constructors, phantom types, branded types

**Key Files:**
- `tests/property/core/agent_properties.rs` - Agent type invariants with quickcheck

### 2. **Integration Tests** (`tests/integration/`)

#### Agent Coordination (`agent_coordination.rs`)
- Complete agent lifecycle (spawn → ready → process → terminate)
- Multi-agent coordination with message passing
- Fault tolerance and recovery mechanisms
- Resource isolation between agents
- Concurrent agent operations

#### FIPA Messaging (`fipa_messaging.rs`)
- FIPA message structure compliance
- Performative semantics and valid transitions
- Conversation protocols (Contract Net, Query, etc.)
- Ontology and language support
- Message ordering and delivery guarantees
- Error handling and recovery

#### WebAssembly Isolation (`wasm_isolation.rs`)
- Memory isolation between WASM agents
- CPU time limits and enforcement
- System call restrictions and sandboxing
- Resource exhaustion protection
- Concurrent WASM agent isolation
- WASM module validation and security

#### Observability (`observability.rs`)
- Structured logging with correlation IDs
- OpenTelemetry trace propagation
- Metrics collection and aggregation
- Event emission and handling
- Performance monitoring and alerting

#### Performance Benchmarks (`performance_benchmarks.rs`)
- Message throughput and latency testing
- Agent spawning/termination performance
- Memory usage and garbage collection
- Resource contention under load
- Scaling characteristics analysis

### 3. **Test Infrastructure**

#### Test Runner (`tests/test_runner.rs`)
- Orchestrates all test suites
- Provides coverage reporting
- Performance metric collection
- Supports selective test execution
```bash
# Run all tests
cargo run --bin test_runner

# Run specific test types
cargo run --bin test_runner unit
cargo run --bin test_runner integration
cargo run --bin test_runner benchmarks

# Generate coverage report
cargo run --bin test_runner -- --coverage
```

## 🎯 Testing Principles

### Type-Driven Testing
- **Make Illegal States Unrepresentable**: Use phantom types and state machines
- **Parse, Don't Validate**: Smart constructors with Result<T, E> returns
- **Property-Based Testing First**: Verify algebraic laws and invariants
- **Total Functions**: Handle all cases explicitly with pattern matching

### London School TDD
- **Test Behavior, Not Implementation**: Focus on what, not how
- **Mock External Dependencies**: Use testcontainers for isolation
- **Fast Feedback Loops**: Property tests run quickly
- **Behavior Specification**: Tests document expected behavior

### Observable by Design
- **Every Operation Traced**: OpenTelemetry integration from the start
- **Structured Logging**: Consistent correlation IDs across operations  
- **Metrics Collection**: Performance characteristics measured
- **Event Emission**: State changes and lifecycle events captured

## 📊 Coverage Requirements

### Minimum Coverage Targets
- **Core Domain Logic**: 80% minimum coverage
- **Critical Path Operations**: 95% coverage (agent lifecycle, message routing)
- **Error Handling**: 90% coverage (all error paths tested)
- **Integration Points**: 85% coverage (FIPA, WASM, observability)

### Quality Gates
- All property tests must pass (no exceptions)
- Integration tests must pass in parallel
- Performance benchmarks must meet SLA requirements
- No security vulnerabilities in WASM isolation
- Memory leaks detected and prevented

## 🚀 Performance Requirements

### Throughput Benchmarks
- **Message Processing**: >1000 messages/second
- **Agent Spawning**: >10 agents/second  
- **Memory Efficiency**: <2x allocation overhead
- **Scaling**: Linear performance up to 50 agents

### Latency Requirements
- **Message Latency**: P95 < 10ms, P99 < 50ms
- **Agent Ready Time**: <500ms from spawn to ready
- **State Transitions**: <1ms for local state changes
- **Resource Contention**: Fair scheduling, no starvation

## 🔧 Running Tests

### Prerequisites
```bash
# Install required tools
cargo install cargo-nextest    # Parallel test runner
cargo install cargo-tarpaulin  # Coverage reporting
cargo install criterion        # Benchmarking

# Start test dependencies (if needed)
docker-compose up -d
```

### Test Execution
```bash
# Run all tests with nextest (preferred)
cargo nextest run --workspace

# Run tests with standard runner
cargo test --workspace

# Run property tests only
cargo test --lib property

# Run integration tests
cargo test --test integration

# Run performance benchmarks
cargo test benchmark --release

# Generate coverage report
cargo tarpaulin --out Html --output-dir target/coverage
```

### Continuous Integration
```yaml
# Example CI pipeline
test:
  - cargo fmt --check
  - cargo clippy --all-targets -- -D warnings
  - cargo nextest run --workspace
  - cargo test --doc
  - cargo bench --no-run  # Validate benchmarks compile
```

## 🛡️ Security Testing

### WASM Isolation Verification
- Memory boundary enforcement
- System call restrictions
- Resource exhaustion protection
- Malicious module detection
- Cross-agent memory access prevention

### FIPA Protocol Security
- Message validation and sanitization
- Conversation hijacking prevention
- Ontology injection attacks
- Protocol compliance enforcement

## 📈 Monitoring and Metrics

### Test Execution Metrics
- Test duration and performance trends
- Flaky test detection and reporting
- Coverage progression over time
- Performance regression detection

### Production Correlation
- Test scenarios mirror production usage
- Performance benchmarks validate real-world SLAs
- Error injection matches production failure modes
- Observability testing ensures debugging capability

## 🧰 Tools and Frameworks

### Testing Libraries
- **quickcheck**: Property-based testing
- **proptest**: Alternative property testing
- **rstest**: Parameterized tests
- **mockall**: Mocking framework
- **wiremock**: HTTP service mocking
- **testcontainers**: Integration test isolation

### Performance Tools
- **criterion**: Statistical benchmarking
- **cargo-nextest**: Parallel test execution
- **tracing-test**: Observability testing
- **tempfile**: Temporary file management

### Coverage and Quality
- **cargo-tarpaulin**: Code coverage analysis
- **cargo-audit**: Security vulnerability scanning
- **cargo-deny**: Dependency policy enforcement

## 📚 Best Practices

### Test Organization
- Group related tests in modules
- Use descriptive test names that specify behavior
- Separate unit, integration, and performance tests
- Maintain test independence and isolation

### Property Test Design
- Test algebraic laws (associativity, commutativity)
- Verify invariants across all valid inputs
- Use shrinking to find minimal failing cases
- Generate realistic test data distributions

### Integration Test Patterns
- Use testcontainers for external dependencies
- Test complete user journeys end-to-end
- Verify observability and error handling
- Test concurrent operations and race conditions

### Performance Testing
- Establish baseline performance metrics
- Test under realistic load conditions
- Verify resource cleanup and leak prevention
- Monitor performance degradation over time

---

This comprehensive test suite ensures Caxton meets the highest standards for reliability, performance, and security in multi-agent orchestration platforms.