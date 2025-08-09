# Configuration Guide

Complete reference for configuring Caxton server and agents.

## Configuration File

Caxton uses YAML configuration files. The default location is `/etc/caxton/config.yaml`.

### Minimal Configuration

```yaml
server:
  host: 0.0.0.0
  port: 8080
```

### Complete Configuration

```yaml
# Server configuration
server:
  host: 0.0.0.0              # Bind address
  port: 8080                  # API port
  metrics_port: 9090          # Prometheus metrics port
  grpc_port: 50051           # gRPC port (optional)
  tls:
    enabled: false
    cert_path: /etc/caxton/tls/cert.pem
    key_path: /etc/caxton/tls/key.pem
    ca_path: /etc/caxton/tls/ca.pem

# Agent runtime configuration
runtime:
  max_agents: 1000            # Maximum concurrent agents
  max_memory_per_agent: 100MB  # Memory limit per agent
  max_cpu_per_agent: 100m      # CPU limit (millicores)
  default_agent_timeout: 30s   # Default message handling timeout
  agent_pool_size: 10         # Pre-warmed agent instances
  enable_hot_reload: true     # Allow agent hot-reloading

# Message routing configuration
messaging:
  max_message_size: 1MB       # Maximum message size
  queue_size: 10000          # Message queue size
  delivery_timeout: 5s       # Message delivery timeout
  retry_attempts: 3          # Delivery retry attempts
  retry_delay: 1s           # Delay between retries
  enable_persistence: true   # Persist messages to disk
  persistence_path: /var/lib/caxton/messages

# Coordination configuration
coordination:
  # Local state storage (per instance)
  local_state:
    type: sqlite            # Always SQLite for local state
    path: /var/lib/caxton/local.db
    journal_mode: WAL

  # Cluster coordination
  cluster:
    enabled: true           # Enable multi-instance coordination
    bind_addr: 0.0.0.0:7946  # SWIM protocol binding
    advertise_addr: auto    # Address advertised to other nodes
    seeds:                  # Initial cluster members
      - caxton-1.example.com:7946
      - caxton-2.example.com:7946
    gossip_interval: 200ms  # Gossip protocol interval
    probe_interval: 1s      # Failure detection interval

  # Partition handling
  partition:
    detection_timeout: 5s   # Time before declaring partition
    quorum_size: 2          # Minimum nodes for write operations
    degraded_mode: true     # Enable degraded mode in minority partition
    queue_writes: true      # Queue writes during partition
    max_queue_size: 10000   # Maximum queued messages

# Observability configuration
observability:
  # Logging
  logging:
    level: info             # Log level (trace, debug, info, warn, error)
    format: json           # Log format (json, pretty, compact)
    output: stdout         # Output (stdout, stderr, file)
    file_path: /var/log/caxton/caxton.log
    max_file_size: 100MB
    max_backups: 10
    max_age: 30d

  # Metrics
  metrics:
    enabled: true
    export_interval: 10s
    histogram_buckets: [0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1, 5, 10]

  # Tracing
  tracing:
    enabled: true
    backend: otlp          # Tracing backend (otlp, jaeger, zipkin)
    endpoint: http://localhost:4317
    sample_rate: 0.1      # Sample 10% of traces
    export_timeout: 10s

# Security configuration
security:
  authentication:
    enabled: false
    type: token           # Authentication type (token, oauth2, mtls)
    token:
      header_name: X-Caxton-Token
      tokens_file: /etc/caxton/tokens.yaml
    oauth2:
      provider: github
      client_id: ${OAUTH_CLIENT_ID}
      client_secret: ${OAUTH_CLIENT_SECRET}
      redirect_url: http://localhost:8080/auth/callback

  authorization:
    enabled: false
    type: rbac           # Authorization type (rbac, abac)
    policy_file: /etc/caxton/policies.yaml

  rate_limiting:
    enabled: true
    requests_per_second: 100
    burst_size: 200

  cors:
    enabled: true
    allowed_origins: ["*"]
    allowed_methods: ["GET", "POST", "PUT", "DELETE"]
    allowed_headers: ["*"]
    max_age: 3600

# Agent deployment configuration
deployment:
  strategies:
    default: direct      # Default deployment strategy
    canary:
      initial_percentage: 10
      increment: 20
      interval: 5m
      error_threshold: 5%
    blue_green:
      warm_up_time: 30s
      switch_delay: 10s

  health_checks:
    enabled: true
    initial_delay: 5s
    interval: 30s
    timeout: 10s
    failure_threshold: 3
    success_threshold: 1

# External integrations
integrations:
  # Model Context Protocol (MCP) for tool access
  mcp:
    enabled: true
    servers:
      - name: web_tools
        url: http://localhost:3000
        capabilities: ["web_search", "web_fetch"]
      - name: database_tools
        url: http://localhost:3001
        capabilities: ["sql_query", "data_export"]

  # Webhook notifications
  webhooks:
    enabled: false
    endpoints:
      - url: https://example.com/webhook
        events: ["agent.deployed", "agent.failed"]
        retry_policy:
          max_attempts: 3
          backoff: exponential

# Resource management
resources:
  # CPU management
  cpu:
    reserve_system: 20%    # Reserve for system
    overcommit_ratio: 1.5  # Allow 150% allocation

  # Memory management
  memory:
    reserve_system: 500MB
    swap_enabled: false
    oom_kill_disable: false

  # Disk management
  disk:
    data_dir: /var/lib/caxton
    temp_dir: /tmp/caxton
    max_temp_size: 10GB
    cleanup_interval: 1h

# Experimental features
experimental:
  enable_gpu_support: false
  enable_distributed_mode: false
  enable_auto_scaling: false
```

