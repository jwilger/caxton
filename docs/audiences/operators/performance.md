---
title: "Performance Optimization Guide"
description: "Comprehensive performance tuning for Caxton embedded architecture"
date: 2025-01-15
layout: page
categories: [Operations, Performance, SysOps]
nav_order: 2
parent: Operators
---

This guide provides detailed instructions for optimizing Caxton performance
with the embedded, zero-dependency architecture (ADRs 28-30). Focus areas
include embedded memory system optimization, configuration agent efficiency,
and production deployment tuning.

## Performance Targets

### Production SLAs (Embedded Architecture)

| Metric | Target P50 | Target P99 | Critical Threshold |
|--------|------------|------------|-------------------|
| Config agent hot reload | 50ms | 200ms | 1s |
| Memory semantic search | 10ms | 50ms | 200ms |
| Agent message processing | 100ms | 500ms | 2s |
| Embedding generation | 5ms | 20ms | 100ms |
| SQLite query latency | 1ms | 10ms | 50ms |
| Server startup time | 2s | 5s | 30s |
| API response time | 50ms | 200ms | 1s |
| Memory usage baseline | 256MB | 400MB | 1GB |

### Operational Monitoring Commands

```bash
# Real-time performance monitoring
watch -n 5 'curl -s localhost:9090/metrics | grep -E "caxton_(memory|agent|server)_"'

# Memory system performance check
caxton memory stats --detailed --last 1h

# Agent performance analysis
caxton agents performance-summary --show-slowest 10

# System resource utilization
caxton system resources --breakdown --alerts
```

## Quick Performance Checklist

### Daily Operations Checklist

1. [ ] **Memory system health**: Search latency P99 < 50ms
2. [ ] **SQLite optimization**: WAL mode enabled, cache sized appropriately
3. [ ] **Embedding cache**: Hit rate > 85%
4. [ ] **Agent validation**: Config validation cached and fast
5. [ ] **Cleanup schedules**: Memory cleanup running every 5-10 minutes
6. [ ] **Resource limits**: No agents exceeding memory/CPU limits
7. [ ] **Database size**: SQLite database < 1GB (migration planning)
8. [ ] **Response times**: API responses P99 < 200ms

### Weekly Performance Review

```bash
#!/bin/bash
# Weekly performance review script
echo "=== WEEKLY CAXTON PERFORMANCE REVIEW ==="
echo "Date: $(date)"

echo "\n1. Memory System Health:"
caxton memory capacity-check --percentage
caxton memory stats --trends --last 7d

echo "\n2. Agent Performance Summary:"
caxton agents performance-report --last 7d --include-slowest

echo "\n3. Resource Utilization:"
caxton system resources --trends --last 7d

echo "\n4. Database Health:"
caxton storage analyze --growth-rate --fragmentation

echo "\n5. Cache Efficiency:"
caxton memory cache-analysis --hit-rates --efficiency

echo "\n6. Alerts and Issues:"
caxton alerts summary --last 7d --severity warning,critical
```

## Configuration Optimization

### Production High-Throughput Configuration

```yaml
# Production configuration for high message throughput
server:
  port: 8080
  max_concurrent_conversations: 2000
  request_timeout: "30s"
  worker_threads: 8                    # Match CPU cores
  max_blocking_threads: 32             # For I/O operations

# Embedded memory system optimization for throughput
memory:
  backend: "embedded"
  sqlite_config:
    wal_mode: true                     # Write-ahead logging (critical)
    cache_size_mb: 256                 # Large cache for throughput
    mmap_size_mb: 1024                 # Memory-mapped I/O
    page_size: 4096                    # Optimal page size
    synchronous: "normal"              # Balance safety/speed
    journal_size_limit_mb: 100         # WAL size limit
    temp_store: "memory"               # Temp data in memory
    optimize_interval: "1h"            # Auto-optimize every hour

  embedding_config:
    model: "all-MiniLM-L6-v2"          # Optimized model choice
    cache_size: 20000                  # Cache 20K embeddings
    batch_size: 64                     # Larger batches for throughput
    thread_count: 4                    # Parallel embedding generation
    precompute_frequent: true          # Pre-compute common queries

  cleanup:
    interval: "5m"                     # Frequent cleanup
    entity_limit: 80000                # Higher limit for throughput
    batch_size: 1000                   # Larger cleanup batches

# Agent runtime optimization for throughput
agents:
  hot_reload_timeout: "5s"
  validation_cache_size: 500           # Cache more validations
  tool_call_timeout: "10s"
  max_memory_entities_per_agent: 10000
  conversation_batch_size: 50          # Process conversations in batches
  response_cache_ttl: "5m"             # Cache responses
```

### Production Low-Latency Configuration

