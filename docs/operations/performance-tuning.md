---
title: "Performance Tuning Guide"
date: 2025-01-15
layout: page
categories: [Operations]
---

This guide provides detailed instructions for optimizing Caxton performance
with the embedded, zero-dependency architecture (ADRs 28-30). Performance
optimization focuses on embedded memory system performance, configuration
agent efficiency, and single-process deployment optimization.

## Performance Targets (Embedded Architecture)

Caxton with embedded architecture targets these performance metrics:

| Metric | Target P50 | Target P99 | Measurement |
|--------|------------|------------|-------------|
| Config agent hot reload | 50ms | 200ms | `caxton_config_reload_time` |
| Memory semantic search | 10ms | 50ms | `caxton_memory_search_latency` |
| Agent message processing | 100ms | 500ms | `caxton_agent_response_time` |
| Embedding generation | 5ms | 20ms | `caxton_embedding_latency` |
| SQLite query latency | 1ms | 10ms | `caxton_sqlite_query_time` |
| Server startup time | 2s | 5s | `caxton_server_startup_time` |

## Quick Optimization Checklist (Embedded Architecture)

1. [ ] Embedded memory system optimized (SQLite + Candle)
2. [ ] Configuration agents validated and efficient
3. [ ] Embedding model cache warmed up
4. [ ] SQLite WAL mode and pragmas configured
5. [ ] Memory cleanup intervals tuned
6. [ ] Agent hot-reload validation optimized
7. [ ] Tool call timeouts configured appropriately
8. [ ] Memory entity limits set for scaling

## Configuration Tuning (Embedded Architecture)

### High Throughput Configuration

For maximum agent processing throughput:

```yaml
# Optimized for throughput (embedded single-node)
server:
  port: 8080
  max_concurrent_conversations: 1000
  request_timeout: "30s"

# Embedded memory system optimization
memory:
  backend: "embedded"
  sqlite_config:
    wal_mode: true
    cache_size_mb: 128
    mmap_size_mb: 512
    page_size: 4096
    synchronous: "normal"
    journal_size_limit_mb: 100

  embedding_config:
    model: "all-MiniLM-L6-v2"
    cache_size: 10000  # Cache 10K embeddings
    batch_size: 32    # Process embeddings in batches
    thread_count: 4   # Parallel embedding generation

  cleanup:
    interval: "5m"    # Frequent cleanup for throughput
    entity_limit: 50000  # Allow more entities before cleanup

# Agent runtime optimization
agents:
  hot_reload_timeout: "5s"
  validation_cache_size: 100
  tool_call_timeout: "10s"
  max_memory_entities_per_agent: 5000
```

### Low Latency Configuration

For minimum response latency:

```yaml
# Optimized for latency (< 100ms P50 response time)
server:
  port: 8080
  max_concurrent_conversations: 500  # Fewer concurrent for less contention
  request_timeout: "10s"  # Shorter timeout

memory:
  backend: "embedded"
  sqlite_config:
    # Optimize for read latency
    wal_mode: true
    cache_size_mb: 256  # Larger cache for faster reads
    mmap_size_mb: 1024
    synchronous: "normal"  # Balance safety/speed
    temp_store: "memory"  # Keep temp data in memory

  embedding_config:
    model: "all-MiniLM-L6-v2"
    cache_size: 50000   # Large embedding cache
    precompute_frequent: true  # Pre-compute common embeddings
    cache_hit_target: 0.95     # Aim for 95% cache hits

  cleanup:
    interval: "10m"     # Less frequent cleanup
    batch_size: 100     # Smaller cleanup batches

agents:
  hot_reload_timeout: "2s"  # Fast reload validation
  validation_cache_size: 500  # Cache more validations
  tool_call_timeout: "5s"    # Shorter tool timeouts
  response_cache_ttl: "1m"   # Cache responses briefly
```

### Memory-Optimized Configuration

For resource-constrained environments:

