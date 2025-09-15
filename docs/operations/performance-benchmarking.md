---
title: "Performance Benchmarking Suite"
date: 2025-01-14
layout: page
categories: [Benchmarks, Performance]
---

## Overview

This guide establishes Caxton's performance benchmarking framework, including
baseline metrics, continuous performance testing, and regression detection
strategies.

## Performance Targets

### Core Metrics

| Metric | Target | Acceptable | Critical |
|--------|--------|------------|----------| | Message Throughput | 100K msg/sec
| 50K msg/sec | 10K msg/sec | | Message Latency (p50) | < 1ms | < 5ms | < 10ms |
| Message Latency (p99) | < 10ms | < 50ms | < 100ms | | Agent Spawn Time | \<
100ms | < 500ms | < 1s | | Memory per Agent | < 10MB | < 50MB | < 100MB | | CPU
per Agent | < 5% | < 10% | < 25% | | Recovery Time | < 30s | < 2min | < 5min |

## Benchmark Categories

### 1. Throughput Benchmarks

#### Message Processing Throughput

```rust
#[bench]
fn bench_message_throughput(b: &mut Bencher) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let orchestrator = create_test_orchestrator();
    let messages = generate_messages(10000);

    b.iter(|| {
        runtime.block_on(async {
            for msg in &messages {
                orchestrator.route_message(msg.clone()).await.unwrap();
            }
        })
    });

    let throughput = (10000.0 * 1_000_000_000.0) / b.ns_per_iter() as f64;
    println!("Throughput: {:.0} messages/sec", throughput);
}
```

#### Agent Task Processing

```rust
#[bench]
fn bench_task_processing(b: &mut Bencher) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let agent_pool = create_agent_pool(10);
    let tasks = generate_tasks(1000);

    b.iter(|| {
        runtime.block_on(async {
            let futures: Vec<_> = tasks
                .iter()
                .map(|task| agent_pool.process_task(task.clone()))
                .collect();
            futures::future::join_all(futures).await
        })
    });
}
```

### 2. Latency Benchmarks

#### End-to-End Message Latency

```rust
pub struct LatencyBenchmark {
    samples: Vec<Duration>,
}

impl LatencyBenchmark {
    pub async fn measure_e2e_latency(&mut self, iterations: usize) {
        for _ in 0..iterations {
            let start = Instant::now();

            // Send request
            let response = self.send_request_response().await;

            let latency = start.elapsed();
            self.samples.push(latency);
        }
    }

    pub fn calculate_percentiles(&self) -> PercentileReport {
        let mut sorted = self.samples.clone();
        sorted.sort();

        PercentileReport {
            p50: sorted[sorted.len() / 2],
            p90: sorted[sorted.len() * 9 / 10],
            p95: sorted[sorted.len() * 95 / 100],
            p99: sorted[sorted.len() * 99 / 100],
            p999: sorted[sorted.len() * 999 / 1000],
            max: sorted[sorted.len() - 1],
        }
    }
}
```

#### Agent Response Time

```rust
#[bench]
fn bench_agent_response_time(b: &mut Bencher) {
    let agent = create_test_agent();
    let request = create_test_request();

    b.iter(|| {
        let start = Instant::now();
        agent.handle_request(&request);
        start.elapsed()
    });
}
```

### 3. Scalability Benchmarks

#### Horizontal Scaling Test

```rust
pub async fn benchmark_horizontal_scaling() -> ScalingReport {
    let mut report = ScalingReport::new();

    for agent_count in [1, 10, 100, 1000, 10000] {
        let orchestrator = create_orchestrator();

        // Spawn agents
        let spawn_time = measure_time(|| {
            spawn_agents(&orchestrator, agent_count)
        }).await;

        // Measure throughput at scale
        let throughput = measure_throughput(&orchestrator).await;

        // Measure resource usage
        let resources = measure_resources(&orchestrator).await;

        report.add_data_point(agent_count, spawn_time, throughput, resources);
    }

    report
}
```

#### Load Testing

```rust
pub struct LoadTest {
    target_rps: u32,
    duration: Duration,
    ramp_up: Duration,
}

impl LoadTest {
    pub async fn run(&self) -> LoadTestReport {
        let mut report = LoadTestReport::new();
        let start = Instant::now();

        while start.elapsed() < self.duration {
            let current_rps = self.calculate_current_rps(start.elapsed());

            // Generate load
            let responses = self.generate_load(current_rps).await;

            // Record metrics
            report.record_interval(IntervalMetrics {
                timestamp: Instant::now(),
                requests_sent: current_rps,
                responses_received: responses.len() as u32,
                errors: count_errors(&responses),
                latencies: extract_latencies(&responses),
            });

            tokio::time::sleep(Duration::from_secs(1)).await;
        }

        report
    }
}
```