```yaml
# Optimized for minimum response latency (< 50ms P50)
server:
  port: 8080
  max_concurrent_conversations: 1000   # Fewer for less contention
  request_timeout: "15s"               # Shorter timeout
  worker_threads: 4                    # Dedicated workers
  request_buffer_size: 32768           # Larger buffers

memory:
  backend: "embedded"
  sqlite_config:
    wal_mode: true
    cache_size_mb: 512                 # Very large cache
    mmap_size_mb: 2048                 # Aggressive memory mapping
    synchronous: "normal"
    temp_store: "memory"
    wal_checkpoint_interval: 5000      # Less frequent checkpoints

  embedding_config:
    model: "all-MiniLM-L6-v2"
    cache_size: 50000                  # Massive embedding cache
    precompute_frequent: true
    cache_hit_target: 0.98             # Aim for 98% cache hits
    prefetch_batches: 4                # Prefetch embeddings

  cleanup:
    interval: "10m"                    # Less frequent cleanup
    batch_size: 100                    # Smaller batches
    low_priority: true                 # Background cleanup

agents:
  hot_reload_timeout: "2s"             # Fast validation
  validation_cache_size: 1000          # Cache many validations
  tool_call_timeout: "5s"              # Quick tool calls
  response_cache_ttl: "2m"             # Cache responses longer
  memory_search_timeout: "50ms"        # Strict search timeout
  conversation_priority_queue: true    # Priority-based processing
```

### Resource-Constrained Configuration

```yaml
# Optimized for environments with limited resources (< 512MB RAM)
server:
  port: 8080
  max_concurrent_conversations: 200    # Limited concurrency
  worker_threads: 2                    # Minimal workers
  max_blocking_threads: 8              # Limited I/O threads

memory:
  backend: "embedded"
  sqlite_config:
    cache_size_mb: 64                  # Small cache
    mmap_size_mb: 128                  # Limited memory mapping
    page_size: 1024                    # Smaller pages
    auto_vacuum: "incremental"         # Incremental vacuuming
    wal_size_limit_mb: 20              # Small WAL
    temp_store: "file"                 # Use disk for temp data

  embedding_config:
    model: "all-MiniLM-L6-v2"          # Still use optimized model
    cache_size: 2000                   # Small cache
    aggressive_cleanup: true           # Aggressive memory cleanup
    memory_map_model: false            # Don't memory-map model
    batch_size: 16                     # Smaller batches

  cleanup:
    interval: "2m"                     # Frequent cleanup
    entity_limit: 10000                # Low entity limit
    relation_limit: 25000              # Low relation limit
    orphan_cleanup: true               # Remove orphaned data
    compress_old_data: true            # Compress old entities

agents:
  max_agents: 50                       # Limit total agents
  max_memory_entities_per_agent: 1000  # Low per-agent limit
  validation_cache_size: 50            # Small validation cache
  conversation_limit: 20               # Limit conversations
  memory_scope: "agent"                # Isolate memory per agent
  tool_call_timeout: "20s"             # Allow longer for limited resources
```

## Embedded Memory System Optimization

### SQLite Performance Tuning

**Essential SQLite pragmas for production:**

```sql
-- Production SQLite configuration
PRAGMA journal_mode = WAL;              -- Write-ahead logging (essential)
PRAGMA synchronous = NORMAL;            -- Balance safety/performance
PRAGMA cache_size = -262144;            -- 256MB cache (-ve = KB)
PRAGMA page_size = 4096;                -- Optimal page size
PRAGMA mmap_size = 1073741824;          -- 1GB memory-mapped I/O
PRAGMA temp_store = MEMORY;             -- Temp tables in memory
PRAGMA wal_autocheckpoint = 2000;       -- Checkpoint every 2000 pages
PRAGMA journal_size_limit = 134217728; -- 128MB WAL limit
PRAGMA secure_delete = OFF;             -- Faster deletes
PRAGMA automatic_index = ON;            -- Auto-create helpful indexes
PRAGMA optimize;                        -- Optimize query planner

-- Memory system indexes for performance
CREATE INDEX IF NOT EXISTS idx_entities_composite
  ON entities(entity_type, name, created_at);
CREATE INDEX IF NOT EXISTS idx_entities_updated
  ON entities(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_relations_composite
  ON relations(from_entity, to_entity, relation_type);
CREATE INDEX IF NOT EXISTS idx_relations_strength
  ON relations(strength DESC);
CREATE INDEX IF NOT EXISTS idx_observations_entity_time
  ON observations(entity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_embeddings_search
  ON embeddings(entity_id, vector_hash);
```

**SQLite maintenance operations:**

```bash
# Daily SQLite maintenance
caxton storage maintenance --vacuum-incremental --analyze-stats

# Weekly full optimization
caxton storage maintenance --vacuum-full --reindex-all --analyze-full

# Monthly database optimization
caxton storage optimize --aggressive --verify-integrity
```

### Embedding Model Optimization

**Advanced embedding configuration:**

