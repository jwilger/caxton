---
title: "Configuration Guide"
date: 2025-09-10
layout: page
categories: [Getting Started]
---

## Complete reference for configuring Caxton server and configuration agents

This guide covers both **server configuration** (how Caxton runs) and
**agent configuration** (how to define configuration agents with TOML
configuration files).

## Server Configuration

Caxton uses YAML configuration files for server settings. The default
location is `/etc/caxton/config.yaml` or `~/.config/caxton/config.yaml`.

### Minimal Server Configuration

```yaml
# Minimal server setup with zero dependencies
server:
  host: 0.0.0.0
  port: 8080

# Embedded memory system (default)
memory:
  backend: embedded
```

This configuration provides:

- REST API on port 8080
- Embedded SQLite + local embeddings memory system
- Zero external dependencies

### Complete Server Configuration

```yaml
# Server configuration
server:
  host: 0.0.0.0 # Bind address
  port: 8080 # REST API port
  metrics_port: 9090 # Prometheus metrics port
  dashboard_enabled: true # Enable web dashboard
  cors:
    enabled: true
    allowed_origins: ["*"] # Configure for production
    allowed_methods: ["GET", "POST", "PUT", "DELETE"]
  tls:
    enabled: false
    cert_path: /etc/caxton/tls/cert.pem
    key_path: /etc/caxton/tls/key.pem
    ca_path: /etc/caxton/tls/ca.pem

# Configuration agent runtime
runtime:
  max_agents: 1000 # Maximum concurrent agents
  agent_timeout: 30s # Default message handling timeout
  enable_hot_reload: true # Allow agent config hot-reloading
  conversation_cleanup: 24h # Clean up old conversations

# LLM Provider Configuration (Pluggable System)
llm:
  provider: "openai" # openai|anthropic|azure|local|custom
  default_model: "gpt-4o" # Default model for agent orchestration

  # OpenAI Configuration (default reference implementation)
  openai:
    api_key: "${OPENAI_API_KEY}"
    base_url: "https://api.openai.com/v1"
    model: "gpt-4o"
    timeout: 30s

  # Anthropic Configuration
  anthropic:
    api_key: "${ANTHROPIC_API_KEY}"
    model: "claude-3-haiku-20240307"
    timeout: 30s

  # Azure OpenAI Configuration
  azure:
    api_key: "${AZURE_OPENAI_API_KEY}"
    endpoint: "${AZURE_OPENAI_ENDPOINT}"
    deployment_name: "gpt-4"
    api_version: "2024-02-15-preview"

  # Local Model Configuration
  local:
    endpoint: "http://localhost:11434" # Ollama/vLLM/etc
    model: "llama3:8b"
    timeout: 60s

# Memory system configuration
memory:
  backend: embedded # embedded|neo4j|qdrant

  # Embedded backend settings (SQLite + Candle)
  embedded:
    database_path: "./caxton.db"
    embedding_model: "all-MiniLM-L6-v2" # Local embedding model
    max_entities: 100000 # Scaling limit for embedded backend
    cleanup_interval: 1h # Memory cleanup frequency
    semantic_threshold: 0.6 # Minimum similarity for semantic search

  # Optional external backends
  neo4j:
    uri: "bolt://localhost:7687"
    username: "neo4j"
    password: "password"
    database: "caxton"

  qdrant:
    host: "localhost"
    port: 6333
    collection_name: "caxton_memory"
    vector_size: 384

# Agent messaging configuration
messaging:
  max_message_size: 1MB # Maximum message size
  queue_size: 10000 # Message queue size
  delivery_timeout: 5s # Message delivery timeout
  conversation_ttl: 24h # Conversation lifetime
  enable_message_persistence: true # Store message history
  capability_routing:
    strategy: "best_match" # best_match|load_balance|broadcast
    timeout: 5s # Capability resolution timeout

# Capability registry
capabilities:
  discovery_interval: 30s # How often to refresh capability registry
  health_check_interval: 60s # How often to check agent health
  auto_cleanup: true # Remove unhealthy agents from registry

# Tool integration (MCP servers in WASM sandbox)
tools:
  sandbox_memory_limit: 100MB # Memory limit for tool execution
  sandbox_cpu_limit: 100m # CPU limit for tool execution
  sandbox_timeout: 30s # Tool execution timeout
  allowed_tools: # Whitelist of allowed tools
    - http_client
    - file_storage
    - database_connection
    - email_service

# Observability
observability:
  logging:
    level: info # trace|debug|info|warn|error
    format: json # json|text
    file: /var/log/caxton/caxton.log
  metrics:
    enabled: true
    prometheus_endpoint: /metrics
    custom_metrics: true # Track agent-specific metrics
  tracing:
    enabled: false
    jaeger_endpoint: "http://localhost:14268/api/traces"
    service_name: caxton
    sample_rate: 0.1

# Clustering (coordination-first architecture)
cluster:
  enabled: false
  node_name: "caxton-1"
  bind_port: 7946 # SWIM protocol port
  seeds: [] # Other cluster nodes
  gossip_interval: 1s
  probe_interval: 5s
  coordination:
    leader_election: false # No leader needed
    state_sync_interval: 30s # Sync agent state
```

