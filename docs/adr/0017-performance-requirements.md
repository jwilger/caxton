---
layout: adr
title: "0017. Performance Requirements"
status: accepted
date: 2025-08-09
---

# ADR-0017: Performance Requirements

## Status
Accepted

## Context
Caxton's distributed architecture requires clear performance targets to guide implementation decisions and set user expectations. With the coordination-first architecture (ADR-0014) and distributed protocols (ADR-0015), we need explicit requirements for latency, throughput, and resource consumption.

These requirements must balance performance with operational simplicity, avoiding premature optimization that would violate our minimal core philosophy (ADR-0004).

## Decision

### Message Routing Performance

#### Latency Targets
```rust
pub struct PerformanceTargets {
    // Message routing latencies
    pub local_routing_p50: Duration,      // 100μs
    pub local_routing_p99: Duration,      // 1ms
    pub remote_routing_p50: Duration,     // 5ms
    pub remote_routing_p99: Duration,     // 50ms

    // Agent execution latencies
    pub agent_startup_p50: Duration,      // 10ms
    pub agent_startup_p99: Duration,      // 100ms
    pub message_processing_p50: Duration, // 1ms
    pub message_processing_p99: Duration, // 10ms
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            local_routing_p50: Duration::from_micros(100),
            local_routing_p99: Duration::from_millis(1),
            remote_routing_p50: Duration::from_millis(5),
            remote_routing_p99: Duration::from_millis(50),
            agent_startup_p50: Duration::from_millis(10),
            agent_startup_p99: Duration::from_millis(100),
            message_processing_p50: Duration::from_millis(1),
            message_processing_p99: Duration::from_millis(10),
        }
    }
}
```

#### Throughput Requirements
```rust
pub struct ThroughputRequirements {
    // Per-instance capacity
    pub messages_per_second: usize,        // 100,000
    pub concurrent_agents: usize,          // 10,000
    pub active_conversations: usize,       // 50,000

    // Cluster-wide capacity
    pub cluster_messages_per_second: usize, // 1,000,000
    pub cluster_total_agents: usize,        // 100,000
}
```

### Resource Consumption Limits

#### Memory Constraints
```rust
pub struct MemoryLimits {
    // Per-agent memory
    pub agent_heap_size: usize,           // 10MB default
    pub agent_stack_size: usize,          // 1MB default
    pub agent_table_elements: usize,      // 10,000
    pub agent_max_instances: usize,       // 10

    // System overhead
    pub base_memory: usize,                // 100MB
    pub per_agent_overhead: usize,         // 2MB
    pub message_buffer_size: usize,        // 500MB

    // Formula: total_memory = base_memory + (agent_count * (heap + stack + overhead)) + message_buffer
}

impl MemoryLimits {
    pub fn calculate_required_memory(&self, agent_count: usize) -> usize {
        self.base_memory
            + (agent_count * (self.agent_heap_size + self.agent_stack_size + self.per_agent_overhead))
            + self.message_buffer_size
    }

    pub fn max_agents_for_memory(&self, available_memory: usize) -> usize {
        let usable = available_memory.saturating_sub(self.base_memory + self.message_buffer_size);
        let per_agent = self.agent_heap_size + self.agent_stack_size + self.per_agent_overhead;
        usable / per_agent
    }
}
```

#### CPU Constraints
```rust
pub struct CpuLimits {
    // Per-agent CPU
    pub agent_cpu_shares: f32,            // 0.1 cores (100m)
    pub agent_burst_multiplier: f32,      // 2.0x burst allowed

    // System reservations
    pub system_reserved_cores: f32,       // 0.5 cores
    pub networking_reserved_cores: f32,   // 0.5 cores

    // Scheduling
    pub max_concurrent_executions: usize, // 2 * cpu_count
    pub execution_time_slice: Duration,   // 10ms
}
```

### Network Performance