```yaml
embedding_config:
  # Model configuration
  model: "all-MiniLM-L6-v2"
  model_path: "/var/lib/caxton/models/"
  model_format: "safetensors"          # Safer, faster format

  # Performance tuning
  batch_size: 64                       # Optimal batch size
  max_sequence_length: 384             # Model's native length
  thread_count: 4                      # Match available cores
  device: "cpu"                        # CPU optimized

  # Advanced caching
  cache_size: 25000                    # Cache 25K embeddings
  cache_hit_target: 0.92               # 92% cache hit target
  precompute_common: true              # Pre-compute frequent queries
  smart_caching: true                  # Cache based on usage patterns

  # Memory management
  embedding_ttl: "2h"                  # TTL for cached embeddings
  cleanup_threshold: 0.85              # Clean when 85% full
  memory_pool_size: "128MB"            # Dedicated memory pool

  # Optimization features
  quantization: "int8"                 # 8-bit quantization for speed
  model_optimization: true             # Use optimized model variants
  prefetch_enabled: true               # Prefetch next batches
```

**Embedding performance monitoring:**

```bash
# Monitor embedding performance
caxton memory embedding-stats --detailed

# Expected output:
# Embedding Model: All-MiniLM-L6-v2
# Cache size: 23,847 embeddings
# Cache hit rate: 94.2%
# Average generation time: 4.2ms
# Batch processing efficiency: 87%
# Memory usage: 87MB / 128MB pool
```

### Memory System Scaling and Migration

**Scaling thresholds and monitoring:**

```yaml
memory_scaling:
  # Embedded backend limits (production)
  limits:
    max_entities: 100000               # Hard limit for embedded
    max_relations: 500000              # Relationship limit
    max_storage_mb: 2048               # 2GB storage limit
    max_memory_mb: 512                 # 512MB memory limit

  # Performance thresholds for migration planning
  migration_triggers:
    search_latency_p99: 100            # Migrate if search > 100ms
    cache_hit_rate_min: 0.80           # Migrate if cache < 80%
    storage_growth_rate: 100           # MB per day growth limit

  # Migration configuration
  migration_planning:
    auto_migrate_enabled: false        # Manual control recommended
    target_backend: "qdrant"           # Primary migration target
    backup_before_migrate: true        # Always backup first
    rollback_enabled: true             # Enable rollback capability
```

**Migration readiness check:**

```bash
# Check if approaching migration thresholds
caxton memory migration-readiness --detailed

# Expected output:
# === MIGRATION READINESS REPORT ===
# Entity count: 78,542 / 100,000 (78% - OK)
# Storage size: 1.2GB / 2GB (60% - OK)
# Search latency P99: 67ms / 100ms (OK)
# Cache hit rate: 88% / 80% (OK)
# Growth rate: 45MB/day (OK)
#
# STATUS: Not ready for migration
# TIME TO LIMITS: ~6 months at current growth
# RECOMMENDATION: Continue monitoring
```

## Configuration Agent Performance

### Agent Configuration Optimization

**High-performance agent template:**

```toml
name = "OptimizedDataProcessor"
version = "1.0.0"
capabilities = ["data-processing"]  # Specific, focused capability
tools = ["http_client", "csv_parser"]  # Only necessary tools

[memory]
enabled = true
scope = "workspace"  # Appropriate scope
auto_store = false   # Manual memory control
search_limit = 50    # Limit search results

# Performance optimization parameters
[parameters]
response_timeout = "15s"      # Appropriate timeout
max_context_length = 6000     # Reasonable context limit
batch_processing = true       # Enable batch processing
cache_responses = true        # Cache similar responses

# Resource management
[resource_limits]
max_conversations = 100              # Conversation limit
max_memory_entities = 5000           # Memory entity limit
tool_call_timeout = "8s"             # Tool timeout
memory_search_timeout = "50ms"       # Fast memory searches
context_switch_limit = 10            # Limit context switches

# Performance hints
[performance]
lazy_tool_loading = true             # Load tools on demand
optimize_prompts = true              # Optimize templates
preload_memory = false               # Don't preload unless needed
parallel_tool_calls = true          # Allow parallel tool execution
response_streaming = true            # Stream responses when possible

system_prompt = '''
You are an optimized data processing agent. Focus on efficient,
accurate data analysis with minimal resource usage.

Performance guidelines:
- Use specific, targeted queries
- Minimize memory searches
- Cache frequently accessed data
- Process data in batches when possible
- Provide concise, structured responses
'''

documentation = '''
# Optimized Data Processor

High-performance agent for data analysis tasks with optimized resource usage
and response times.
'''
```

**Agent resource tuning by use case:**