```yaml
# Optimized for low memory usage (< 256MB total)
server:
  port: 8080
  max_concurrent_conversations: 100  # Limit concurrent work

memory:
  backend: "embedded"
  sqlite_config:
    cache_size_mb: 32       # Small cache
    mmap_size_mb: 64       # Limited memory mapping
    page_size: 1024        # Smaller pages
    auto_vacuum: "full"    # Compact database
    wal_size_limit_mb: 10  # Small WAL file

  embedding_config:
    model: "all-MiniLM-L6-v2"  # Already optimized model
    cache_size: 1000       # Small embedding cache
    aggressive_cleanup: true
    memory_map_model: false # Don't memory-map model

  cleanup:
    interval: "2m"         # Frequent cleanup
    entity_limit: 5000     # Low entity limit
    relation_limit: 10000  # Low relation limit
    orphan_cleanup: true   # Remove orphaned data

agents:
  max_agents: 20               # Limit total agents
  max_memory_entities_per_agent: 500
  validation_cache_size: 20
  tool_call_timeout: "15s"     # Allow longer for limited resources

  # Aggressive memory management
  conversation_limit: 10
  memory_scope: "agent"  # Isolate memory per agent
```

## Embedded Memory System Tuning

### SQLite Performance Optimization

Optimize the embedded SQLite database:

```sql
-- Essential performance pragmas for embedded memory
PRAGMA journal_mode = WAL;           -- Write-ahead logging
PRAGMA synchronous = NORMAL;         -- Balance safety/performance
PRAGMA cache_size = -131072;         -- 128MB cache (negative = KB)
PRAGMA page_size = 4096;             -- Optimal page size
PRAGMA mmap_size = 536870912;        -- 512MB memory-mapped I/O
PRAGMA temp_store = MEMORY;          -- Temp tables in memory
PRAGMA wal_autocheckpoint = 1000;    -- Checkpoint every 1000 pages
PRAGMA optimize;                     -- Optimize schema

-- Indexes for memory system queries
CREATE INDEX IF NOT EXISTS idx_entities_type ON entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON entities(name);
CREATE INDEX IF NOT EXISTS idx_relations_from ON relations(from_entity);
CREATE INDEX IF NOT EXISTS idx_relations_to ON relations(to_entity);
CREATE INDEX IF NOT EXISTS idx_observations_entity ON observations(entity_id);
CREATE INDEX IF NOT EXISTS idx_embeddings_vector ON embeddings(entity_id);
```

### Embedding Model Optimization

Optimize the All-MiniLM-L6-v2 embedding model:

```yaml
embedding_config:
  # Model configuration
  model: "all-MiniLM-L6-v2"
  model_path: "./models/"  # Local model storage

  # Performance tuning
  batch_size: 32          # Process in batches
  max_sequence_length: 384 # Model's native length
  thread_count: 4         # Parallel processing

  # Caching strategy
  cache_size: 10000       # Cache 10K embeddings
  cache_hit_target: 0.90  # 90% cache hit rate target
  precompute_common: true # Pre-compute frequent queries

  # Memory management
  embedding_ttl: "1h"     # TTL for cached embeddings
  cleanup_threshold: 0.8  # Clean when 80% full
```

### Memory System Scaling

Configure scaling thresholds and migration points:

```yaml
memory:
  scaling:
    # Embedded backend limits
    max_entities: 100000      # Migrate beyond 100K entities
    max_relations: 500000     # Migrate beyond 500K relations
    max_storage_mb: 1024      # Migrate beyond 1GB storage

    # Performance thresholds
    max_search_latency_ms: 100  # Migrate if search > 100ms P99
    min_cache_hit_rate: 0.80    # Migrate if cache hit < 80%

    # Migration triggers
    auto_migrate_enabled: false # Manual migration control
    migration_backend: "qdrant" # Target for migration
```

## Configuration Agent Performance

### Configuration Agent Optimization

Optimize configuration agents for performance:

```yaml
# In agent configuration files
---
name: OptimizedAgent
version: "1.0.0"
capabilities:
  - data-processing  # Specific, focused capabilities
tools:
  - http_client     # Only include needed tools
parameters:
  # Optimized parameters
  response_timeout: "10s"    # Appropriate timeout
  max_context_length: 4000   # Reasonable context limit

# Resource optimization
resource_limits:
  memory_scope: "agent"              # Isolated memory
  max_conversations: 50              # Conversation limit
  max_memory_entities: 1000          # Entity limit per agent
  tool_call_timeout: "5s"            # Tool timeout
  memory_search_limit: 100           # Search result limit

# Performance hints
performance:
  cache_responses: true              # Cache similar responses
  preload_memory: false             # Don't preload on startup
  lazy_tool_loading: true           # Load tools on demand
  optimize_prompts: true            # Optimize prompt templates
---
```

### Configuration Agent Resource Tuning

Set appropriate resource limits for config agents:

```yaml
# Global agent resource configuration
agents:
  default_limits:
    # Configuration agent limits
    max_conversations: 100         # Concurrent conversations
    max_memory_entities: 5000      # Memory entities per agent
    response_timeout: "30s"        # Maximum response time
    tool_call_timeout: "10s"       # Tool execution timeout
    memory_search_timeout: "5s"    # Memory search timeout

  # Per-agent type overrides
  overrides:
    - agent_type: "heavy-processor"
      max_conversations: 200
      max_memory_entities: 10000
      response_timeout: "60s"
      memory_scope: "workspace"     # Shared workspace memory

    - agent_type: "quick-responder"
      max_conversations: 50
      max_memory_entities: 1000
      response_timeout: "10s"
      memory_scope: "agent"         # Isolated memory

    - agent_type: "data-analyzer"
      max_conversations: 150
      max_memory_entities: 15000
      tool_call_timeout: "30s"      # Longer tool calls
      memory_scope: "global"        # Global memory access
```

### WASM Agent Optimization (Advanced Use Cases)

Optimize WASM agents for performance-critical scenarios:

```yaml
wasm_runtime:
  # Compilation optimization
  jit_enabled: true
  optimization_level: "speed"    # vs "size"

  # Caching strategy
  cache_compiled_modules: true
  module_cache_size: 50          # Cache 50 compiled modules
  module_cache_ttl: "1h"         # Cache TTL

  # Resource management
  memory_pooling: true           # Pool WASM memory
  stack_pooling: true            # Pool execution stacks
  instance_pooling: true         # Pool WASM instances

  # Security vs Performance
  fuel_enabled: true             # CPU metering (slight overhead)
  fuel_per_instruction: 1        # Fuel consumption rate
  max_fuel: 1000000             # Maximum execution fuel

  # Memory configuration
  max_memory_pages: 256          # 16MB max memory
  memory_growth_enabled: true    # Allow dynamic growth

# WASM agent specific limits
wasm_agents:
  default_limits:
    memory_mb: 16                # 16MB WASM memory
    fuel_limit: 1000000         # CPU fuel limit
    execution_timeout: "5s"      # Max execution time

  performance_profile:
    - profile: "cpu_intensive"
      memory_mb: 64
      fuel_limit: 5000000
      execution_timeout: "30s"

    - profile: "memory_intensive"
      memory_mb: 128
      fuel_limit: 1000000
      execution_timeout: "10s"
```

## Agent Processing Optimization

### Conversation Batch Processing

Optimize conversation processing for efficiency:

```yaml
conversation_processing:
  # Batch configuration
  batch_size: 10              # Process 10 conversations together
  batch_timeout: "100ms"      # Max wait time for batch
  parallel_conversations: 4   # Concurrent conversation processing

  # Memory optimization
  memory_batch_search: true   # Batch memory searches
  embedding_batch_size: 32    # Generate embeddings in batches

  # Caching strategy
  conversation_cache_size: 1000
  context_cache_ttl: "5m"
  response_cache_enabled: true
```

