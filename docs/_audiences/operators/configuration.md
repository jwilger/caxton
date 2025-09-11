---
title: "Configuration Guide"
date: 2025-09-10
layout: page
audience: operators
navigation_order: 2
categories: [Operators, Configuration]
---

## Complete reference for configuring Caxton server and production deployment

This guide covers **server configuration** for production deployment,
including security, monitoring, scaling, and operational considerations.
For agent configuration, see the
[Agent Configuration Reference](../../config-agents/agent-format.md).

## Server Configuration Overview

Caxton uses YAML configuration files for server settings. The default
location is `/etc/caxton/config.yaml` for production or
`~/.config/caxton/config.yaml` for development.

### Configuration Hierarchy

Configuration is loaded in order of precedence:

1. **Command line flags** (highest priority)
2. **Environment variables** (CAXTON_*)
3. **Configuration file** (YAML)
4. **Default values** (lowest priority)

## Production Server Configuration

### Minimal Production Configuration

```yaml
# Minimal production setup with embedded memory
server:
  host: 0.0.0.0
  port: 8080
  dashboard_enabled: false  # Disable dashboard in production

# Embedded memory system (production-ready)
memory:
  backend: embedded
  embedded:
    database_path: "/var/lib/caxton/memory.db"
    max_entities: 500000

# Production logging
observability:
  logging:
    level: info
    format: json
    file: "/var/log/caxton/caxton.log"
```

### Complete Production Configuration