```yaml
# Global agent configuration
agents:
  # Default limits for all agents
  default_limits:
    max_conversations: 200
    max_memory_entities: 8000
    response_timeout: "30s"
    tool_call_timeout: "12s"
    memory_search_timeout: "100ms"
    context_length_limit: 8000

  # Performance profiles for different agent types
  performance_profiles:
    # Quick response agents (< 100ms target)
    - profile: "quick_response"
      agent_types: ["status-checker", "simple-query"]
      overrides:
        max_conversations: 500
        max_memory_entities: 1000
        response_timeout: "5s"
        tool_call_timeout: "3s"
        memory_search_timeout: "25ms"
        memory_scope: "agent"          # Isolated memory
        preload_memory: true           # Preload for speed

    # Heavy processing agents
    - profile: "heavy_processor"
      agent_types: ["data-analyzer", "report-generator"]
      overrides:
        max_conversations: 50
        max_memory_entities: 20000
        response_timeout: "120s"
        tool_call_timeout: "60s"
        memory_search_timeout: "500ms"
        memory_scope: "global"         # Full memory access
        batch_processing: true         # Enable batching

    # Real-time agents
    - profile: "realtime"
      agent_types: ["monitor", "alert-handler"]
      overrides:
        max_conversations: 1000
        max_memory_entities: 5000
        response_timeout: "10s"
        tool_call_timeout: "5s"
        memory_search_timeout: "50ms"
        priority: "high"               # High priority processing
        dedicated_resources: true      # Dedicated resources
```

### WASM Agent Performance Optimization

**WASM runtime tuning for performance:**

```yaml
wasm_runtime:
  # Compilation optimization
  jit_enabled: true                    # Enable JIT compilation
  optimization_level: "speed"          # Optimize for speed vs size
  compiler: "cranelift"                # Fast compilation

  # Instance management
  cache_compiled_modules: true         # Cache compiled WASM
  module_cache_size: 100               # Cache 100 modules
  module_cache_ttl: "2h"               # Cache TTL

  # Resource pooling
  memory_pooling: true                 # Pool WASM memory pages
  stack_pooling: true                  # Pool execution stacks
  instance_pooling: true               # Pool WASM instances
  pool_size: 50                        # Pool size per type

  # Performance vs Security balance
  fuel_enabled: true                   # CPU metering (slight overhead)
  fuel_per_instruction: 1              # Fuel consumption rate
  bounds_checks: "optimized"           # Optimized bounds checking

  # Memory configuration
  memory_page_size: 65536              # 64KB pages (WASM standard)
  max_memory_pages: 1024               # 64MB max memory per instance
  memory_growth_enabled: true          # Dynamic memory growth
  memory_initialization: "lazy"        # Lazy memory initialization

# WASM agent performance profiles
wasm_agents:
  default_limits:
    memory_mb: 32                      # 32MB WASM memory
    fuel_limit: 2000000                # CPU fuel limit
    execution_timeout: "10s"           # Max execution time
    stack_size_kb: 512                 # 512KB stack

  # Performance-critical WASM agents
  performance_profiles:
    - profile: "cpu_intensive"
      memory_mb: 128                   # More memory for CPU tasks
      fuel_limit: 10000000             # Higher CPU limit
      execution_timeout: "60s"         # Longer execution time
      optimization_level: "aggressive" # Aggressive optimization

    - profile: "memory_intensive"
      memory_mb: 256                   # High memory limit
      fuel_limit: 2000000              # Standard CPU limit
      execution_timeout: "30s"         # Moderate execution time
      garbage_collection: "frequent"   # Frequent GC

    - profile: "realtime"
      memory_mb: 64                    # Moderate memory
      fuel_limit: 1000000              # Limited CPU for fairness
      execution_timeout: "1s"          # Very fast execution
      priority: "high"                 # High scheduling priority
```

## Advanced Performance Optimizations

### Message Processing Optimization

**Batch processing configuration:**

```yaml
message_processing:
  # Batch configuration for throughput
  batch_processing:
    enabled: true
    batch_size: 100                    # Messages per batch
    batch_timeout: "50ms"              # Max wait for batch
    max_batches_queued: 10             # Limit queued batches

  # Parallel processing
  parallel_processing:
    enabled: true
    worker_threads: 8                  # Processing threads
    work_stealing: true                # Work-stealing scheduler
    thread_affinity: false             # Usually not needed

  # Message prioritization
  priority_queues:
    enabled: true
    queue_count: 4                     # Number of priority queues
    priorities:
      - name: "critical"
        max_latency_ms: 100            # Critical < 100ms
        queue_size: 100
        dedicated_threads: 2           # Dedicated processing

      - name: "high"
        max_latency_ms: 500            # High priority < 500ms
        queue_size: 1000
        dedicated_threads: 4

      - name: "normal"
        max_latency_ms: 2000           # Normal < 2s
        queue_size: 5000
        dedicated_threads: 2

      - name: "background"
        max_latency_ms: 10000          # Background < 10s
        queue_size: 10000
        dedicated_threads: 1

  # Performance optimizations
  optimizations:
    zero_copy_enabled: true            # Zero-copy message handling
    message_pooling: true              # Pool message objects
    compression_enabled: true          # Compress large messages
    compression_threshold: 4096        # Compress if > 4KB
```