### Request Priority Handling

Implement priority queues for agent requests:

```yaml
request_handling:
  priority_queues:
    enabled: true
    queues:
      - name: "critical"
        priority: 0
        max_latency: "1s"       # Critical requests < 1s
        queue_size: 100

      - name: "high"
        priority: 1
        max_latency: "5s"       # High priority < 5s
        queue_size: 500

      - name: "normal"
        priority: 2
        max_latency: "15s"      # Normal requests < 15s
        queue_size: 1000

      - name: "background"
        priority: 3
        max_latency: "60s"      # Background tasks < 60s
        queue_size: 2000

  # Request classification
  classification:
    # Classify by agent capability
    critical_capabilities: ["emergency-response"]
    high_capabilities: ["real-time-analysis"]
    normal_capabilities: ["data-processing"]
    background_capabilities: ["batch-processing"]
```

### HTTP Connection Optimization

Optimize HTTP connections for the REST API:

```yaml
http_server:
  # Connection management
  max_connections: 1000        # Maximum concurrent connections
  connection_timeout: "30s"    # Connection timeout
  keep_alive_timeout: "60s"    # Keep-alive timeout
  request_timeout: "30s"       # Request processing timeout

  # Performance tuning
  thread_pool_size: 8          # HTTP handler threads
  request_buffer_size: 8192    # Request buffer size
  response_buffer_size: 8192   # Response buffer size

  # Compression
  enable_compression: true     # Compress responses
  compression_threshold: 1024  # Compress responses > 1KB
  compression_level: 6         # Balance speed/ratio
```

## Embedded Memory Storage Optimization

### Advanced SQLite Tuning

Optimize SQLite for embedded memory system:

```sql
-- Memory system specific pragmas
PRAGMA journal_mode = WAL;
PRAGMA synchronous = NORMAL;
PRAGMA cache_size = -262144;         -- 256MB cache for memory system
PRAGMA page_size = 4096;
PRAGMA mmap_size = 1073741824;       -- 1GB memory-mapped I/O
PRAGMA temp_store = MEMORY;
PRAGMA wal_autocheckpoint = 2000;    -- Checkpoint every 2000 pages
PRAGMA journal_size_limit = 104857600; -- 100MB WAL limit
PRAGMA optimize;                      -- Auto-optimize

-- Memory system query optimization
CREATE INDEX IF NOT EXISTS idx_entities_type_name ON entities(entity_type, name);
CREATE INDEX IF NOT EXISTS idx_entities_created ON entities(created_at);
CREATE INDEX IF NOT EXISTS idx_entities_updated ON entities(updated_at);
CREATE INDEX IF NOT EXISTS idx_relations_type ON relations(relation_type);
CREATE INDEX IF NOT EXISTS idx_relations_strength ON relations(strength);
CREATE INDEX IF NOT EXISTS idx_observations_content ON observations(content);
CREATE INDEX IF NOT EXISTS idx_embeddings_similarity ON embeddings(vector); -- For vector similarity

-- Maintenance queries
PRAGMA incremental_vacuum(1000);     -- Incremental cleanup
PRAGMA wal_checkpoint(RESTART);      -- Periodic checkpoints
ANALYZE;                             -- Update statistics
```

### Memory System Write Optimization

Optimize memory system writes for performance:

```yaml
memory_writes:
  # Batch configuration for memory operations
  entity_batch_size: 100           # Batch entity writes
  relation_batch_size: 200         # Batch relation writes
  observation_batch_size: 500      # Batch observation writes
  embedding_batch_size: 50         # Batch embedding writes

  # Transaction management
  transaction_timeout: "5s"        # Max transaction time
  max_transaction_size: 1000       # Max operations per transaction

  # Write scheduling
  flush_interval: "1s"             # Flush writes every second
  max_pending_writes: 5000         # Max pending writes

  # Compression for large observations
  compress_observations: true      # Compress large text
  compression_threshold: 1024      # Compress if > 1KB
```

