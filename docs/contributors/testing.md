---
title: "Testing Guide"
date: 2025-01-10
layout: page
categories: [Contributors]
---

This document defines the comprehensive testing strategy for Caxton, covering
unit tests, integration tests, system tests, and chaos engineering. The strategy
emphasizes testing distributed systems behaviors, agent interactions, and fault
tolerance.

## Testing Principles

1. **Test at Multiple Levels**: Unit, integration, system, and chaos tests
2. **Focus on Behavior**: Test observable behavior, not implementation details
3. **Deterministic Tests**: Avoid flaky tests through proper synchronization
4. **Fast Feedback**: Quick unit tests, comprehensive integration tests
5. **Production-Like**: Test environments should mirror production

## Testing Levels

### 1. Unit Tests

#### Scope

Individual components and functions in isolation.

#### Framework

```toml
[dev-dependencies]
# Testing framework
tokio-test = "0.4"
mockall = "0.12"  # Mocking framework
proptest = "1.0"  # Property-based testing
criterion = "0.5" # Benchmarking
```

#### Example Unit Test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[tokio::test]
    async fn test_message_routing() {
        // Arrange
        let mut mock_agent = MockAgent::new();
        mock_agent
            .expect_receive()
            .with(eq(test_message()))
            .times(1)
            .returning(|_| Ok(()));

        let router = MessageRouter::new();
        router.register_agent("test-agent", mock_agent);

        // Act
        let result = router.route(test_message()).await;

        // Assert
        assert!(result.is_ok());
        assert_eq!(result.unwrap().status, DeliveryStatus::Delivered);
    }

    #[test]
    fn test_agent_registry_concurrent_access() {
        use proptest::prelude::*;

        proptest!(|(
            agents in prop::collection::vec(any::<AgentId>(), 1..100),
            operations in prop::collection::vec(any::<Operation>(), 1..1000)
        )| {
            let registry = Arc::new(RwLock::new(AgentRegistry::new()));

            // Spawn concurrent operations
            let handles: Vec<_> = operations
                .into_iter()
                .map(|op| {
                    let reg = registry.clone();
                    tokio::spawn(async move {
                        match op {
                            Operation::Insert(agent) => reg.write().await.insert(agent),
                            Operation::Remove(agent) => reg.write().await.remove(agent),
                            Operation::Lookup(agent) => reg.read().await.get(agent),
                        }
                    })
                })
                .collect();

            // All operations should complete without panic
            for handle in handles {
                assert!(handle.await.is_ok());
            }
        });
    }
}
```

### 2. Integration Tests

#### Scope

Multiple components working together, including external dependencies.

#### Test Categories

##### Agent Integration Tests

```rust
#[tokio::test]
async fn test_agent_deployment_lifecycle() {
    // Start test Caxton instance
    let caxton = TestCaxton::start().await;

    // Deploy agent
    let agent_path = test_fixtures::simple_agent_wasm();
    let deployment = caxton
        .deploy_agent(agent_path, DeploymentConfig::default())
        .await
        .expect("deployment failed");

    // Verify agent is running
    assert!(caxton.is_agent_running(&deployment.agent_id).await);

    // Send message to agent
    let response = caxton
        .send_message(AgentMessage::request(
            "system",
            &deployment.agent_id,
            "ping",
        ))
        .await
        .expect("message send failed");

    assert_eq!(response.body, "pong");

    // Remove agent
    caxton.remove_agent(&deployment.agent_id).await.expect("removal failed");

    // Verify agent is gone
    assert!(!caxton.is_agent_running(&deployment.agent_id).await);
}
```

##### Cluster Integration Tests

```rust
#[tokio::test]
async fn test_cluster_formation() {
    // Start seed node
    let node1 = TestNode::start_seed().await;

    // Start additional nodes
    let node2 = TestNode::join(&node1).await;
    let node3 = TestNode::join(&node1).await;

    // Wait for convergence
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Verify all nodes see each other
    assert_eq!(node1.cluster_size().await, 3);
    assert_eq!(node2.cluster_size().await, 3);
    assert_eq!(node3.cluster_size().await, 3);

    // Verify agent registry is synchronized
    node1.deploy_agent("agent-1").await;
    tokio::time::sleep(Duration::from_millis(500)).await;

    assert!(node2.has_agent("agent-1").await);
    assert!(node3.has_agent("agent-1").await);
}
```

### 3. System Tests

#### End-to-End Scenarios

```rust
#[tokio::test]
async fn test_multi_agent_conversation() {
    let cluster = TestCluster::new(3).await;

    // Deploy buyer agent on node 1
    let buyer = cluster.nodes[0]
        .deploy_agent(test_fixtures::buyer_agent())
        .await;

    // Deploy seller agent on node 2
    let seller = cluster.nodes[1]
        .deploy_agent(test_fixtures::seller_agent())
        .await;

    // Deploy mediator agent on node 3
    let mediator = cluster.nodes[2]
        .deploy_agent(test_fixtures::mediator_agent())
        .await;

    // Initiate negotiation
    let negotiation = cluster.nodes[0]
        .send_message(AgentMessage::cfp(
            &buyer.agent_id,
            &seller.agent_id,
            "product-123",
        ))
        .await;

    // Wait for conversation to complete
    let result = cluster
        .wait_for_conversation_completion(&negotiation.conversation_id)
        .await;

    // Verify outcome
    assert!(result.is_agreement_reached());
    assert!(result.price > 0.0);
}
```

### 4. Performance Tests

#### Benchmark Suite

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn routing_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let router = runtime.block_on(setup_router());

    c.bench_function("local routing", |b| {
        b.to_async(&runtime).iter(|| async {
            router.route_local(black_box(test_message())).await
        });
    });

    c.bench_function("remote routing", |b| {
        b.to_async(&runtime).iter(|| async {
            router.route_remote(black_box(test_message())).await
        });
    });
}

fn agent_startup_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let wasm_module = load_test_agent();

    c.bench_function("agent startup", |b| {
        b.to_async(&runtime).iter(|| async {
            let agent = Agent::new(black_box(wasm_module.clone())).await;
            agent.start().await
        });
    });
}

criterion_group!(benches, routing_benchmark, agent_startup_benchmark);
criterion_main!(benches);
```