### HTTP/REST API Performance Tuning

**Production HTTP server configuration:**

```yaml
http_server:
  # Core server configuration
  bind_address: "0.0.0.0:8080"
  worker_threads: 8                    # HTTP worker threads
  max_blocking_threads: 32             # Blocking I/O thread pool
  thread_stack_size: "2MB"             # Thread stack size

  # Connection management
  max_connections: 2000                # Maximum concurrent connections
  connection_timeout: "30s"            # Connection establishment timeout
  keep_alive_timeout: "120s"           # HTTP keep-alive timeout
  request_timeout: "60s"               # Request processing timeout
  max_request_size: "16MB"             # Maximum request size

  # Buffer optimization
  request_buffer_size: 32768           # 32KB request buffer
  response_buffer_size: 32768          # 32KB response buffer
  header_buffer_size: 16384            # 16KB header buffer
  body_buffer_size: 131072             # 128KB body buffer

  # Performance features
  compression:
    enabled: true                      # Enable gzip compression
    level: 6                           # Compression level (1-9)
    threshold: 2048                    # Compress responses > 2KB
    types: ["application/json", "text/plain"]

  # Caching
  response_caching:
    enabled: true                      # Cache static responses
    cache_size: "64MB"                 # Response cache size
    default_ttl: "300s"                # Default cache TTL
    vary_headers: ["Accept-Encoding"]  # Vary by these headers

  # Rate limiting
  rate_limiting:
    enabled: true
    global_rate: 10000                 # Requests per second globally
    per_ip_rate: 100                   # Requests per second per IP
    burst_size: 200                    # Burst allowance

  # Health and monitoring
  health_check:
    path: "/api/v1/health"
    timeout: "5s"
    interval: "10s"

  metrics:
    path: "/metrics"
    enabled: true
    detailed: true                     # Include detailed metrics
```

**API response optimization:**

```yaml
api_optimization:
  # Response formatting
  response_format:
    default: "json"                    # Default response format
    compression_enabled: true          # Compress responses
    pretty_print: false                # Disable pretty printing

  # Pagination for large datasets
  pagination:
    default_page_size: 100             # Default items per page
    max_page_size: 1000                # Maximum page size
    page_size_header: "X-Page-Size"    # Page size header
    total_count_header: "X-Total-Count" # Total count header

  # Response caching strategy
  caching:
    # Cache agent list responses (frequently requested)
    agent_list:
      enabled: true
      ttl: "30s"                       # Short TTL for agent lists
      vary_by: ["type", "status"]      # Cache variations

    # Cache agent details (less frequent changes)
    agent_details:
      enabled: true
      ttl: "300s"                      # Longer TTL for details
      vary_by: ["id"]

    # Cache memory statistics
    memory_stats:
      enabled: true
      ttl: "60s"                       # 1 minute cache

    # Don't cache real-time data
    real_time_endpoints:
      - "/api/v1/agents/*/messages"
      - "/api/v1/conversations/*/messages"
      - "/api/v1/system/status"

  # Error handling
  error_responses:
    detailed_errors: true              # Include detailed error info
    include_request_id: true           # Include request ID for tracing
    include_suggestions: true          # Include fix suggestions
    stack_traces: false                # Never include stack traces

  # Content negotiation
  content_negotiation:
    default_accept: "application/json"
    supported_types:
      - "application/json"
      - "text/plain"
      - "application/msgpack"          # For high-performance clients
```

## Performance Monitoring and Alerting

### Critical Performance Metrics

**Primary metrics to monitor continuously:**

```bash
# Memory system performance
curl -s localhost:9090/metrics | grep -E "caxton_memory_" | head -20

# Key metrics:
# caxton_memory_entities_total{} 45628
# caxton_memory_relations_total{} 138472
# caxton_memory_search_latency_seconds{quantile="0.5"} 0.008
# caxton_memory_search_latency_seconds{quantile="0.99"} 0.045
# caxton_memory_sqlite_size_bytes{} 587202560
# caxton_memory_cache_hit_rate{type="embedding"} 0.94
# caxton_memory_cache_hit_rate{type="query"} 0.87

# Configuration agent performance
curl -s localhost:9090/metrics | grep -E "caxton_agent_" | head -15

# Key metrics:
# caxton_agent_config_count{} 12
# caxton_agent_wasm_count{} 3
# caxton_agent_reload_duration_seconds{quantile="0.99"} 0.087
# caxton_agent_response_time_seconds{quantile="0.5"} 0.156
# caxton_agent_response_time_seconds{quantile="0.99"} 0.823
# caxton_agent_tool_call_duration_seconds{quantile="0.99"} 3.245
# caxton_agent_memory_entities_per_agent{quantile="0.99"} 8934

# Server and API performance
curl -s localhost:9090/metrics | grep -E "caxton_server_" | head -10

# Key metrics:
# caxton_server_request_duration_seconds{quantile="0.5"} 0.023
# caxton_server_request_duration_seconds{quantile="0.99"} 0.189
# caxton_server_memory_used_bytes{} 387973120  # ~370MB
# caxton_server_cpu_usage_percent{} 18.7
# caxton_server_active_connections{} 47
# caxton_server_requests_per_second{} 285
```