### 4. Resource Usage Benchmarks

#### Memory Profiling

```rust
pub struct MemoryBenchmark {
    allocator: StatsAllocator,
}

impl MemoryBenchmark {
    pub async fn profile_agent_memory(&self) -> MemoryProfile {
        let initial = self.allocator.stats();

        let agent = Agent::new();
        let after_creation = self.allocator.stats();

        // Process messages
        for _ in 0..1000 {
            agent.process_message(generate_message()).await;
        }
        let after_processing = self.allocator.stats();

        // Create checkpoint
        agent.create_checkpoint().await;
        let after_checkpoint = self.allocator.stats();

        MemoryProfile {
            creation_overhead: after_creation - initial,
            processing_overhead: after_processing - after_creation,
            checkpoint_overhead: after_checkpoint - after_processing,
            total: after_checkpoint - initial,
        }
    }
}
```

#### CPU Profiling

```rust
pub async fn profile_cpu_usage() -> CpuProfile {
    let sampler = CpuSampler::new();

    // Profile different operations
    let idle = sampler.sample_duration(Duration::from_secs(10)).await;

    let processing = sampler.sample_while(|| async {
        process_messages(10000).await
    }).await;

    let spawning = sampler.sample_while(|| async {
        spawn_agents(100).await
    }).await;

    CpuProfile {
        idle_usage: idle,
        processing_usage: processing,
        spawning_usage: spawning,
    }
}
```

### 5. WebAssembly Performance

#### WASM Execution Overhead

```rust
#[bench]
fn bench_wasm_overhead(b: &mut Bencher) {
    let wasm_agent = create_wasm_agent();
    let native_agent = create_native_agent();
    let task = create_compute_task();

    let wasm_time = bench_agent(&wasm_agent, &task);
    let native_time = bench_agent(&native_agent, &task);

    println!("WASM overhead: {:.2}x slower",
             wasm_time.as_secs_f64() / native_time.as_secs_f64());
}
```

#### WASM Memory Limits

```rust
pub async fn test_wasm_memory_limits() -> MemoryLimitReport {
    let mut report = MemoryLimitReport::new();

    for memory_limit in [1, 10, 100, 1000] {
        let agent = create_wasm_agent_with_limit(memory_limit * MB);

        let max_allocation = find_max_allocation(&agent).await;
        let performance = measure_performance_at_limit(&agent).await;

        report.add_result(memory_limit, max_allocation, performance);
    }

    report
}
```

## Continuous Benchmarking

### CI/CD Integration

#### GitHub Actions Workflow

```yaml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Benchmarks
        run: |
          cargo bench --bench throughput -- --save-baseline current
          cargo bench --bench latency -- --save-baseline current
          cargo bench --bench scalability -- --save-baseline current

      - name: Compare with Baseline
        run: |
          cargo bench --bench throughput -- --baseline main
          cargo bench --bench latency -- --baseline main
          cargo bench --bench scalability -- --baseline main

      - name: Upload Results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: target/criterion/

      - name: Comment PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('target/criterion/report.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: report
            });
```

### Regression Detection

```rust
pub struct RegressionDetector {
    baseline: BenchmarkResults,
    threshold: f64, // e.g., 0.1 for 10% regression
}

impl RegressionDetector {
    pub fn detect_regressions(
        &self,
        current: &BenchmarkResults
    ) -> Vec<Regression> {
        let mut regressions = Vec::new();

        // Check throughput
        if current.throughput < self.baseline.throughput * (1.0 - self.threshold) {
            regressions.push(Regression {
                metric: "throughput",
                baseline: self.baseline.throughput,
                current: current.throughput,
                change_percent: calculate_change_percent(
                    self.baseline.throughput,
                    current.throughput
                ),
            });
        }

        // Check latency
        if current.p99_latency > self.baseline.p99_latency * (1.0 + self.threshold) {
            regressions.push(Regression {
                metric: "p99_latency",
                baseline: self.baseline.p99_latency,
                current: current.p99_latency,
                change_percent: calculate_change_percent(
                    self.baseline.p99_latency,
                    current.p99_latency
                ),
            });
        }

        regressions
    }
}
```

## Benchmark Scenarios

### Scenario 1: Peak Load

```rust
pub async fn benchmark_peak_load() {
    let config = LoadTestConfig {
        agents: 1000,
        messages_per_second: 100_000,
        duration: Duration::from_secs(300),
        message_size: 1024,
    };

    let results = run_load_test(config).await;
    assert!(results.success_rate > 0.99);
    assert!(results.p99_latency < Duration::from_millis(100));
}
```

### Scenario 2: Sustained Load