## Configuration Agent Schema

Configuration agents are defined as TOML configuration files.
Here's the complete schema:

### Required Agent Fields

```toml
name = "string"                 # Unique agent identifier
version = "string"             # Semantic version (e.g., "1.0.0")
capabilities = ["string"]      # List of capabilities this agent provides
system_prompt = "string"       # Core behavior instructions
```

### Complete Agent Configuration

```toml
# Identity and Metadata
name = "AgentName"              # Unique identifier (no spaces)
version = "1.0.0"              # Semantic version
description = "Brief description of agent purpose"
author = "Your Name"           # Optional author info
tags = ["category", "type"]    # Optional tags for organization

# Capabilities and Behavior
capabilities = [               # What this agent can do
  "primary-capability",        # Use hyphenated names
  "secondary-capability",
  "tertiary-capability"
]

system_prompt = '''            # Core behavior definition
You are [AgentName], a [role] that [primary purpose].

Your responsibilities:
1. [First responsibility]
2. [Second responsibility]
3. [Third responsibility]

When processing requests:
- [Guideline 1]
- [Guideline 2]
- [Guideline 3]

Personality: [Personality traits]
'''

user_prompt_template = '''     # Template for user interactions
Request: {{request}}

Context: {{context}}
Memory: {{memory_context}}
User Info: {{user_info}}

Please help with this request.
'''

# Tools and Integration
tools = [                      # External services this agent can use
  "http_client",              # Web requests
  "database_connection",      # Database access
  "file_storage",            # File operations
  "email_service",           # Email capabilities
  "calendar_integration",    # Calendar access
  "notification_service",    # Push notifications
  "custom_tool_name"         # Your custom MCP tools
]

# Memory Configuration
[memory]
enabled = true                 # Enable persistent memory
scope = "workspace"            # agent|workspace|global
retention = "90d"              # How long to keep memories
semantic_search = true         # Enable vector search
relationship_tracking = true   # Track entity relationships
auto_cleanup = true            # Automatically clean old memories
learning_rate = "adaptive"     # How aggressively to learn patterns

# Conversation Management
[conversation]
max_turns = 50                 # Maximum conversation length
timeout = "5m"                 # Response timeout
context_window = 8192          # Token limit for context
temperature = 0.7              # LLM temperature setting
memory_integration = true      # Include memory in conversations

# Agent Parameters (Custom Configuration)
[parameters]
# Domain-specific settings
max_file_size = "10MB"
supported_formats = ["json", "yaml", "csv"]
default_priority = "medium"
time_zone = "UTC"
language = "en"

# Processing settings
batch_size = 100
retry_attempts = 3
cache_ttl = "1h"

# Security and Limits
[security]
restricted_tools = []          # Tools this agent cannot use
max_memory_usage = "50MB"      # Memory usage limit
max_processing_time = "30s"    # Processing time limit
allowed_domains = []           # HTTP domains agent can access

[security.sandbox_restrictions] # Additional sandbox restrictions
network_access = true
file_system_access = false

# Deployment Configuration
[deployment]
replicas = 1                   # Number of instances

[deployment.auto_scale]
enabled = false
min_replicas = 1
max_replicas = 5
cpu_threshold = 70

[deployment.health_check]
enabled = true
interval = "30s"
timeout = "5s"

[deployment.resource_limits]
memory = "100MB"
cpu = "100m"

# Monitoring and Metrics
[monitoring]
metrics_enabled = true         # Track agent-specific metrics
log_level = "info"            # trace|debug|info|warn|error
performance_tracking = true    # Track response times
conversation_analytics = true  # Analyze conversation patterns
custom_metrics = [             # Custom metrics to track
  "task_completion_rate",
  "user_satisfaction",
  "knowledge_growth"
]

documentation = '''
# Agent Documentation

The documentation section contains your agent documentation:

- Usage examples
- Feature descriptions
- API patterns
- Integration guides
'''
```

### Field Descriptions

#### Core Identity