```yaml
# Server configuration
server:
  host: 0.0.0.0              # Bind to all interfaces
  port: 8080                  # REST API port
  metrics_port: 9090          # Prometheus metrics port
  dashboard_enabled: false    # Disable dashboard in production
  request_timeout: 30s        # Global request timeout
  shutdown_timeout: 30s       # Graceful shutdown timeout

  # TLS configuration (production requirement)
  tls:
    enabled: true
    cert_path: /etc/caxton/tls/cert.pem
    key_path: /etc/caxton/tls/key.pem
    ca_path: /etc/caxton/tls/ca.pem
    min_version: "1.2"
    cipher_suites:
      - "TLS_AES_256_GCM_SHA384"
      - "TLS_CHACHA20_POLY1305_SHA256"

  # CORS configuration
  cors:
    enabled: true
    allowed_origins:
      - "https://your-domain.com"
      - "https://admin.your-domain.com"
    allowed_methods: ["GET", "POST", "PUT", "DELETE"]
    allowed_headers: ["Authorization", "Content-Type"]
    max_age: 3600

# Configuration agent runtime
runtime:
  max_agents: 5000            # Maximum concurrent config agents
  agent_timeout: 30s          # Default message handling timeout
  enable_hot_reload: true     # Allow agent config hot-reloading
  conversation_cleanup: 24h   # Clean up old conversations
  max_memory_per_agent: 1GB   # Memory limit per agent
  cpu_limit_per_agent: 100m   # CPU limit per agent

# LLM Provider Configuration
llm:
  provider: "anthropic"       # openai|anthropic|azure|local|custom
  default_model: "claude-3-sonnet-20240229"  # Production model
  fallback_model: "claude-3-haiku-20240307"  # Backup model
  timeout: 30s                # LLM request timeout
  retry_attempts: 3           # Retry failed requests

  # Anthropic Configuration (Production)
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-3-sonnet-20240229"
    timeout: 30s
    max_tokens: 4096

  # OpenAI Configuration (Alternative)
  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    model: "gpt-4o"
    timeout: 30s
    max_tokens: 4096
    organization: "${OPENAI_ORG_ID}"

# Memory system configuration
memory:
  backend: embedded           # embedded|neo4j|qdrant

  # Embedded backend settings (Production)
  embedded:
    database_path: "/var/lib/caxton/memory.db"
    embedding_model: "all-MiniLM-L6-v2"
    max_entities: 500000      # Production scaling limit
    cleanup_interval: 1h      # Memory cleanup frequency
    semantic_threshold: 0.6   # Minimum similarity for semantic search
    backup_interval: 24h      # Automatic backup frequency
    vacuum_interval: 7d       # SQLite optimization frequency
    page_size: 4096          # SQLite page size optimization
    cache_size: -2000000     # SQLite cache size (2GB)

# Agent messaging configuration
messaging:
  max_message_size: 10MB      # Maximum message size
  queue_size: 50000           # Message queue size
  delivery_timeout: 10s       # Message delivery timeout
  conversation_ttl: 7d        # Conversation lifetime
  enable_message_persistence: true  # Store message history
  message_retention: 30d      # How long to keep messages

  capability_routing:
    strategy: "best_match"    # best_match|load_balance|broadcast
    timeout: 10s              # Capability resolution timeout
    health_check_interval: 30s # Agent health check frequency
    load_balancing_algorithm: "round_robin"  # round_robin|least_connections|weighted

# Capability registry
capabilities:
  discovery_interval: 30s     # How often to refresh capability registry
  health_check_interval: 60s # How often to check agent health
  auto_cleanup: true          # Remove unhealthy agents from registry
  max_capabilities_per_agent: 20  # Limit capabilities per agent

# Security configuration
security:
  # Authentication
  authentication:
    enabled: true
    method: "jwt"             # jwt|api_key|oauth2
    jwt_secret: "${JWT_SECRET}"
    jwt_expiry: 24h
    refresh_token_expiry: 7d

  # Authorization
  authorization:
    enabled: true
    default_role: "user"
    roles_config: "/etc/caxton/roles.yaml"

  # Rate limiting
  rate_limiting:
    enabled: true
    requests_per_minute: 1000
    burst: 50
    window: 1m
    exclude_paths: ["/api/v1/health", "/metrics"]

  # Input validation
  validation:
    max_payload_size: 10MB
    sanitize_inputs: true
    validate_schemas: true

# Tool integration (MCP servers in WASM sandbox)
tools:
  sandbox_memory_limit: 100MB # Memory limit for tool execution
  sandbox_cpu_limit: 100m     # CPU limit for tool execution
  sandbox_timeout: 30s        # Tool execution timeout
  max_concurrent_tools: 50    # Maximum concurrent tool executions

  # Whitelist of allowed tools (security)
  allowed_tools:
    - http_client
    - file_storage
    - database_connection
    - email_service
    - calendar_integration
    - notification_service

  # Tool-specific configuration
  tool_configs:
    http_client:
      timeout: 30s
      max_redirects: 5
      allowed_domains: ["api.example.com", "data.company.com"]
    database_connection:
      max_connections: 10
      connection_timeout: 10s
      query_timeout: 30s

# Observability (Production)
observability:
  # Logging
  logging:
    level: info               # trace|debug|info|warn|error
    format: json              # json|text (use json for production)
    file: "/var/log/caxton/caxton.log"
    max_size: 100MB           # Log file rotation
    max_files: 10             # Keep 10 rotated files
    compress: true            # Compress rotated files

    # Structured logging fields
    structured_fields:
      service: "caxton"
      environment: "production"
      version: "1.0.0"

  # Metrics (Prometheus)
  metrics:
    enabled: true
    prometheus_endpoint: "/metrics"
    custom_metrics: true      # Track agent-specific metrics
    histogram_buckets: [0.1, 0.25, 0.5, 1, 2.5, 5, 10]

    # Metric labels
    default_labels:
      service: "caxton"
      environment: "production"

  # Distributed tracing
  tracing:
    enabled: true
    service_name: "caxton"
    sample_rate: 0.1          # Sample 10% of traces

    # Jaeger configuration
    jaeger:
      endpoint: "http://jaeger:14268/api/traces"
      batch_size: 100
      flush_interval: 5s

    # OTLP configuration (alternative)
    otlp:
      endpoint: "http://otel-collector:4317"
      headers:
        "api-key": "${OTLP_API_KEY}"

# Clustering (coordination-first architecture)
cluster:
  enabled: true               # Enable clustering
  node_name: "caxton-1"       # Unique node identifier
  bind_port: 7946             # SWIM protocol port
  advertise_addr: "10.0.1.10" # Address other nodes use to reach this node

  # Seed nodes for cluster formation
  seeds:
    - "caxton-1:7946"
    - "caxton-2:7946"
    - "caxton-3:7946"

  # SWIM protocol configuration
  gossip_interval: 1s         # Gossip protocol interval
  probe_interval: 5s          # Node health probe interval
  probe_timeout: 3s           # Probe timeout
  suspect_timeout: 10s        # Time before marking node as suspect

  # Coordination settings
  coordination:
    leader_election: false    # No leader needed (coordination-first)
    state_sync_interval: 30s  # Sync agent state between nodes
    conflict_resolution: "timestamp"  # timestamp|node_priority

# Performance tuning
performance:
  # Connection pooling
  connection_pool:
    max_idle_connections: 100
    max_open_connections: 1000
    connection_max_lifetime: 1h
    connection_max_idle_time: 30m

  # Memory management
  memory:
    gc_target_percentage: 100  # Go GC target
    max_heap_size: 4GB        # Maximum heap size

  # Concurrency
  concurrency:
    max_workers: 100          # Maximum worker goroutines
    queue_size: 10000         # Work queue size
    batch_size: 50            # Batch processing size

# Data retention
retention:
  conversations: 30d          # Keep conversations for 30 days
  messages: 30d               # Keep messages for 30 days
  metrics: 15d                # Keep metrics for 15 days
  logs: 7d                    # Keep logs for 7 days
  memory_entities: 1y         # Keep memory entities for 1 year

  # Cleanup schedules
  cleanup_schedule:
    conversations: "0 2 * * *"  # Daily at 2 AM
    metrics: "0 3 * * 0"        # Weekly on Sunday at 3 AM
    logs: "0 4 * * *"           # Daily at 4 AM
```