## HTTP/REST API Optimization

### HTTP Server Tuning

Optimize the REST API server for embedded architecture:

```yaml
http_server:
  # Server configuration
  bind_address: "0.0.0.0:8080"
  worker_threads: 8                 # Match CPU cores
  max_blocking_threads: 32          # Blocking I/O threads

  # Connection limits
  max_connections: 1000
  connection_timeout: "30s"
  keep_alive_timeout: "60s"
  request_timeout: "30s"

  # Buffer optimization
  request_buffer_size: 16384        # 16KB request buffer
  response_buffer_size: 16384       # 16KB response buffer
  header_buffer_size: 8192          # 8KB header buffer

  # Performance features
  enable_compression: true          # gzip compression
  compression_level: 6              # Balance speed/ratio
  compression_threshold: 1024       # Compress if > 1KB

  # Caching headers
  cache_static_content: true        # Cache static responses
  cache_ttl: "5m"                  # Cache TTL
  etag_enabled: true               # ETag support
```

### API Response Optimization

```yaml
api_optimization:
  # Response format optimization
  default_format: "json"            # JSON responses
  enable_msgpack: false            # MessagePack not needed for single-node

  # Pagination for large responses
  default_page_size: 50            # Default pagination
  max_page_size: 1000             # Maximum page size

  # Response caching
  cache_agent_list: true           # Cache agent list responses
  cache_ttl: "30s"                # Short cache TTL

  # Error handling
  detailed_errors: true            # Detailed error messages
  include_stack_traces: false      # No stack traces in production
```

## Monitoring Embedded Performance

### Key Metrics to Monitor (Embedded Architecture)

```bash
# Embedded memory system metrics
curl -s localhost:9090/metrics | grep memory
# caxton_memory_entities_total 15423
# caxton_memory_relations_total 45678
# caxton_memory_search_latency_seconds{quantile="0.5"} 0.012
# caxton_memory_search_latency_seconds{quantile="0.99"} 0.048
# caxton_memory_sqlite_size_bytes 104857600
# caxton_memory_cache_hit_rate 0.87

# Configuration agent metrics
curl -s localhost:9090/metrics | grep agent
# caxton_agent_config_count 8
# caxton_agent_wasm_count 2
# caxton_agent_reload_duration_seconds{quantile="0.99"} 0.15
# caxton_agent_response_time_seconds{quantile="0.5"} 0.089
# caxton_agent_tool_call_duration_seconds{quantile="0.99"} 2.3

# Embedding model metrics
curl -s localhost:9090/metrics | grep embedding
# caxton_embedding_generation_duration_seconds{quantile="0.5"} 0.005
# caxton_embedding_cache_size 8934
# caxton_embedding_cache_hit_rate 0.92

# Server resource utilization
curl -s localhost:9090/metrics | grep server
# caxton_server_memory_used_bytes 268435456  # ~256MB
# caxton_server_cpu_usage_percent 23.4
# caxton_server_sqlite_connections 5
# caxton_server_http_requests_per_second 145
```

### Performance Testing (Embedded Architecture)

Run benchmarks for embedded system performance:

```bash
# Memory system performance test
caxton benchmark memory \
  --entities 10000 \
  --relations 50000 \
  --search-queries 1000 \
  --duration 300s

# Configuration agent load test
caxton benchmark agents \
  --config-agents 20 \
  --conversations-per-second 100 \
  --duration 600s \
  --response-timeout 30s

# Embedding generation performance
caxton benchmark embeddings \
  --batch-size 32 \
  --total-embeddings 5000 \
  --parallel-threads 4

# SQLite storage performance
caxton benchmark storage \
  --write-ops-per-second 1000 \
  --read-ops-per-second 5000 \
  --duration 300s

# Full system stress test
caxton benchmark system \
  --config-agents 50 \
  --memory-entities 100000 \
  --concurrent-conversations 500 \
  --find-limits
```