- **name**: Unique identifier for the agent (used in capability routing)
- **version**: Semantic version for deployment management
- **capabilities**: List of what the agent can do (used for message routing)
- **system_prompt**: Instructions that define agent behavior and personality

#### Memory Configuration

```toml
[memory]
enabled = true              # Turn on persistent memory
scope = "workspace"         # Memory sharing level
retention = "90d"           # How long to keep memories
semantic_search = true      # Enable vector similarity search
relationship_tracking = true # Track entity relationships
```

**Memory Scopes**:

- `agent`: Private to this agent instance
- `workspace`: Shared within a project/team
- `global`: Shared across all agents (use carefully)

#### Tool Integration

```toml
tools = [
  "http_client",             # Make web requests
  "database_connection",     # Query databases
  "file_storage",           # Read/write files
  "email_service",          # Send emails
  "calendar_integration",   # Calendar operations
  "notification_service"    # Push notifications
]
```

Tools are MCP servers running in WebAssembly sandboxes for security.

#### Conversation Management

```toml
[conversation]
max_turns = 50            # Conversation length limit
timeout = "5m"            # Response timeout
context_window = 8192     # Token limit for context
temperature = 0.7         # LLM creativity setting
```

### Configuration Examples

#### Simple Single-Purpose Agent

```toml
name = "WeatherBot"
version = "1.0.0"
capabilities = ["weather-information"]
tools = ["http_client"]

system_prompt = '''
You are WeatherBot, a helpful weather information assistant.

When users ask about weather:
1. Use the HTTP client to fetch current weather data
2. Provide clear, accurate weather information
3. Include relevant warnings if severe weather is expected

Always be friendly and informative.
'''

documentation = '''
# WeatherBot

Get current weather information for any location worldwide.
'''
```

#### Multi-Capability Learning Agent

```toml
name = "CustomerSupport"
version = "2.1.0"
capabilities = ["customer-inquiry", "order-tracking", "technical-support", "escalation-management"]
tools = ["database_connection", "email_service", "notification_service"]

[memory]
enabled = true
scope = "global"
retention = "1y"
semantic_search = true
relationship_tracking = true

[parameters]
supported_languages = ["en", "es", "fr"]
escalation_threshold = "high"
response_time_target = "5m"

system_prompt = '''
You are a customer support specialist with access to order systems,
knowledge base, and escalation procedures.

For each capability:
- customer-inquiry: Answer general questions using knowledge base
- order-tracking: Look up order status in database
- technical-support: Troubleshoot using stored solutions
- escalation-management: Route complex issues to humans

Always check memory for similar issues and their resolutions.
Learn from each interaction to improve future responses.
'''
```

#### Workflow Orchestrator Agent

```toml
name = "DataPipeline"
version = "3.0.0"
capabilities = ["data-processing", "pipeline-orchestration"]
tools = ["database_connection", "file_storage", "notification_service"]

[memory]
enabled = true
scope = "workspace"
semantic_search = true

[parameters]
batch_size = 1000
retry_attempts = 3
notification_channels = ["email", "slack"]

system_prompt = '''
You orchestrate data processing pipelines by coordinating with other agents.

Pipeline workflow:
1. Receive data processing requests
2. Send data extraction tasks to "data-extraction" capability
3. Send transformation tasks to "data-transformation" capability
4. Send loading tasks to "data-loading" capability
5. Monitor progress and handle errors
6. Send completion notifications

Use capability-based messaging to coordinate the entire pipeline.
'''
```

## Environment Configuration

### Development Environment

```yaml
# development.yaml
server:
  host: localhost
  port: 8080

runtime:
  llm_provider: "anthropic"
  llm_model: "claude-3-haiku"
  enable_hot_reload: true

memory:
  backend: embedded
  embedded:
    database_path: "./dev.db"

observability:
  logging:
    level: debug
    format: text
```

### Production Environment

```yaml
# production.yaml
server:
  host: 0.0.0.0
  port: 8080
  tls:
    enabled: true
    cert_path: /etc/caxton/tls/cert.pem
    key_path: /etc/caxton/tls/key.pem

runtime:
  max_agents: 5000
  llm_provider: "anthropic"
  llm_model: "claude-3-sonnet"

memory:
  backend: qdrant
  qdrant:
    host: "qdrant.internal"
    port: 6333

cluster:
  enabled: true
  seeds: ["node1:7946", "node2:7946", "node3:7946"]

observability:
  logging:
    level: info
    format: json
    file: /var/log/caxton/caxton.log
  metrics:
    enabled: true
  tracing:
    enabled: true
    jaeger_endpoint: "http://jaeger:14268/api/traces"
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
data:
  config.yaml: |
    server:
      host: 0.0.0.0
      port: 8080
    memory:
      backend: embedded
      embedded:
        database_path: "/data/caxton.db"
    observability:
      logging:
        level: info
        format: json
```