### 5. Chaos Testing

#### Failure Injection

```rust
pub struct ChaosTest {
    cluster: TestCluster,
    chaos: ChaosMonkey,
}

impl ChaosTest {
    pub async fn test_partition_tolerance() {
        // Deploy agents across cluster
        self.deploy_test_agents().await;

        // Start continuous workload
        let workload = self.start_workload().await;

        // Inject network partition
        self.chaos.partition_network(
            &self.cluster.nodes[0..2],
            &self.cluster.nodes[2..],
        ).await;

        // Verify system continues operating
        assert!(workload.is_still_running().await);

        // Heal partition
        self.chaos.heal_partition().await;

        // Verify recovery
        tokio::time::sleep(Duration::from_secs(5)).await;
        assert!(self.cluster.is_consistent().await);

        // Check for data loss
        assert_eq!(workload.messages_lost(), 0);
    }

    pub async fn test_cascading_failures() {
        // Setup dependency chain
        let agents = self.deploy_dependent_agents().await;

        // Kill critical agent
        self.chaos.kill_agent(&agents[0]).await;

        // Verify circuit breakers activate
        assert!(self.cluster.circuit_breakers_active().await);

        // Verify system doesn't cascade fail
        assert!(self.cluster.healthy_agents() > agents.len() / 2);

        // Restart failed agent
        self.cluster.restart_agent(&agents[0]).await;

        // Verify recovery
        assert!(self.cluster.all_agents_healthy().await);
    }
}
```

## Test Infrastructure

### Test Fixtures

```rust
pub mod test_fixtures {
    pub fn simple_agent_wasm() -> Vec<u8> {
        include_bytes!("../../fixtures/simple_agent.wasm").to_vec()
    }

    pub fn complex_agent_wasm() -> Vec<u8> {
        include_bytes!("../../fixtures/complex_agent.wasm").to_vec()
    }

    pub fn malicious_agent_wasm() -> Vec<u8> {
        // Agent that tries to exceed resource limits
        include_bytes!("../../fixtures/malicious_agent.wasm").to_vec()
    }
}
```

### Test Harness

```rust
pub struct TestHarness {
    docker: Docker,
    network: Network,
    nodes: Vec<Container>,
}

impl TestHarness {
    pub async fn setup() -> Self {
        let docker = Docker::connect().await;
        let network = docker.create_network("caxton-test").await;

        Self {
            docker,
            network,
            nodes: Vec::new(),
        }
    }

    pub async fn start_node(&mut self, config: NodeConfig) -> NodeHandle {
        let container = self.docker
            .run_container("caxton:test", config)
            .with_network(&self.network)
            .await;

        self.nodes.push(container.clone());
        NodeHandle::new(container)
    }

    pub async fn cleanup(self) {
        for node in self.nodes {
            node.stop().await;
            node.remove().await;
        }
        self.network.remove().await;
    }
}
```

## Test Data Management

### Test Data Generation