## Environment-Specific Configurations

### Development Environment

```yaml
# development.yaml
server:
  host: localhost
  port: 8080
  dashboard_enabled: true     # Enable dashboard for development

runtime:
  llm_provider: "anthropic"
  llm_model: "claude-3-haiku"  # Faster, cheaper model for dev
  enable_hot_reload: true

memory:
  backend: embedded
  embedded:
    database_path: "./dev.db"
    max_entities: 10000       # Smaller limit for development

observability:
  logging:
    level: debug              # Verbose logging for development
    format: text              # Human-readable logs
    file: "./caxton.log"
  tracing:
    enabled: false            # Disable tracing in dev

security:
  authentication:
    enabled: false            # Disable auth in development
  rate_limiting:
    enabled: false            # Disable rate limiting in dev
```

### Staging Environment

```yaml
# staging.yaml
server:
  host: 0.0.0.0
  port: 8080
  dashboard_enabled: true     # Enable dashboard for testing

runtime:
  llm_provider: "anthropic"
  llm_model: "claude-3-sonnet"  # Production model for realistic testing

memory:
  backend: embedded
  embedded:
    database_path: "/var/lib/caxton-staging/memory.db"
    max_entities: 100000      # Realistic dataset size

observability:
  logging:
    level: info
    format: json
    file: "/var/log/caxton-staging/caxton.log"
  tracing:
    enabled: true             # Test tracing in staging

security:
  authentication:
    enabled: true             # Test auth in staging
  rate_limiting:
    enabled: true
    requests_per_minute: 500  # More permissive than production
```

### Production Environment

```yaml
# production.yaml
server:
  host: 0.0.0.0
  port: 8080
  dashboard_enabled: false    # Disable dashboard in production
  tls:
    enabled: true
    cert_path: /etc/caxton/tls/cert.pem
    key_path: /etc/caxton/tls/key.pem

runtime:
  max_agents: 5000
  llm_provider: "anthropic"
  llm_model: "claude-3-sonnet"

memory:
  backend: embedded           # or external backend for clustering
  embedded:
    database_path: "/var/lib/caxton/memory.db"
    max_entities: 500000

cluster:
  enabled: true
  seeds: ["node1:7946", "node2:7946", "node3:7946"]

observability:
  logging:
    level: info
    format: json
    file: "/var/log/caxton/caxton.log"
  metrics:
    enabled: true
  tracing:
    enabled: true
    jaeger_endpoint: "http://jaeger:14268/api/traces"

security:
  authentication:
    enabled: true
  authorization:
    enabled: true
  rate_limiting:
    enabled: true
    requests_per_minute: 1000
```