### Profiling (Embedded System)

Profile to identify embedded system bottlenecks:

```bash
# CPU profiling for embedded workloads
caxton profile cpu \
  --duration 60s \
  --focus embedding,sqlite,agents \
  --output embedded-cpu.prof

# Memory profiling for embedded memory system
caxton profile memory \
  --interval 5s \
  --track-allocations \
  --focus-on memory_system,embedding_model \
  --output embedded-mem.prof

# SQLite query profiling
caxton profile sqlite \
  --slow-query-threshold 10ms \
  --duration 300s \
  --output sqlite-queries.prof

# Agent conversation tracing
caxton profile conversations \
  --agent data-processor \
  --trace-count 100 \
  --include-memory-searches \
  --output conversation-trace.json

# Embedding model performance profiling
caxton profile embeddings \
  --batch-sizes 1,8,16,32,64 \
  --model-warmup true \
  --output embedding-perf.prof
```

## Common Performance Issues (Embedded)

### Issue: High Response Latency

**Symptoms:**

- Agent response P99 > 500ms
- Memory search latency high
- SQLite query bottlenecks

**Diagnosis:**

```bash
# Diagnose response latency
caxton diagnose latency --component agents
# Check memory system performance
caxton memory diagnose --slow-queries
# Check SQLite performance
caxton storage analyze --query-performance
```

**Solutions:**

1. Increase SQLite cache size
2. Optimize embedding cache hit rate
3. Reduce memory search scope
4. Tune agent resource limits
5. Add database indexes for frequent queries

### Issue: Memory Growth

**Symptoms:**

- Steadily increasing memory usage beyond 256MB baseline
- SQLite database growing rapidly
- Embedding cache consuming too much memory

**Diagnosis:**

```bash
# Analyze memory usage breakdown
caxton memory usage-breakdown --detailed
# Check SQLite growth patterns
caxton storage analyze --growth-rate
# Check embedding cache efficiency
caxton memory embedding-stats --cache-analysis
```

**Solutions:**

1. Increase memory cleanup frequency
2. Reduce entity/relation limits per agent
3. Enable aggressive SQLite vacuuming
4. Reduce embedding cache size
5. Implement memory scope isolation
6. Consider migration to external backend

### Issue: Slow Memory Searches

**Symptoms:**

- Memory semantic search P99 > 100ms
- Low embedding cache hit rate (<80%)
- SQLite vector query slowdowns

**Diagnosis:**

```bash
# Analyze memory search performance
caxton memory search-analysis --detailed
# Check embedding model performance
caxton memory embedding-performance
# Analyze SQLite query patterns
caxton storage query-analysis --focus-vectors
```

**Solutions:**

1. Increase embedding cache size and TTL
2. Pre-compute embeddings for common queries
3. Optimize SQLite vector indexes
4. Reduce search result limits
5. Use more specific search queries
6. Consider external vector database (Qdrant) migration

### Issue: Configuration Agent Slowdowns

**Symptoms:**

- Agent hot-reload taking > 1s
- Tool call timeouts
- YAML validation slowdowns

**Diagnosis:**

```bash
# Analyze config agent performance
caxton agents performance-analysis
# Check validation pipeline
caxton agents validation-stats
# Check tool call patterns
caxton tools performance-analysis
```

**Solutions:**

1. Cache YAML validation results
2. Optimize agent configuration size
3. Reduce tool call timeouts appropriately
4. Use lazy loading for agent tools
5. Optimize prompt template complexity

## Performance Best Practices (Embedded Architecture)