## Environment Variables

Caxton supports environment variable substitution in config files:

```yaml
database:
  password: ${DB_PASSWORD}
  host: ${DB_HOST:-localhost}  # With default value
```

### Standard Environment Variables

```bash
# Configuration
CAXTON_CONFIG_PATH=/etc/caxton/config.yaml
CAXTON_ENV=production  # Environment (development, staging, production)

# Server
CAXTON_HOST=0.0.0.0
CAXTON_PORT=8080
CAXTON_METRICS_PORT=9090

# Logging
RUST_LOG=info  # Log level
RUST_BACKTRACE=1  # Enable backtraces

# Database
CAXTON_DB_HOST=localhost
CAXTON_DB_PORT=5432
CAXTON_DB_NAME=caxton
CAXTON_DB_USER=caxton
CAXTON_DB_PASSWORD=secret

# Observability
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
OTEL_SERVICE_NAME=caxton
OTEL_RESOURCE_ATTRIBUTES=environment=production

# Security
CAXTON_AUTH_ENABLED=true
CAXTON_TLS_ENABLED=true
```

## CLI Configuration

### Global Options

```bash
# Specify config file
caxton --config /path/to/config.yaml server start

# Override specific values
caxton --set server.port=9000 server start

# Use environment
caxton --env production server start
```

### Agent-Specific Configuration

When deploying agents, you can provide configuration:

```yaml
# agent-config.yaml
name: my-agent
resources:
  memory: 50MB
  cpu: 50m
  timeout: 10s

environment:
  - name: LOG_LEVEL
    value: debug
  - name: API_KEY
    valueFrom:
      secretRef: api-key-secret

capabilities:
  - web_search
  - database_access

metadata:
  version: "1.0.0"
  author: "developer@example.com"
  description: "Process automation agent"
```

Deploy with configuration:

```bash
caxton deploy agent.wasm --config agent-config.yaml
```

## Configuration Profiles

Use profiles for different environments:

### Development Profile

```yaml
# config-dev.yaml
extends: base.yaml

server:
  host: localhost
  port: 8080

runtime:
  max_agents: 10
  enable_hot_reload: true

observability:
  logging:
    level: debug
    format: pretty

security:
  authentication:
    enabled: false
```

### Production Profile

```yaml
# config-prod.yaml
extends: base.yaml

server:
  tls:
    enabled: true

runtime:
  max_agents: 1000
  enable_hot_reload: false

observability:
  logging:
    level: info
    format: json
  tracing:
    enabled: true
    sample_rate: 0.01

security:
  authentication:
    enabled: true
  rate_limiting:
    enabled: true
```