#### SWIM Protocol Overhead
```rust
pub struct SwimPerformance {
    // Gossip parameters
    pub gossip_interval: Duration,         // 200ms
    pub gossip_fanout: usize,             // 3 nodes
    pub max_packet_size: usize,           // 1400 bytes (MTU safe)

    // Bandwidth calculation
    pub fn bandwidth_per_node(&self, cluster_size: usize) -> f64 {
        let packets_per_second = 1000.0 / self.gossip_interval.as_millis() as f64;
        let bytes_per_second = packets_per_second * self.max_packet_size as f64 * self.gossip_fanout as f64;

        // Account for both sending and receiving
        bytes_per_second * 2.0
    }

    // Convergence time
    pub fn convergence_time(&self, cluster_size: usize) -> Duration {
        // O(log N) rounds for convergence
        let rounds = (cluster_size as f64).log2().ceil() as u32;
        self.gossip_interval * rounds
    }
}
```

#### Inter-Node Communication
```rust
pub struct NetworkRequirements {
    // Bandwidth requirements
    pub min_bandwidth_mbps: f64,          // 10 Mbps minimum
    pub recommended_bandwidth_mbps: f64,   // 100 Mbps recommended

    // Latency requirements
    pub max_rtt_same_region: Duration,    // 10ms
    pub max_rtt_cross_region: Duration,   // 100ms

    // Connection limits
    pub max_connections_per_node: usize,  // 1000
    pub connection_pool_size: usize,      // 100
    pub idle_connection_timeout: Duration, // 60s
}
```

### Storage Performance

#### SQLite Local Storage
```rust
pub struct StoragePerformance {
    // Write performance
    pub agent_state_write_p50: Duration,  // 1ms
    pub agent_state_write_p99: Duration,  // 10ms
    pub batch_write_size: usize,          // 1000 records

    // Read performance
    pub agent_lookup_p50: Duration,       // 100μs
    pub agent_lookup_p99: Duration,       // 1ms

    // WAL configuration
    pub wal_checkpoint_interval: Duration, // 10s
    pub wal_size_limit: usize,            // 100MB

    // Cache configuration
    pub page_cache_size: usize,           // 50MB
    pub prepared_statement_cache: usize,   // 100
}
```

### Scalability Limits

#### Single Instance Limits
```rust
pub struct InstanceLimits {
    pub max_agents: usize,                // 10,000
    pub max_concurrent_messages: usize,   // 100,000
    pub max_message_queue_depth: usize,   // 1,000,000
    pub max_connections: usize,            // 10,000
}
```

#### Cluster Limits
```rust
pub struct ClusterLimits {
    pub max_nodes: usize,                 // 100
    pub max_total_agents: usize,          // 1,000,000
    pub max_messages_per_second: usize,   // 10,000,000

    // Scaling characteristics
    pub linear_scaling_up_to: usize,      // 20 nodes
    pub degradation_factor: f64,          // 0.9 per doubling beyond 20
}
```

### Performance Monitoring

#### Key Metrics
```rust
pub struct PerformanceMetrics {
    // Latency histograms
    routing_latency: Histogram,
    processing_latency: Histogram,
    end_to_end_latency: Histogram,

    // Throughput counters
    messages_processed: Counter,
    messages_dropped: Counter,
    agents_active: Gauge,

    // Resource utilization
    memory_used: Gauge,
    cpu_usage: Gauge,
    network_bandwidth: Gauge,

    // Error rates
    routing_errors: Counter,
    agent_crashes: Counter,
    timeout_errors: Counter,
}

impl PerformanceMetrics {
    pub fn record_routing(&self, duration: Duration) {
        self.routing_latency.record(duration);

        // Alert if P99 exceeds target
        if self.routing_latency.percentile(99.0) > Duration::from_millis(1) {
            warn!("Routing latency P99 exceeds target");
        }
    }
}
```

### Performance Testing Requirements