1. **Measure Embedded Metrics**: Focus on memory system, SQLite, and agent performance
2. **Monitor Memory Growth**: Track entity/relation counts and database size
3. **Optimize Cache Hit Rates**: Target >90% for embeddings, >80% for memory searches
4. **Tune SQLite Aggressively**: Use WAL mode, large cache, memory mapping
5. **Plan Migration Path**: Know when to migrate from embedded to external backends
6. **Profile Regularly**: Focus on SQLite queries, embedding generation, agent processing
7. **Configuration Validation**: Cache YAML validation to avoid repeated parsing
8. **Resource Boundaries**: Set appropriate limits for memory entities per agent
9. **Cleanup Strategy**: Regular cleanup of old entities, relations, conversations
10. **Single-Process Optimization**: Optimize for single-node performance, not distribution

## Advanced Optimizations (Embedded)

### CPU Optimization for Embedded Workloads

Optimize CPU usage for embedded memory and agent processing:

```yaml
runtime:
  cpu_optimization:
    # Thread allocation
    http_worker_threads: 4           # HTTP request handling
    memory_worker_threads: 2         # Memory system operations
    agent_worker_threads: 4          # Agent processing
    embedding_threads: 2             # Embedding generation

    # CPU affinity (optional)
    enable_cpu_affinity: false       # Usually not needed for embedded

    # Thread priorities
    high_priority_threads:
      - "http_workers"              # Prioritize API responses
      - "memory_search"             # Prioritize memory searches

    # Async runtime optimization
    async_runtime: "tokio"          # Use Tokio for async
    tokio_worker_threads: 8         # Total Tokio threads
    tokio_max_blocking_threads: 16  # Blocking thread pool
```

### Memory Allocator Optimization

Optimize memory allocation for embedded workloads:

```yaml
memory_allocation:
  # Allocator selection
  allocator: "system"              # "system", "jemalloc", "mimalloc"

  # Allocation strategy for embedding model
  embedding_model:
    memory_mapping: true            # Memory-map model files
    prefault_pages: true           # Pre-fault memory pages
    huge_pages: false              # Usually not beneficial

  # SQLite memory allocation
  sqlite_memory:
    page_cache_allocator: "malloc" # Use malloc for page cache
    temp_allocator: "malloc"       # Temp storage allocation

  # Agent memory management
  agent_memory:
    conversation_pool: true        # Pool conversation objects
    response_pool: true           # Pool response objects
    pool_size: 1000              # Object pool size
```

### Advanced Memory System Tuning

Fine-tune the embedded memory system for optimal performance:

```yaml
advanced_memory_tuning:
  # SQLite advanced configuration
  sqlite_advanced:
    wal_checkpoint_strategy: "passive"  # "passive", "full", "restart"
    wal_checkpoint_interval: 2000       # Pages before checkpoint
    analyze_frequency: "1h"             # ANALYZE statistics update
    vacuum_strategy: "incremental"      # "incremental", "full", "auto"

  # Embedding model optimization
  embedding_advanced:
    model_precision: "fp16"             # "fp32", "fp16", "int8"
    batch_optimization: true            # Optimize batch processing
    prefetch_batches: 2                 # Prefetch next batches

  # Vector similarity optimization
  vector_similarity:
    algorithm: "cosine"                 # "cosine", "dot", "euclidean"
    precision_threshold: 0.001          # Similarity precision
    early_termination: true             # Stop search early if possible

  # Memory cleanup optimization
  cleanup_advanced:
    cleanup_strategy: "lru"             # "lru", "age", "size"
    cleanup_batch_size: 1000            # Clean items in batches
    cleanup_thread_count: 2             # Parallel cleanup threads
```

## References

- [ADR-0030: Embedded Memory System](../adr/0030-embedded-memory-system.md)
- [ADR-0028: Configuration-Driven Agent Architecture](../adr/0028-configuration-driven-agent-architecture.md)
- [ADR-0029: FIPA-ACL Lightweight Messaging](../adr/0029-fipa-acl-lightweight-messaging.md)
- [Agent Lifecycle Management](agent-lifecycle-management.md)
- [Operational Runbook](operational-runbook.md)
- [DevOps Security Guide](devops-security-guide.md)