## External Memory Backend Configuration

For enterprise deployments requiring shared memory across clusters:

### Neo4j Backend

```yaml
memory:
  backend: neo4j
  neo4j:
    uri: "bolt://neo4j-cluster.internal:7687"
    username: "caxton"
    password: "${NEO4J_PASSWORD}"
    database: "caxton"

    # Connection pool settings
    pool_size: 50
    max_connection_lifetime: 1h
    max_connection_pooling_time: 30s

    # Performance settings
    timeout: 30s
    max_retry_time: 30s
    initial_retry_delay: 1s
    multiplier: 2.0
    jitter: 0.2

    # TLS settings
    tls:
      enabled: true
      ca_cert_path: "/etc/caxton/neo4j-ca.pem"
      verify_server_cert: true
```

### Qdrant Backend

```yaml
memory:
  backend: qdrant
  qdrant:
    host: "qdrant-cluster.internal"
    port: 6333
    collection_name: "caxton_memory"
    vector_size: 384

    # Authentication
    api_key: "${QDRANT_API_KEY}"

    # Connection settings
    timeout: 30s
    max_retries: 3
    retry_delay: 1s

    # Performance settings
    batch_size: 100
    parallel_writes: 4

    # TLS settings
    tls:
      enabled: true
      ca_cert_path: "/etc/caxton/qdrant-ca.pem"
```

## Kubernetes Configuration

### ConfigMap Example

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
  namespace: caxton
data:
  config.yaml: |
    server:
      host: 0.0.0.0
      port: 8080
      metrics_port: 9090

    memory:
      backend: embedded
      embedded:
        database_path: "/data/caxton.db"
        max_entities: 500000

    observability:
      logging:
        level: info
        format: json
      metrics:
        enabled: true
      tracing:
        enabled: true
        otlp:
          endpoint: "http://otel-collector.monitoring.svc.cluster.local:4317"

    cluster:
      enabled: true
      seeds:
        - "caxton-0.caxton.caxton.svc.cluster.local:7946"
        - "caxton-1.caxton.caxton.svc.cluster.local:7946"
        - "caxton-2.caxton.caxton.svc.cluster.local:7946"
```

### Secret Management

```yaml
apiVersion: v1
kind: Secret
metadata:
  name: caxton-secrets
  namespace: caxton
type: Opaque
data:
  anthropic-api-key: <base64-encoded-key>
  jwt-secret: <base64-encoded-secret>
  neo4j-password: <base64-encoded-password>
```

## Configuration Validation

### Server Configuration Validation

```bash
# Validate configuration syntax
caxton config validate /etc/caxton/config.yaml

# Test configuration without starting server
caxton config test --dry-run --config /etc/caxton/config.yaml

# Show effective configuration (with environment variables and defaults)
caxton config show --effective --config /etc/caxton/config.yaml

# Validate specific sections
caxton config validate --section memory /etc/caxton/config.yaml
caxton config validate --section security /etc/caxton/config.yaml
```

### Configuration Linting

```bash
# Check for security issues
caxton config lint --security /etc/caxton/config.yaml

# Check for performance issues
caxton config lint --performance /etc/caxton/config.yaml

# Check for production readiness
caxton config lint --production /etc/caxton/config.yaml
```

## Configuration Templates

### Generate Configuration Templates

```bash
# Generate production config template
caxton config init --template production --output /etc/caxton/config.yaml

# Generate development config template
caxton config init --template development --output ./dev-config.yaml

# Generate staging config template
caxton config init --template staging --output ./staging-config.yaml

