# Performance Tuning Guide

This guide provides detailed instructions for optimizing Caxton performance. For performance targets and requirements, see [ADR-0017: Performance Requirements](../adr/0017-performance-requirements.md).

## Performance Targets

Caxton is designed to meet these performance targets:

| Metric | Target P50 | Target P99 | Measurement |
|--------|------------|------------|-------------|
| Local message routing | 100μs | 1ms | `caxton_routing_latency` |
| Remote message routing | 5ms | 50ms | `caxton_remote_routing_latency` |
| Agent startup | 10ms | 100ms | `caxton_agent_startup_time` |
| Message processing | 1ms | 10ms | `caxton_message_processing_time` |
| Gossip convergence | - | 5s | `caxton_gossip_convergence_time` |

## Quick Optimization Checklist

1. [ ] QUIC transport enabled (better than TCP)
2. [ ] MessagePack serialization (more efficient than JSON)
3. [ ] Agent pool pre-warming configured
4. [ ] Batch processing enabled where applicable
5. [ ] Resource limits properly set
6. [ ] Gossip parameters tuned for cluster size

## Configuration Tuning

### High Throughput Configuration

For maximum message throughput:

```yaml
# Optimized for throughput (100k+ msgs/sec)
runtime:
  max_agents: 5000
  agent_pool_size: 100  # Pre-warm agents
  max_concurrent_messages: 10000

messaging:
  queue_size: 100000
  batch_size: 1000  # Process in batches
  delivery_timeout: 5s
  enable_persistence: false  # Trade durability for speed

  # Use parallel processing
  parallel_routes: 8
  worker_threads: 16

coordination:
  cluster:
    # Larger gossip intervals for high throughput
    gossip_interval: 500ms
    gossip_fanout: 2  # Reduce gossip overhead

transport:
  type: quic  # Better performance than TCP
  max_streams: 1000
  congestion_control: bbr  # Better for high throughput
```

### Low Latency Configuration

For minimum message latency:

```yaml
# Optimized for latency (< 100μs P50)
runtime:
  agent_pool_size: 200  # More pre-warmed agents
  max_agents: 1000  # Fewer agents, less contention
  cpu_affinity: true  # Pin to CPU cores

messaging:
  queue_size: 10000  # Smaller queue, less queuing delay
  delivery_timeout: 1s
  priority_routing: true  # Priority queue for important messages

  # Direct routing, no batching
  batch_size: 1
  parallel_routes: 1

coordination:
  cluster:
    # Faster gossip for quick convergence
    gossip_interval: 100ms
    gossip_fanout: 4
    probe_interval: 500ms

transport:
  type: quic
  idle_timeout: 100ms  # Quick connection cleanup
  max_concurrent_streams: 100
```

### Memory-Optimized Configuration

For resource-constrained environments:

```yaml
# Optimized for low memory usage
runtime:
  max_agents: 500
  agent_pool_size: 10

  # Strict memory limits
  max_memory_per_agent: 10MB
  agent_heap_size: 5MB
  agent_stack_size: 512KB

messaging:
  queue_size: 5000
  enable_compression: true  # Trade CPU for memory

  # Aggressive cleanup
  message_ttl: 60s
  cleanup_interval: 10s

storage:
  # Compact storage settings
  type: sqlite
  page_cache_size: 10MB
  wal_size_limit: 50MB
  auto_vacuum: full
```

## SWIM Protocol Tuning

### Cluster Size Optimization

Tune SWIM parameters based on cluster size:

```yaml
# Small cluster (< 10 nodes)
coordination:
  cluster:
    gossip_interval: 100ms
    gossip_fanout: 3
    probe_interval: 500ms
    suspicion_multiplier: 3

# Medium cluster (10-50 nodes)
coordination:
  cluster:
    gossip_interval: 200ms
    gossip_fanout: 4
    probe_interval: 1s
    suspicion_multiplier: 4

# Large cluster (> 50 nodes)
coordination:
  cluster:
    gossip_interval: 500ms
    gossip_fanout: 5
    probe_interval: 2s
    suspicion_multiplier: 5
```

### Network Condition Tuning

Adjust for network conditions:

```yaml
# Low latency network (same datacenter)
coordination:
  cluster:
    probe_timeout: 200ms
    gossip_interval: 100ms
    indirect_probes: 2

# High latency network (cross-region)
coordination:
  cluster:
    probe_timeout: 2s
    gossip_interval: 1s
    indirect_probes: 5

# Unreliable network
coordination:
  cluster:
    probe_timeout: 5s
    suspicion_multiplier: 6
    gossip_to_dead: 5  # More attempts before marking dead
```

## Agent Performance

### Agent Pool Tuning

Pre-warm agents for better startup latency:

```rust
// Configure agent pool
pub struct AgentPoolConfig {
    // Number of pre-warmed instances
    pool_size: 100,

    // Warm up strategy
    warmup_strategy: WarmupStrategy::Eager,

    // Instance recycling
    max_reuse_count: 1000,
    recycle_after: Duration::from_hours(1),
}
```

### Resource Limits

Set appropriate resource limits:

```yaml
agents:
  default_limits:
    memory: 50MB
    cpu_shares: 100m  # 0.1 CPU core
    max_execution_time: 10s

  # Per-agent overrides
  overrides:
    - agent_id: heavy-processor
      memory: 500MB
      cpu_shares: 1000m  # 1 full core

    - agent_id: quick-responder
      memory: 10MB
      cpu_shares: 50m
      max_execution_time: 100ms
```

### WebAssembly Optimization

```rust
// WASM runtime optimization
pub struct WasmOptimization {
    // JIT compilation
    jit_enabled: true,
    optimization_level: OptLevel::Speed,

    // Caching
    cache_compiled_modules: true,
    module_cache_size: 100,

    // Memory management
    memory_pooling: true,
    stack_pooling: true,
}
```

## Message Routing Optimization

### Batch Processing

Process messages in batches for efficiency:

```rust
pub struct BatchProcessor {
    batch_size: 1000,
    batch_timeout: Duration::from_millis(10),
    parallel_batches: 4,
}

impl BatchProcessor {
    pub async fn process(&self) {
        // Collect messages into batches
        let batch = self.collect_batch().await;

        // Process batch in parallel
        let results = batch
            .par_iter()
            .map(|msg| self.process_message(msg))
            .collect();
    }
}
```

### Priority Routing

Implement priority queues for important messages:

```yaml
messaging:
  priority_routing:
    enabled: true
    queues:
      - name: critical
        priority: 0
        max_latency: 10ms

      - name: high
        priority: 1
        max_latency: 100ms

      - name: normal
        priority: 2
        max_latency: 1s

      - name: low
        priority: 3
        max_latency: 10s
```

### Connection Pooling

Optimize connection management:

```yaml
transport:
  connection_pool:
    min_idle: 10
    max_idle: 100
    max_lifetime: 300s
    idle_timeout: 60s

    # Per-node connection limits
    max_connections_per_node: 50

    # Connection warming
    pre_warm: true
    warm_connections: 5
```

## Storage Optimization

### SQLite Tuning

Optimize local SQLite storage:

```sql
-- Performance pragmas
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -64000;  -- 64MB cache
PRAGMA page_size = 4096;
PRAGMA mmap_size = 268435456;  -- 256MB memory-mapped I/O
PRAGMA temp_store = MEMORY;

-- Optimize queries
CREATE INDEX idx_agents_status ON agents(status);
CREATE INDEX idx_messages_timestamp ON messages(timestamp);
CREATE INDEX idx_conversations_active ON conversations(active) WHERE active = 1;
```

### Write Batching

Batch database writes:

```rust
pub struct BatchWriter {
    batch_size: 1000,
    flush_interval: Duration::from_millis(100),
}

impl BatchWriter {
    pub async fn write(&self, records: Vec<Record>) {
        // Begin transaction
        let tx = self.db.begin().await?;

        // Batch insert
        for chunk in records.chunks(self.batch_size) {
            tx.insert_batch(chunk).await?;
        }

        // Commit once
        tx.commit().await?;
    }
}
```

## Network Optimization

### QUIC Transport Tuning

```yaml
transport:
  quic:
    # Congestion control
    congestion_control: bbr  # or cubic

    # Stream management
    max_concurrent_streams: 1000
    stream_receive_window: 1MB
    connection_receive_window: 10MB

    # Keep-alive
    keep_alive_interval: 30s
    idle_timeout: 120s

    # 0-RTT for lower latency
    enable_0rtt: true

    # Datagram support
    enable_datagram: true
    max_datagram_size: 1200
```