Use profiles:

```bash
# Development
caxton --profile dev server start

# Production
caxton --profile prod server start
```

## Dynamic Configuration

Some configuration can be changed at runtime:

```bash
# Update log level
caxton config set observability.logging.level=debug

# Update rate limits
caxton config set security.rate_limiting.requests_per_second=200

# View current configuration
caxton config get

# View specific section
caxton config get runtime
```

## Configuration Validation

Validate configuration before starting:

```bash
# Validate configuration file
caxton config validate --file config.yaml

# Test configuration
caxton server start --dry-run
```

## Secret Management

### Using External Secrets

```yaml
# Reference Kubernetes secrets
database:
  password:
    valueFrom:
      secretKeyRef:
        name: db-secret
        key: password

# Reference HashiCorp Vault
api_keys:
  openai:
    valueFrom:
      vaultKeyRef:
        path: secret/data/api-keys
        key: openai

# Reference AWS Secrets Manager
credentials:
  aws:
    valueFrom:
      awsSecretRef:
        name: caxton/prod/aws
        region: us-east-1
```

### Local Secrets File

```yaml
# secrets.yaml (chmod 600)
tokens:
  admin: "secret-admin-token"
  service: "secret-service-token"

api_keys:
  openai: "sk-..."
  anthropic: "sk-ant-..."
```

Reference in config:

```yaml
security:
  authentication:
    tokens_file: /etc/caxton/secrets.yaml
```

## Performance Tuning

### High Throughput Configuration

```yaml
runtime:
  max_agents: 5000
  agent_pool_size: 100

messaging:
  queue_size: 100000
  delivery_timeout: 1s
  enable_persistence: false  # Disable for speed

storage:
  type: memory  # Use in-memory storage

resources:
  cpu:
    overcommit_ratio: 2.0
```

### Low Latency Configuration

```yaml
runtime:
  agent_pool_size: 50  # Pre-warm more agents
  default_agent_timeout: 5s

messaging:
  max_message_size: 100KB
  queue_size: 1000
  delivery_timeout: 500ms
  retry_attempts: 1

observability:
  metrics:
    export_interval: 30s  # Reduce overhead
  tracing:
    enabled: false  # Disable for lowest latency
```

### Resource Constrained Configuration

```yaml
runtime:
  max_agents: 100
  max_memory_per_agent: 10MB
  agent_pool_size: 5

messaging:
  queue_size: 1000
  enable_persistence: true

storage:
  type: sqlite

resources:
  memory:
    swap_enabled: true
```

## Troubleshooting Configuration

### Common Issues

1. **Port Already in Use**
   ```yaml
   server:
     port: 8081  # Change to available port
   ```

2. **Database Connection Failed**
   ```yaml
   storage:
     postgres:
       connection_timeout: 30s  # Increase timeout
       max_connections: 10      # Reduce connections
   ```

3. **High Memory Usage**
   ```yaml
   runtime:
     max_memory_per_agent: 50MB  # Reduce per-agent memory
     agent_pool_size: 5           # Reduce pool size
   ```

4. **Slow Message Delivery**
   ```yaml
   messaging:
     queue_size: 50000          # Increase queue size
     delivery_timeout: 10s      # Increase timeout
   ```

### Debug Configuration

Enable detailed debugging:

```yaml
observability:
  logging:
    level: trace
    format: pretty

  tracing:
    enabled: true
    sample_rate: 1.0  # Sample all traces

experimental:
  enable_debug_endpoints: true
```

## Configuration Best Practices

1. **Use Environment Variables** for sensitive data
2. **Separate Profiles** for different environments
3. **Version Control** configuration files (except secrets)
4. **Validate** configuration before deployment
5. **Monitor** configuration changes
6. **Document** custom configuration
7. **Backup** configuration regularly
8. **Test** configuration changes in staging first

## Next Steps

- [CLI Reference](../user-guide/cli-reference.md) - Complete CLI documentation
- [Production Deployment](../operations/production-deployment.md) - Production configuration
- [Monitoring Guide](../user-guide/monitoring.md) - Observability configuration
- [Security Guide](../operations/devops-security-guide.md) - Security configuration