```rust
use proptest::prelude::*;

prop_compose! {
    fn arb_agent_id()(
        name in "[a-z]{5,10}",
        uuid in any::<u128>()
    ) -> AgentId {
        AgentId::new(format!("{}-{:x}", name, uuid))
    }
}

prop_compose! {
    fn arb_agent_message()(
        performative in prop_oneof![
            Just(Performative::Request),
            Just(Performative::Inform),
            Just(Performative::Query),
        ],
        sender in arb_agent_id(),
        receiver in arb_agent_id(),
        content in any::<String>(),
    ) -> AgentMessage {
        AgentMessage {
            performative,
            sender,
            receiver,
            content,
            ..Default::default()
        }
    }
}
```

## Continuous Integration

### CI Pipeline

```yaml
name: Test Suite

on: [push, pull_request]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run unit tests
        run: cargo test --lib

  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Start test infrastructure
        run: docker-compose -f test/docker-compose.yml up -d
      - name: Run integration tests
        run: cargo test --test integration

  system-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Build test image
        run: docker build -t caxton:test .
      - name: Run system tests
        run: cargo test --test system

  performance-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run benchmarks
        run: cargo bench
      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmarks
          path: target/criterion
```

## Test Coverage

### Coverage Requirements

- Unit tests: 80% line coverage
- Integration tests: 70% branch coverage
- Critical paths: 100% coverage

### Coverage Measurement

```bash
# Install coverage tools
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# Check coverage thresholds
cargo tarpaulin --fail-under 80
```

## Debugging Failed Tests

### Test Debugging Tools

```rust
#[tokio::test]
async fn test_with_debugging() {
    // Enable detailed logging for debugging
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init();

    // Capture test events
    let (tx, rx) = mpsc::channel();
    let recorder = TestRecorder::new(tx);

    // Run test with recording
    let result = run_test_with_recorder(recorder).await;

    // On failure, dump events
    if result.is_err() {
        let events: Vec<_> = rx.try_iter().collect();
        eprintln!("Test failed. Events:");
        for event in events {
            eprintln!("  {:?}", event);
        }
    }

    result.unwrap();
}
```

## Test Maintenance

### Test Organization

```text
tests/
├── unit/
│   ├── agents/
│   ├── routing/
│   └── cluster/
├── integration/
│   ├── agent_lifecycle.rs
│   ├── cluster_formation.rs
│   └── message_routing.rs
├── system/
│   ├── end_to_end.rs
│   └── scenarios/
├── performance/
│   └── benchmarks.rs
└── chaos/
    ├── partition.rs
    └── failures.rs
```

### Test Documentation

Each test should include:

- Purpose description
- Setup requirements
- Expected behavior
- Cleanup needs

## Security Testing

### Security Test Suite

```rust
#[tokio::test]
async fn test_resource_limits_enforced() {
    let caxton = TestCaxton::start().await;

    // Deploy malicious agent that tries to exceed limits
    let agent = caxton
        .deploy_agent(test_fixtures::malicious_agent_wasm())
        .await;

    // Agent should be terminated for exceeding limits
    tokio::time::sleep(Duration::from_secs(2)).await;
    assert!(!caxton.is_agent_running(&agent.agent_id).await);

    // Verify termination reason
    let status = caxton.get_agent_status(&agent.agent_id).await;
    assert_eq!(status.termination_reason, Some("MemoryLimitExceeded"));
}

#[tokio::test]
async fn test_message_authentication() {
    let cluster = TestCluster::new(2).await;

    // Try to send unauthenticated message
    let result = cluster.nodes[0]
        .send_unauthenticated_message(test_message())
        .await;

    assert!(result.is_err());
    assert_eq!(result.unwrap_err().kind(), ErrorKind::Unauthorized);
}
```

## Test Metrics

### Key Metrics to Track

- Test execution time
- Test flakiness rate
- Coverage trends
- Performance regression detection
- Failure patterns

### Test Dashboard

Monitor test health with metrics:

```rust
pub struct TestMetrics {
    test_duration: Histogram,
    test_failures: Counter,
    flaky_tests: Gauge,
    coverage_percentage: Gauge,
}
```

## Best Practices

1. **Isolation**: Tests should not depend on external state
2. **Determinism**: Use fixed seeds for random data
3. **Cleanup**: Always clean up test resources
4. **Naming**: Use descriptive test names
5. **Speed**: Keep unit tests under 1ms
6. **Documentation**: Document complex test scenarios
7. **Maintenance**: Regularly review and update tests
8. **Parallelization**: Design tests to run in parallel

## Future Enhancements

1. **Fuzzing**: Add fuzzing for protocol implementations
2. **Load Testing**: Comprehensive load testing framework
3. **Compliance Testing**: Agent protocol compliance suite
4. **Visual Testing**: UI component testing (if applicable)
5. **Contract Testing**: Agent interaction contract tests

## References

- [Development Guide](development-guide.md)
- [Performance Benchmarks](../operations/performance-tuning.md)
- [Architecture Decision Records](../adrs/)
- [Security Guide](../operations/devops-security-guide.md)