### Performance Alerting Configuration

**Prometheus alerting rules for performance:**

```yaml
# caxton-performance-alerts.yml
groups:
- name: caxton.performance
  rules:
  # Memory system performance alerts
  - alert: CaxtonMemorySearchSlow
    expr: caxton_memory_search_latency_seconds{quantile="0.99"} > 0.1
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Caxton memory searches are slow"
      description: "Memory search P99 latency is {{ $value }}s (threshold: 100ms)"

  - alert: CaxtonMemorySearchCritical
    expr: caxton_memory_search_latency_seconds{quantile="0.99"} > 0.5
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "Caxton memory searches critically slow"
      description: "Memory search P99 latency is {{ $value }}s (critical: 500ms)"

  - alert: CaxtonMemoryCacheHitLow
    expr: caxton_memory_cache_hit_rate{type="embedding"} < 0.80
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Low embedding cache hit rate"
      description: "Embedding cache hit rate is {{ $value }} (target: >85%)"

  # Agent performance alerts
  - alert: CaxtonAgentResponseSlow
    expr: caxton_agent_response_time_seconds{quantile="0.99"} > 2.0
    for: 3m
    labels:
      severity: warning
    annotations:
      summary: "Agent responses are slow"
      description: "Agent response P99 time is {{ $value }}s (threshold: 2s)"

  - alert: CaxtonAgentReloadSlow
    expr: caxton_agent_reload_duration_seconds{quantile="0.99"} > 1.0
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Agent hot-reload is slow"
      description: "Agent reload P99 time is {{ $value }}s (threshold: 1s)"

  # System resource alerts
  - alert: CaxtonHighMemoryUsage
    expr: caxton_server_memory_used_bytes > 1073741824  # 1GB
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High memory usage"
      description: "Memory usage is {{ $value | humanize1024 }}B (warning: >1GB)"

  - alert: CaxtonHighCPUUsage
    expr: caxton_server_cpu_usage_percent > 80
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High CPU usage"
      description: "CPU usage is {{ $value }}% (warning: >80%)"

  # Database performance alerts
  - alert: CaxtonSQLiteGrowthFast
    expr: increase(caxton_memory_sqlite_size_bytes[24h]) > 104857600  # 100MB/day
    for: 1h
    labels:
      severity: warning
    annotations:
      summary: "Fast SQLite database growth"
      description: "Database grew {{ $value | humanize1024 }}B in 24h (warning: >100MB/day)"

  - alert: CaxtonSQLiteSizeLarge
    expr: caxton_memory_sqlite_size_bytes > 2147483648  # 2GB
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "SQLite database is very large"
      description: "Database size is {{ $value | humanize1024 }}B (migration recommended: >2GB)"
```

### Performance Dashboards

**Grafana dashboard configuration for operators:**

```json
{
  "dashboard": {
    "title": "Caxton Performance Dashboard",
    "panels": [
      {
        "title": "Message Throughput",
        "type": "stat",
        "targets": [
          {
            "expr": "rate(caxton_messages_processed_total[5m])",
            "legendFormat": "Messages/sec"
          }
        ]
      },
      {
        "title": "Response Latency",
        "type": "graph",
        "targets": [
          {
            "expr": "caxton_server_request_duration_seconds{quantile=\"0.5\"}",
            "legendFormat": "P50"
          },
          {
            "expr": "caxton_server_request_duration_seconds{quantile=\"0.99\"}",
            "legendFormat": "P99"
          }
        ]
      },
      {
        "title": "Memory System Performance",
        "type": "graph",
        "targets": [
          {
            "expr": "caxton_memory_search_latency_seconds{quantile=\"0.99\"}",
            "legendFormat": "Search Latency P99"
          },
          {
            "expr": "caxton_memory_cache_hit_rate{type=\"embedding\"}",
            "legendFormat": "Embedding Cache Hit Rate"
          }
        ]
      },
      {
        "title": "Resource Utilization",
        "type": "graph",
        "targets": [
          {
            "expr": "caxton_server_memory_used_bytes",
            "legendFormat": "Memory Usage"
          },
          {
            "expr": "caxton_server_cpu_usage_percent",
            "legendFormat": "CPU Usage %"
          }
        ]
      }
    ]
  }
}
```

## Performance Benchmarking

### Production Benchmarking Suite

**Automated performance testing:**