#### Load Testing Scenarios
```yaml
scenarios:
  steady_state:
    agents: 1000
    messages_per_second: 10000
    duration: 1h
    success_criteria:
      p99_latency: 10ms
      error_rate: < 0.01%

  peak_load:
    agents: 10000
    messages_per_second: 100000
    duration: 15m
    success_criteria:
      p99_latency: 50ms
      error_rate: < 0.1%

  sustained_load:
    agents: 5000
    messages_per_second: 50000
    duration: 24h
    success_criteria:
      p99_latency: 20ms
      error_rate: < 0.01%
      memory_growth: < 10%

  burst_traffic:
    baseline_mps: 1000
    burst_mps: 100000
    burst_duration: 10s
    success_criteria:
      message_loss: 0
      recovery_time: < 5s
```

#### Benchmark Suite
```rust
pub struct BenchmarkSuite {
    pub routing_benchmark: RoutingBenchmark,
    pub agent_startup_benchmark: StartupBenchmark,
    pub message_processing_benchmark: ProcessingBenchmark,
    pub gossip_convergence_benchmark: ConvergenceBenchmark,
}

impl BenchmarkSuite {
    pub async fn run_all(&self) -> BenchmarkResults {
        info!("Running performance benchmarks...");

        let results = BenchmarkResults {
            routing: self.routing_benchmark.run().await,
            startup: self.agent_startup_benchmark.run().await,
            processing: self.message_processing_benchmark.run().await,
            convergence: self.gossip_convergence_benchmark.run().await,
        };

        // Verify against requirements
        results.verify_requirements(&PerformanceTargets::default());
        results
    }
}
```

## Performance Optimization Strategies

### 1. Zero-Copy Message Passing
```rust
// Use bytes::Bytes for zero-copy semantics
pub struct Message {
    payload: Bytes,  // Reference-counted, zero-copy
}
```

### 2. Connection Pooling
```rust
// Reuse connections between nodes
pub struct ConnectionPool {
    connections: HashMap<NodeId, Vec<Connection>>,
    max_idle: usize,
}
```

### 3. Batch Processing
```rust
// Batch multiple operations for efficiency
pub struct BatchProcessor {
    batch_size: usize,
    flush_interval: Duration,
}
```

### 4. Async I/O
```rust
// Use tokio for async I/O
pub async fn process_messages(stream: TcpStream) {
    let (reader, writer) = stream.split();
    // Process concurrently
}
```

## Consequences

### Positive
- **Clear targets**: Teams know what performance to aim for
- **Realistic goals**: Based on proven technology capabilities
- **Monitoring built-in**: Performance tracking from day one
- **Graceful degradation**: System degrades predictably under load

### Negative
- **Complexity**: Performance optimization adds implementation complexity
- **Trade-offs**: Some features may be limited by performance constraints
- **Testing overhead**: Comprehensive performance testing required

### Neutral
- Industry-standard performance levels
- Similar to other production agent systems
- Can be tuned based on deployment needs

## Implementation Notes

### Phase 1: Establish Baselines
- Implement basic benchmarks
- Measure current performance
- Identify bottlenecks

### Phase 2: Optimize Critical Path
- Focus on message routing latency
- Optimize agent startup time
- Reduce memory overhead

### Phase 3: Scale Testing
- Test with target agent counts
- Verify cluster scaling
- Validate under sustained load

### Phase 4: Production Hardening
- Add performance regression tests
- Implement adaptive tuning
- Document tuning guidelines

## References
- [High Performance Browser Networking](https://hpbn.co/)
- [Systems Performance by Brendan Gregg](http://www.brendangregg.com/systems-performance-2nd-edition-book.html)
- [ADR-0001: Observability-First Design](0001-observability-first-design.md)
- [ADR-0014: Coordination-First Architecture](0014-coordination-first-architecture.md)
- [ADR-0015: Distributed Protocol Architecture](0015-distributed-protocol-architecture.md)