# Generate minimal config template
caxton config init --template minimal --output ./minimal-config.yaml
```

### Custom Templates

```bash
# Create custom template
caxton config template create --name custom-production \
  --base production \
  --override memory.backend=qdrant \
  --override cluster.enabled=true

# Use custom template
caxton config init --template custom-production --output config.yaml
```

## Security Configuration

### TLS Certificate Management

```bash
# Generate self-signed certificate for testing
caxton tls generate-cert \
  --host localhost,127.0.0.1,caxton.example.com \
  --output-dir /etc/caxton/tls

# Use Let's Encrypt certificates
certbot certonly --standalone -d caxton.example.com
ln -s /etc/letsencrypt/live/caxton.example.com/fullchain.pem /etc/caxton/tls/cert.pem
ln -s /etc/letsencrypt/live/caxton.example.com/privkey.pem /etc/caxton/tls/key.pem
```

### JWT Configuration

```bash
# Generate secure JWT secret
JWT_SECRET=$(openssl rand -base64 32)
echo "JWT_SECRET=${JWT_SECRET}" >> /etc/caxton/environment

# Configure JWT in YAML
echo "
security:
  authentication:
    jwt_secret: \"\${JWT_SECRET}\"
" >> /etc/caxton/config.yaml
```

### Role-Based Access Control

Create `/etc/caxton/roles.yaml`:

```yaml
roles:
  admin:
    permissions:
      - "agents:*"
      - "config:*"
      - "system:*"
      - "metrics:*"

  operator:
    permissions:
      - "agents:read"
      - "agents:deploy"
      - "agents:update"
      - "system:health"
      - "metrics:read"

  user:
    permissions:
      - "agents:read"
      - "messages:send"
      - "conversations:read"

  readonly:
    permissions:
      - "agents:read"
      - "system:health"
      - "metrics:read"

# User assignments
users:
  admin@company.com:
    roles: ["admin"]
  ops@company.com:
    roles: ["operator"]
  dev@company.com:
    roles: ["user"]
```

## Environment Variables

### Standard Environment Variables

```bash
# Core configuration
export CAXTON_CONFIG="/etc/caxton/config.yaml"
export CAXTON_LOG_LEVEL="info"
export CAXTON_HOST="0.0.0.0"
export CAXTON_PORT="8080"

# Security
export CAXTON_TLS_ENABLED="true"
export CAXTON_JWT_SECRET="your-secret-key"
export ANTHROPIC_API_KEY="your-anthropic-key"
export OPENAI_API_KEY="your-openai-key"

# Memory backends
export NEO4J_PASSWORD="your-neo4j-password"
export QDRANT_API_KEY="your-qdrant-key"

# Observability
export CAXTON_METRICS_ENABLED="true"
export CAXTON_TRACING_ENABLED="true"
export JAEGER_ENDPOINT="http://jaeger:14268/api/traces"
```

### Docker Environment File

Create `.env` file for Docker Compose:

```bash
# Core settings
CAXTON_CONFIG=/etc/caxton/config.yaml
CAXTON_LOG_LEVEL=info

# Security
ANTHROPIC_API_KEY=your-anthropic-key
JWT_SECRET=your-jwt-secret

# Memory
CAXTON_MEMORY_MAX_ENTITIES=500000

# Clustering
CAXTON_CLUSTER_ENABLED=true
CAXTON_NODE_NAME=caxton-1
```

## Performance Tuning

### Memory Optimization

```yaml
memory:
  embedded:
    # SQLite optimizations
    page_size: 4096           # Optimal page size
    cache_size: -2000000      # 2GB cache
    vacuum_interval: 7d       # Regular optimization

    # Embedding optimizations
    embedding_batch_size: 100
    embedding_cache_size: 10000

    # Cleanup optimizations
    cleanup_batch_size: 1000
    cleanup_interval: 1h
```

### Concurrency Settings

```yaml
performance:
  concurrency:
    max_workers: 200          # Based on CPU cores
    queue_size: 20000         # Based on memory
    batch_size: 100           # Optimize for throughput

  connection_pool:
    max_idle_connections: 200
    max_open_connections: 2000
    connection_max_lifetime: 2h