### TCP Tuning (if not using QUIC)

```yaml
transport:
  tcp:
    # Disable Nagle's algorithm for low latency
    nodelay: true

    # Keep-alive settings
    keepalive: true
    keepalive_time: 30s
    keepalive_interval: 10s
    keepalive_probes: 3

    # Buffer sizes
    send_buffer_size: 256KB
    recv_buffer_size: 256KB

    # Connection settings
    connect_timeout: 5s
    linger: 0
```

## Monitoring Performance

### Key Metrics to Monitor

```bash
# Latency metrics
curl -s localhost:9090/metrics | grep latency
# caxton_routing_latency_seconds{quantile="0.5"} 0.0001
# caxton_routing_latency_seconds{quantile="0.99"} 0.001

# Throughput metrics
curl -s localhost:9090/metrics | grep throughput
# caxton_messages_per_second 45123
# caxton_agents_messages_processed_total 1234567

# Resource utilization
curl -s localhost:9090/metrics | grep resource
# caxton_memory_used_bytes 5242880000
# caxton_cpu_usage_percent 34.5
```

### Performance Testing

Run performance benchmarks:

```bash
# Run standard benchmark suite
caxton benchmark run --suite standard

# Custom load test
caxton benchmark custom \
  --agents 1000 \
  --messages-per-second 10000 \
  --duration 60s \
  --pattern request-reply

# Stress test to find limits
caxton benchmark stress \
  --ramp-up-time 60s \
  --max-agents 10000 \
  --find-breaking-point
```

### Profiling

Profile to identify bottlenecks:

```bash
# CPU profiling
caxton profile cpu --duration 30s --output cpu.prof

# Memory profiling
caxton profile memory --interval 1s --output mem.prof

# Trace profiling for latency analysis
caxton profile trace --messages 1000 --output trace.json
```

## Common Performance Issues

### Issue: High Message Latency

**Symptoms:**
- P99 latency > 10ms
- Message queue growing

**Diagnosis:**
```bash
caxton performance diagnose --issue high-latency
```

**Solutions:**
1. Increase worker threads
2. Enable priority routing
3. Reduce batch size
4. Check for slow agents

### Issue: Memory Growth

**Symptoms:**
- Steadily increasing memory usage
- OOM kills

**Diagnosis:**
```bash
caxton memory analyze --duration 1h
```

**Solutions:**
1. Enable message TTL
2. Reduce agent pool size
3. Increase cleanup frequency
4. Check for memory leaks in agents

### Issue: Gossip Storm

**Symptoms:**
- High network traffic
- Slow convergence

**Diagnosis:**
```bash
caxton cluster analyze-gossip
```

**Solutions:**
1. Increase gossip interval
2. Reduce gossip fanout
3. Tune suspicion multiplier
4. Check for network issues

## Performance Best Practices

1. **Measure First**: Always benchmark before optimizing
2. **Monitor Continuously**: Set up alerts for performance regression
3. **Test Under Load**: Test with realistic workloads
4. **Profile Regularly**: Regular profiling catches issues early
5. **Tune Gradually**: Make one change at a time
6. **Document Changes**: Keep records of what worked
7. **Plan Capacity**: Keep 20-30% headroom

## Advanced Optimizations

### CPU Affinity

Pin agents to specific CPU cores:

```yaml
runtime:
  cpu_affinity:
    enabled: true
    strategy: numa_aware

    # Pin critical agents
    pinned_agents:
      - agent_id: router
        cpu_cores: [0, 1]

      - agent_id: coordinator
        cpu_cores: [2, 3]
```

### NUMA Awareness

Optimize for NUMA systems:

```yaml
runtime:
  numa:
    enabled: true
    memory_policy: local_alloc
    cpu_bind: true
    interleave_memory: false
```

### Custom Memory Allocator

Use jemalloc for better performance:

```bash
# Install jemalloc
apt-get install libjemalloc-dev

# Run with jemalloc
LD_PRELOAD=/usr/lib/x86_64-linux-gnu/libjemalloc.so caxton server start
```

## References

- [ADR-0017: Performance Requirements](../adr/0017-performance-requirements.md)
- [Performance Benchmarking Guide](../benchmarks/performance-benchmarking-guide.md)
- [Testing Strategy](../development/testing-strategy.md)
- [Clustering Guide](../user-guide/clustering.md)