```bash
#!/bin/bash
# Production performance benchmarking script

BENCHMARK_DIR="/var/log/caxton/benchmarks"
DATE=$(date +%Y%m%d-%H%M%S)
RESULTS_FILE="$BENCHMARK_DIR/benchmark-$DATE.json"

echo "Starting Caxton performance benchmark suite..."

# 1. Memory system benchmark
echo "1/5: Memory system performance test"
caxton benchmark memory \
  --entities 50000 \
  --relations 150000 \
  --search-queries 5000 \
  --duration 300s \
  --output "$RESULTS_FILE" \
  --section memory

# 2. Configuration agent load test
echo "2/5: Configuration agent load test"
caxton benchmark agents \
  --config-agents 50 \
  --conversations-per-second 500 \
  --duration 600s \
  --response-timeout 30s \
  --output "$RESULTS_FILE" \
  --section agents \
  --append

# 3. API performance test
echo "3/5: REST API performance test"
caxton benchmark api \
  --concurrent-connections 100 \
  --requests-per-second 1000 \
  --duration 300s \
  --endpoints "/api/v1/agents,/api/v1/health,/api/v1/agents/*/status" \
  --output "$RESULTS_FILE" \
  --section api \
  --append

# 4. Embedding generation benchmark
echo "4/5: Embedding generation performance test"
caxton benchmark embeddings \
  --batch-sizes 1,16,32,64,128 \
  --total-embeddings 10000 \
  --parallel-threads 4 \
  --warmup-iterations 100 \
  --output "$RESULTS_FILE" \
  --section embeddings \
  --append

# 5. Full system stress test
echo "5/5: Full system stress test"
caxton benchmark system \
  --config-agents 100 \
  --memory-entities 200000 \
  --concurrent-conversations 1000 \
  --duration 1800s \
  --find-breaking-points \
  --output "$RESULTS_FILE" \
  --section stress \
  --append

echo "Benchmark complete. Results saved to: $RESULTS_FILE"

# Generate summary report
caxton benchmark report --input "$RESULTS_FILE" --format markdown \
  > "$BENCHMARK_DIR/summary-$DATE.md"

# Check for performance regressions
caxton benchmark compare \
  --current "$RESULTS_FILE" \
  --baseline "$BENCHMARK_DIR/baseline.json" \
  --threshold 0.1 \
  --output "$BENCHMARK_DIR/regression-$DATE.txt"
```

### Continuous Performance Testing

**Performance CI/CD integration:**

```yaml
# .github/workflows/performance.yml
name: Performance Regression Testing

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  performance-test:
    runs-on: ubuntu-latest
    timeout-minutes: 60

    steps:
    - uses: actions/checkout@v4

    - name: Setup Rust
      uses: dtolnay/rust-toolchain@stable

    - name: Build Caxton
      run: cargo build --release

    - name: Setup Performance Environment
      run: |
        # Prepare isolated environment for testing
        sudo sysctl -w vm.swappiness=1
        sudo sysctl -w vm.dirty_ratio=5
        echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor

    - name: Run Performance Benchmarks
      run: |
        # Start Caxton server
        cargo run --release &
        SERVER_PID=$!
        sleep 10

        # Run benchmark suite
        cargo run --bin benchmark -- \
          --suite production \
          --duration 300s \
          --output results.json

        # Stop server
        kill $SERVER_PID

    - name: Performance Regression Analysis
      run: |
        # Compare with baseline
        cargo run --bin benchmark-compare -- \
          --current results.json \
          --baseline .github/baselines/performance-baseline.json \
          --threshold 0.05 \
          --output regression-report.md

    - name: Upload Results
      uses: actions/upload-artifact@v4
      with:
        name: performance-results
        path: |
          results.json
          regression-report.md

    - name: Comment PR with Results
      if: github.event_name == 'pull_request'
      uses: actions/github-script@v7
      with:
        script: |
          const fs = require('fs');
          const report = fs.readFileSync('regression-report.md', 'utf8');
          github.rest.issues.createComment({
            issue_number: context.issue.number,
            owner: context.repo.owner,
            repo: context.repo.repo,
            body: `## Performance Test Results\n\n${report}`
          });
```

## Troubleshooting Performance Issues

### Common Performance Problems

#### Issue: High Memory Search Latency

**Symptoms:**

- Memory search P99 > 100ms
- Low embedding cache hit rate (< 80%)
- SQLite query slowdowns

**Diagnostic commands:**

```bash
# Analyze memory search patterns
caxton memory search-analysis --detailed --last 1h

# Check embedding cache efficiency
caxton memory embedding-stats --cache-analysis

# Analyze SQLite query performance
caxton storage query-analysis --slow-queries --threshold 50ms