## Configuration Validation

### Validate Server Configuration

```bash
# Check server config syntax
caxton config validate /etc/caxton/config.yaml

# Test server config without starting
caxton config test --dry-run

# Show effective configuration (with defaults)
caxton config show --effective
```

### Validate Agent Configuration

```bash
# Validate agent TOML syntax
caxton agent validate task-manager.toml

# Check capability names against registry
caxton agent validate task-manager.toml --check-capabilities

# Validate tool availability
caxton agent validate task-manager.toml --check-tools

# Full validation with schema checking
caxton agent validate task-manager.toml --strict
```

### Configuration Templates

Generate configuration templates:

```bash
# Generate server config template
caxton config init --template production

# Generate agent config template
caxton agent init --template multi-capability

# Generate minimal agent template
caxton agent init --template simple
```

## Configuration Best Practices

### 1. Use Environment-Specific Configs

```bash
# Development
caxton server start --config development.yaml

# Production
caxton server start --config production.yaml

# Override specific values
caxton server start --config production.yaml --override server.port=9080
```

### 2. Secure Sensitive Values

```yaml
# Use environment variables for secrets
memory:
  neo4j:
    password: "${NEO4J_PASSWORD}"

# Or use external secret management
memory:
  neo4j:
    password_file: "/run/secrets/neo4j_password"
```

### 3. Version Your Agent Configs

```toml
name = "CustomerSupport"
version = "2.1.0"          # Increment for each change
description = "Added Spanish language support"
```

### 4. Use Meaningful Capability Names

```toml
# Good: Specific, actionable capabilities
capabilities = [
  "customer-inquiry",
  "order-tracking",
  "technical-support"
]

# Avoid: Generic or vague capabilities
capabilities = [
  "general-help",
  "assistant"
]
```

### 5. Design for Memory Efficiency

```toml
[memory]
enabled = true
scope = "workspace"        # Don't use global unless needed
retention = "90d"         # Match your data retention policies
semantic_search = true    # Only if you need vector search
```

### 6. Configure Appropriate Timeouts

```toml
[conversation]
timeout = "5m"           # Balance user experience vs resource usage
max_turns = 20           # Prevent runaway conversations

[runtime]
agent_timeout = "30s"      # Quick enough for real-time use
```

### 7. Monitor and Tune Performance

```yaml
observability:
  metrics:
    enabled: true
    custom_metrics: true # Track agent-specific performance

monitoring:
  performance_tracking: true
  conversation_analytics: true
```

## Troubleshooting Configuration

### Common Server Config Issues

```bash
# Check for port conflicts
netstat -lan | grep 8080

# Validate YAML syntax
caxton config validate --strict

# Check file permissions
ls -la /etc/caxton/config.yaml

# Test memory backend connectivity
caxton memory test --config production.yaml
```

### Common Agent Config Issues

```bash
# Check TOML syntax
caxton agent validate agent.toml --toml-only

# Verify capability names are valid
caxton capability list --available

# Check tool availability
caxton tools status

# Test agent deployment
caxton agent deploy agent.toml --dry-run
```

### Performance Tuning

```bash
# Monitor memory usage
caxton memory stats --live

# Check agent response times
caxton metrics --agents --response-times

# Analyze conversation patterns
caxton conversations analyze --performance

# Profile agent execution
caxton profile --agent AgentName --duration 5m
```

## Migration Guide

### Upgrading From WASM Agents

If migrating from WebAssembly agents:

1. **Extract core logic** into system prompts
2. **Convert compiled behavior** into configuration
3. **Replace host functions** with tool integrations
4. **Migrate state management** to memory system

```toml
# Before: WASM agent with host functions
# After: Config agent with tools
name = "DataProcessor"
capabilities = ["data-processing"]
tools = [
  "database_connection",  # Replaces database host function
  "file_storage"        # Replaces file I/O host function
]

[memory]
enabled = true         # Replaces manual state management
```

### Scaling Up Memory Backend

When embedded memory reaches limits:

```bash
# Export current data
caxton memory export --format json --output backup.json

# Update config for external backend
# Edit config.yaml to use neo4j or qdrant

# Restart with new backend
caxton server restart

# Import existing data
caxton memory import --format json --input backup.json
```

This configuration system supports Caxton's evolution from simple agents
to sophisticated multi-agent systems while maintaining zero-dependency
simplicity for getting started.