```rust
pub async fn benchmark_sustained_load() {
    let config = LoadTestConfig {
        agents: 100,
        messages_per_second: 10_000,
        duration: Duration::from_hours(24),
        message_size: 512,
    };

    let results = run_load_test(config).await;
    assert!(results.memory_leak_detected == false);
    assert!(results.performance_degradation < 0.05);
}
```

### Scenario 3: Burst Traffic

```rust
pub async fn benchmark_burst_traffic() {
    let config = BurstTestConfig {
        baseline_rps: 1000,
        burst_rps: 50000,
        burst_duration: Duration::from_secs(10),
        recovery_time_target: Duration::from_secs(30),
    };

    let results = run_burst_test(config).await;
    assert!(results.handled_burst_successfully);
    assert!(results.recovery_time < config.recovery_time_target);
}
```

## Performance Optimization Guide

### Optimization Strategies

#### 1. Message Batching

```rust
pub struct MessageBatcher {
    batch_size: usize,
    batch_timeout: Duration,
    buffer: Vec<Message>,
    last_flush: Instant,
}

impl MessageBatcher {
    pub async fn add_message(&mut self, msg: Message) {
        self.buffer.push(msg);

        if self.should_flush() {
            self.flush().await;
        }
    }

    fn should_flush(&self) -> bool {
        self.buffer.len() >= self.batch_size ||
        self.last_flush.elapsed() > self.batch_timeout
    }
}
```

#### 2. Connection Pooling

```rust
pub struct ConnectionPool {
    connections: Vec<Arc<Connection>>,
    next_connection: AtomicUsize,
}

impl ConnectionPool {
    pub fn get_connection(&self) -> Arc<Connection> {
        let index = self.next_connection.fetch_add(1, Ordering::Relaxed)
                    % self.connections.len();
        self.connections[index].clone()
    }
}
```

#### 3. Zero-Copy Optimizations

```rust
pub fn process_message_zero_copy(data: &[u8]) -> Result<()> {
    // Parse without allocation
    let message = Message::parse_borrowed(data)?;

    // Process in-place
    process_in_place(&message)?;

    // Forward without copying
    forward_borrowed(&message)?;

    Ok(())
}
```

## Benchmark Reports

### Report Format

```markdown
# Performance Benchmark Report

Date: 2025-01-15
Commit: abc123def
Environment: Production-like

## Summary

- ✅ All performance targets met
- ⚠️ 2 minor regressions detected
- 📈 15% improvement in throughput

## Detailed Results

### Throughput

| Metric       | Baseline | Current | Change |
| ------------ | -------- | ------- | ------ |
| Messages/sec | 85,000   | 97,750  | +15%   |
| Tasks/sec    | 12,000   | 11,800  | -1.7%  |

### Latency

| Percentile | Baseline | Current | Change |
| ---------- | -------- | ------- | ------ |
| p50        | 0.8ms    | 0.7ms   | -12.5% |
| p99        | 9.2ms    | 10.1ms  | +9.8%  |

### Resource Usage

| Resource     | Baseline | Current | Change |
| ------------ | -------- | ------- | ------ |
| Memory/agent | 8.5MB    | 8.2MB   | -3.5%  |
| CPU/agent    | 4.2%     | 4.0%    | -4.8%  |
```

## Monitoring Production Performance

### Real-time Metrics

```rust
pub struct ProductionMonitor {
    metrics: Arc<Metrics>,
    alerting: Arc<AlertingService>,
}

impl ProductionMonitor {
    pub async fn monitor(&self) {
        loop {
            let snapshot = self.metrics.snapshot();

            // Check against SLOs
            if snapshot.p99_latency > SLO_LATENCY {
                self.alerting.trigger(Alert::HighLatency(snapshot.p99_latency));
            }

            if snapshot.error_rate > SLO_ERROR_RATE {
                self.alerting.trigger(Alert::HighErrorRate(snapshot.error_rate));
            }

            tokio::time::sleep(Duration::from_secs(10)).await;
        }
    }
}
```

## Best Practices

1. **Benchmark Early and Often**
   - Run benchmarks on every commit
   - Track performance over time
   - Set up alerts for regressions

2. **Use Representative Workloads**
   - Model real production patterns
   - Include edge cases
   - Test failure scenarios

3. **Isolate Variables**
   - Control environment
   - Minimize noise
   - Run multiple iterations

4. **Profile Before Optimizing**
   - Identify actual bottlenecks
   - Measure impact of changes
   - Avoid premature optimization

5. **Document Performance Characteristics**
   - Known limitations
   - Scaling boundaries
   - Optimization opportunities

## References

- [Criterion.rs Documentation](https://bheisler.github.io/criterion.rs/book/)
- [cargo-flamegraph](https://github.com/flamegraph-rs/flamegraph)
- [ADR-0001: Observability-First Architecture](../adrs/0001-observability-first-architecture.md)
- [Metrics Integration Guide](../monitoring/metrics-integration-guide.md)