# Check for memory fragmentation
caxton memory fragmentation-check
```

**Resolution steps:**

1. **Increase embedding cache size and TTL**
2. **Pre-compute embeddings for frequent queries**
3. **Optimize SQLite indexes for vector operations**
4. **Reduce search result limits**
5. **Consider external vector database migration**

#### Issue: Agent Response Slowdowns

**Symptoms:**

- Agent response P99 > 2s
- Tool call timeouts increasing
- Configuration validation slowdowns

**Diagnostic commands:**

```bash
# Analyze agent performance breakdown
caxton agents performance-breakdown --detailed --agent-type all

# Check tool call patterns and timeouts
caxton tools performance-analysis --show-slowest 10

# Validate configuration parsing performance
caxton agents validation-perf --show-slow-configs

# Check memory entity limits per agent
caxton agents memory-usage --show-heavy-users
```

**Resolution steps:**

1. **Cache TOML validation results**
2. **Optimize agent configuration complexity**
3. **Tune tool call timeouts per tool type**
4. **Implement lazy loading for agent tools**
5. **Reduce memory entity limits per agent**

#### Issue: SQLite Performance Degradation

**Symptoms:**

- Database queries P99 > 50ms
- WAL file growing large (> 100MB)
- Database size growing rapidly

**Diagnostic commands:**

```bash
# Analyze SQLite performance
caxton storage performance-analysis --include-wal

# Check database fragmentation
caxton storage fragmentation-check

# Analyze query patterns
caxton storage query-patterns --frequent-queries 20

# Check index utilization
caxton storage index-analysis --show-unused
```

**Resolution steps:**

1. **Run VACUUM to defragment database**
2. **Add indexes for frequent query patterns**
3. **Tune WAL checkpoint frequency**
4. **Increase SQLite cache size**
5. **Consider database partitioning strategies**

### Performance Optimization Strategies

#### 1. Cache Optimization Strategy

```yaml
cache_optimization:
  # Tiered caching approach
  l1_cache:
    type: "memory"
    size: "128MB"
    ttl: "5m"
    hit_target: 0.95

  l2_cache:
    type: "sqlite"
    size: "512MB"
    ttl: "1h"
    hit_target: 0.85

  # Smart cache warming
  cache_warming:
    enabled: true
    warmup_queries:
      - "common agent capabilities"
      - "frequent memory searches"
      - "popular embeddings"
    warmup_schedule: "startup,hourly"

  # Cache invalidation
  invalidation:
    strategy: "lru_with_ttl"
    max_age: "2h"
    memory_pressure_threshold: 0.85
```

#### 2. Resource Allocation Optimization

```yaml
resource_optimization:
  # CPU allocation
  cpu:
    http_workers: 4                    # 50% of cores for HTTP
    agent_workers: 4                   # 50% of cores for agents
    memory_workers: 2                  # Dedicated memory workers
    background_workers: 1              # Background tasks

  # Memory allocation
  memory:
    sqlite_cache: "256MB"              # 40% for SQLite
    embedding_model: "200MB"           # 30% for model
    agent_runtime: "128MB"             # 20% for agents
    system_overhead: "64MB"            # 10% system overhead

  # I/O optimization
  io:
    async_io_threads: 8                # Async I/O threads
    disk_read_ahead: "1MB"             # Read-ahead buffer
    write_buffer: "4MB"                # Write buffer
```

## Best Practices Summary

### Performance Excellence Guidelines

1. **Monitoring First**: Establish comprehensive performance monitoring before optimization
2. **Baseline Everything**: Record performance baselines for all key metrics
3. **Measure Before Optimizing**: Profile to identify actual bottlenecks
4. **Optimize in Order**: Address highest-impact issues first
5. **Cache Strategically**: Cache at appropriate layers with proper TTLs
6. **Scale Proactively**: Plan migrations before hitting embedded limits
7. **Test Continuously**: Automated performance testing in CI/CD
8. **Document Everything**: Record optimization decisions and their impacts

### Operational Performance Practices

1. **Daily Health Checks**: Automated daily performance health reports
2. **Weekly Trend Analysis**: Review performance trends and capacity planning
3. **Monthly Optimization**: Regular optimization reviews and tuning
4. **Quarterly Benchmarking**: Full benchmark suite and regression analysis
5. **Emergency Procedures**: Performance-related emergency response plans
6. **Capacity Planning**: Proactive planning for growth and scaling
7. **Team Training**: Regular training on performance tools and techniques
8. **Documentation Updates**: Keep performance guides current with changes

## References

- [Operations Runbook](runbook.md)
- [Lifecycle Management](lifecycle.md)
- [Security Operations](security.md)
- [Troubleshooting Guide](troubleshooting.md)
- [ADR-0030: Embedded Memory System](../../adrs/0030-embedded-memory-system.md)
- [ADR-0028: Configuration-Driven Agents](../../adrs/0028-configuration-driven-agent-architecture.md)
- [Prometheus Performance Monitoring](https://prometheus.io/docs/)
- [SQLite Performance Tuning](https://www.sqlite.org/optoverview.html)