```

### Load Balancing

```yaml
messaging:
  capability_routing:
    load_balancing_algorithm: "least_connections"
    health_check_interval: 15s
    load_threshold: 0.8       # Route to agent when under 80% load

  # Circuit breaker
  circuit_breaker:
    enabled: true
    failure_threshold: 5
    recovery_timeout: 30s
    test_request_volume: 3
```

## Monitoring Configuration

### Prometheus Metrics

```yaml
observability:
  metrics:
    enabled: true
    prometheus_endpoint: "/metrics"

    # Custom histogram buckets for response times
    histogram_buckets: [0.1, 0.25, 0.5, 1, 2.5, 5, 10, 30]

    # Enable detailed metrics
    detailed_metrics:
      memory_usage: true
      agent_performance: true
      capability_routing: true
      conversation_analytics: true
```

### Log Configuration

```yaml
observability:
  logging:
    level: info
    format: json

    # Log rotation
    file: "/var/log/caxton/caxton.log"
    max_size: 100MB
    max_files: 10
    compress: true

    # Structured logging
    structured_fields:
      service: "caxton"
      environment: "production"
      version: "1.0.0"
      datacenter: "us-east-1"
```

## Backup and Recovery

### Embedded Memory Backup

```yaml
memory:
  embedded:
    backup_interval: 24h      # Daily automated backups
    backup_retention: 30d     # Keep 30 days of backups
    backup_path: "/backup/caxton"
    backup_compression: true

    # Point-in-time recovery
    wal_mode: true            # Enable WAL mode
    checkpoint_interval: 1h   # Frequent checkpoints
```

### Backup Script Integration

```bash
# Custom backup configuration
backup:
  enabled: true
  schedule: "0 2 * * *"       # Daily at 2 AM
  retention: 30d
  compression: gzip
  destinations:
    - type: s3
      bucket: "caxton-backups"
      region: "us-east-1"
    - type: local
      path: "/backup/caxton"
```

## Troubleshooting Configuration

### Common Configuration Issues

```bash
# Check configuration syntax
caxton config validate --strict

# Test specific configuration sections
caxton config test --section server
caxton config test --section memory
caxton config test --section security

# Validate environment variables
caxton config env-check

# Check file permissions
caxton config permissions-check
```

### Configuration Debugging

```bash
# Show resolved configuration
caxton config show --resolved

# Show configuration sources
caxton config show --sources

# Debug specific values
caxton config get server.port
caxton config get memory.backend
caxton config get security.authentication.enabled
```

### Performance Diagnostics

```bash
# Test memory backend performance
caxton memory benchmark --config /etc/caxton/config.yaml

# Test TLS configuration
caxton tls test --config /etc/caxton/config.yaml

# Test clustering setup
caxton cluster test --config /etc/caxton/config.yaml
```

## Migration and Upgrades

### Configuration Migration

```bash
# Migrate from v0.9 to v1.0 configuration
caxton config migrate --from v0.9 --to v1.0 --input old-config.yaml --output new-config.yaml

# Validate migrated configuration
caxton config validate new-config.yaml --version v1.0
```

### Scaling Memory Backend

```bash
# Export from embedded to external backend
caxton memory export --format json --output backup.json

# Update configuration for external backend
# Edit config.yaml to use neo4j or qdrant

# Import to new backend
caxton memory import --format json --input backup.json
```

This comprehensive configuration guide ensures your Caxton deployment is
production-ready with proper security, monitoring, and scalability
considerations.

## Next Steps

- **[Operational Runbook](../../operations/operational-runbook.md)** -
  Day-to-day operations procedures
- **[Performance Tuning](../../operations/performance-tuning.md)** -
  Advanced optimization techniques
- **[Security Guide](../../operations/devops-security-guide.md)** - Security
  hardening and best practices
- **[Agent Format Reference](../../config-agents/agent-format.md)** - Agent
  configuration specification
