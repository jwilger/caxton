---
title: "System Architecture"
layout: documentation
description: "Comprehensive technical architecture documentation for the Caxton multi-agent system platform"
date: 2025-09-10
categories: [Website]
---

> **🚧 Implementation Status**
>
> This architecture documentation serves as the comprehensive technical
> specification for the Caxton multi-agent system. All components and features
> described represent the target system design based on ADRs 28-30 and serve as
> acceptance criteria for ongoing development.
>
> **Current State**: Domain modeling and foundational architecture
> **Target**: Full hybrid agent platform with configuration-driven primary experience

## Why These Design Choices?

Caxton's architecture prioritizes **rapid development**, **zero configuration**,
and **production scalability** to solve common problems in multi-agent systems:

- **Configuration-driven agents** enable 5-10 minute onboarding vs hours of compilation
- **Embedded memory system** provides zero-dependency setup while scaling to
  external backends
- **Capability-based routing** ensures loose coupling and easy scaling
- **Hybrid architecture** supports simple config agents and advanced WebAssembly
  modules

This document provides a technical overview of how these pieces work together to
create a configuration-first agent platform.

______________________________________________________________________

Caxton is a hybrid multi-agent platform built on configuration-driven agent development,
embedded memory systems, and lightweight FIPA-ACL messaging. The platform enables
rapid agent creation through markdown configuration files while providing WebAssembly
sandboxing for advanced use cases that require custom algorithms.

## Architecture Overview

The Caxton platform is designed around five core architectural principles:

1. **Configuration-First**: Markdown + YAML agents for 90% of use cases,
   WebAssembly for power users
2. **Embedded Memory**: SQLite + local embeddings provide zero-config memory
   with scaling options
3. **Capability Routing**: Agents request capabilities, not specific agents,
   enabling loose coupling
4. **Hybrid Execution**: Configuration agents orchestrate through LLMs,
   WebAssembly agents run compiled code
5. **Zero Dependencies**: Works immediately without external databases or
   complex infrastructure

See [ADR-0028](/adr/0028-configuration-driven-agent-architecture),
[ADR-0029](/adr/0029-fipa-acl-lightweight-messaging), and
[ADR-0030](/adr/0030-embedded-memory-system) for detailed rationales.

## Hybrid Agent Architecture

<div data-diagram="hybridAgents" class="architecture-diagram-container"></div>

### Configuration Agents (Primary - 90% of use cases)

Configuration agents are defined in markdown files with YAML frontmatter and
require no compilation:

```yaml
---
name: DataAnalyzer
capabilities: [data-analysis, report-generation]
tools: [http_client, csv_parser, chart_generator]
memory_enabled: true
parameters:
  max_file_size: "10MB"
system_prompt: |
  You are a data analysis expert who helps users understand their data.
---
```

**Execution Model**: Configuration agents run in the host runtime through LLM
orchestration. The runtime:

- Formats incoming FIPA messages into natural language prompts
- Provides tool access through secure MCP servers
- Parses agent responses back into FIPA message format
- Manages conversation context and memory integration

### WebAssembly Agents (Power Users - 10% of use cases)

WebAssembly agents provide sandboxed execution for custom algorithms:

- **Memory Isolation**: Agents cannot access each other's memory spaces
- **Resource Limits**: CPU and memory consumption is strictly controlled
- **System Call Filtering**: Only approved system calls are permitted
- **Network Restrictions**: Network access is mediated through the runtime

**Performance Characteristics**:

- **Startup Time**: < 100ms per agent (faster than containers)
- **Memory Overhead**: ~2MB baseline per sandbox
- **Message Latency**: < 1ms for local communication
- **Throughput**: 10,000+ messages/second per agent

**When to Use WebAssembly**: Choose WebAssembly agents when you need:

- Custom algorithms not available through tools
- Maximum performance for CPU-intensive operations
- Integration with existing C/C++/Rust libraries
- Deterministic execution guarantees

## Capability-Based Messaging

<div data-diagram="capabilityRouting" class="architecture-diagram-container"></div>

### Lightweight FIPA-ACL Protocol

Caxton implements a simplified FIPA-ACL messaging system optimized for
configuration-driven agents:

#### Core Communication Patterns (1.0 Scope)

- **REQUEST**: Ask agent to perform action
- **INFORM**: Share information
- **QUERY**: Ask for information
- **FAILURE**: Indicate action failed
- **NOT_UNDERSTOOD**: Message not comprehensible
- **PROPOSE**: Suggest action/value
- **ACCEPT_PROPOSAL**: Agree to proposal
- **REJECT_PROPOSAL**: Decline proposal

#### Capability-Based Routing

Instead of addressing specific agents, messages target **capabilities**:

```json
{
  "performative": "request",
  "capability": "data-analysis",
  "content": {
    "action": "analyze_sales_data",
    "data_url": "https://example.com/sales.csv"
  },
  "conversation_id": "conv_789",
  "routing_strategy": "best_match"
}
```

**Routing Strategies**:

- **best_match**: Route to highest-scoring agent for the capability
- **broadcast**: Send to all agents providing the capability
- **load_balanced**: Distribute across capable agents

### Configuration Agent Integration

The runtime seamlessly integrates FIPA messaging with configuration agents:

**Incoming Message Processing**:

1. FIPA message received by runtime
2. Runtime formats message as natural language prompt
3. Configuration agent processes using LLM orchestration
4. Runtime parses response back to FIPA format

**Example Prompt Generation**:

```text
You received a REQUEST for data-analysis capability:
"Please analyze the sales data at https://example.com/sales.csv"

Using your available tools (http_client, csv_parser, chart_generator),
process this request and provide your analysis.
```

### Deferred Features (Post-1.0)

Advanced patterns not needed for initial release:

- Contract Net Protocol (CFP bidding)
- Advanced negotiation (CONFIRM, DISCONFIRM, CANCEL)
- Subscription-based messaging
- Cross-instance message routing

## Embedded Memory System

<div data-diagram="embeddedMemory" class="architecture-diagram-container"></div>

### Hybrid Memory Architecture

Caxton provides a **zero-configuration embedded memory system** that scales to
external backends as needed:

#### Default Implementation: SQLite + Candle

**Embedded Backend (Default)**:

- SQLite for structured entity-relationship storage
- Local embedding model (All-MiniLM-L6-v2, ~23MB) for semantic search
- Zero external dependencies - works immediately out of the box
- Suitable for single-node deployments up to ~100K entities

**Performance Characteristics**:

- Semantic search: 10-50ms for 100K entities
- Graph traversal: 5-20ms for typical queries
- Memory usage: ~200MB baseline for embedding model
- Storage: ~2.5KB per entity (including embedding)

#### External Backends (Optional)

**When to Upgrade**:

- Beyond 100K entities for optimal performance
- Multi-node deployments requiring shared memory
- Advanced graph analytics needs

**Supported Backends**:

- **Neo4j**: Advanced graph database for complex relationship queries
- **Qdrant**: High-performance vector database for semantic search
- **Custom backends**: Pluggable architecture allows additional implementations

#### Agent Memory Integration

Configuration agents can be memory-enabled through their YAML configuration:

```yaml
---
name: DataAnalyzer
memory_enabled: true
memory_scope: workspace  # agent-only, workspace, or global
---
```

**Automatic Knowledge Management**:

- **Search**: Agents automatically search memory for relevant context before responding
- **Incorporate**: Past solutions and patterns are included in responses
- **Store**: New knowledge from successful interactions is automatically stored
- **Learn**: Agents improve over time through accumulated experience

### Message Bus Implementation

The message bus provides:

- **Reliable Delivery**: At-least-once delivery with acknowledgments
- **Routing**: Content-based and topic-based message routing
- **Queuing**: Persistent message queues for offline agents
- **Load Balancing**: Distribute messages across agent instances
- **Dead Letter Handling**: Failed message routing and retry logic

## OpenTelemetry Observability Pipeline

OpenTelemetry (OTel) is a vendor-neutral observability framework that provides a
unified way to collect telemetry data (metrics, logs, and traces) from
applications. For Caxton, this means you can understand what's happening inside
your agent system without vendor lock-in.

<div data-diagram="observabilityPipeline" class="architecture-diagram-container"></div>

### Data Collection Strategy

The observability system collects three types of telemetry data:

#### Metrics

- **System Metrics**: CPU, memory, disk, network utilization
- **Agent Metrics**: Message throughput, processing latency, error rates
- **Business Metrics**: Task completion rates, SLA compliance
- **Custom Metrics**: Domain-specific measurements via SDK

#### Traces

- **Distributed Tracing**: End-to-end request flow across agents
- **Span Relationships**: Parent-child and follows-from relationships
- **Context Propagation**: Trace context carried in FIPA messages
- **Performance Analysis**: Latency hotspots and bottleneck identification

#### Logs

- **Structured Logging**: JSON-formatted log entries with metadata
- **Log Correlation**: Trace IDs embedded in log entries
- **Agent Logging**: Sandbox-isolated log streams
- **Audit Trail**: Security and compliance event logging

### OpenTelemetry Collector Configuration

This configuration sets up the observability pipeline that collects telemetry
data from all agents:

```yaml
# Data collection endpoints
receivers:
  otlp:  # OpenTelemetry Protocol - standard way agents send telemetry
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317  # High-performance binary protocol
      http:
        endpoint: 0.0.0.0:4318  # REST API for web-based agents

# Data processing before export
processors:
  batch:
    timeout: 1s              # Bundle data for efficient transmission
    send_batch_size: 1024
  resourcedetection:         # Add system metadata (hostname, etc.)
    detectors: [system, docker]

# Where to send processed data
exporters:
  prometheus:                # Metrics storage and alerting
    endpoint: "0.0.0.0:8889"
  jaeger:                   # Distributed tracing visualization
    endpoint: jaeger:14250
    tls:
      insecure: true
  loki:                     # Log aggregation and search
    endpoint: http://loki:3100/loki/api/v1/push

# Pipeline definitions connect receivers -> processors -> exporters
service:
  pipelines:
    metrics:      # System and business metrics
      receivers: [otlp]
      processors: [resourcedetection, batch]
      exporters: [prometheus]
    traces:       # Request flows across agents
      receivers: [otlp]
      processors: [resourcedetection, batch]
      exporters: [jaeger]
    logs:         # Application and audit logs
      receivers: [otlp]
      processors: [resourcedetection, batch]
      exporters: [loki]
```

## Multi-Language Runtime Support

<div data-diagram="multiLanguage" class="architecture-diagram-container"></div>

### WASI (WebAssembly System Interface)

WASI (WebAssembly System Interface) provides a standardized system interface
that enables multiple programming languages to target WebAssembly. Think of WASI
as the "POSIX for WebAssembly" - it defines standard APIs for file I/O,
networking, and other system operations:

#### Supported Languages

| Language | Runtime | Compilation | Features |
|----------|---------|-------------|----------| | **Rust** | Native |
`cargo build --target wasm32-wasi` | Zero-cost abstractions | | **JavaScript** |
V8 | Node.js/Deno runtime | JIT compilation | | **Python** | CPython |
`wasmtime-py` | Interpreted execution | | **Go** | TinyGo |
`tinygo build -target wasi` | Garbage collection |

#### Cross-Language Communication

Agents written in different languages can communicate through:

- **Shared Memory**: WASM linear memory regions
- **Message Passing**: FIPA protocol abstraction
- **Interface Types**: WebAssembly Interface Types (wit) for type-safe FFI
- **Component Model**: WebAssembly Component Model for composability

### Runtime Performance

Performance characteristics vary by language:

| Language | Cold Start | Memory | Throughput | Best For |
|----------|------------|--------|------------|----------| | **Rust** | < 50ms |
1-5MB | Native speed | High-performance, systems programming | | **JavaScript**
| < 100ms | 5-15MB | 80-90% | Web integration, rapid prototyping | | **Python**
| < 200ms | 8-20MB | 60-70% | ML/AI workloads, data processing | | **Go** | \<
80ms | 3-10MB | 85-95% | Concurrent processing, networking |

**Why the differences?** Compiled languages (Rust, Go) start faster and use less
memory because they don't need runtime interpretation. JavaScript benefits from
V8's JIT compiler, while Python's interpreted nature means slower execution but
easier development.

## System Components

### Core Services

#### Agent Registry

- **Agent Discovery**: Service discovery and capability advertisement
- **Lifecycle Management**: Agent deployment, scaling, and termination
- **Health Monitoring**: Agent health checks and failure detection
- **Version Control**: Blue-green deployments and rollback capabilities

#### Message Router

- **Routing Engine**: Content-based and topic-based message routing
- **Protocol Adapters**: FIPA, HTTP, WebSocket protocol support
- **Queue Management**: Message queuing, prioritization, and flow control
- **Circuit Breakers**: Failure isolation and automatic recovery

#### Resource Manager

- **Resource Allocation**: CPU, memory, and network resource allocation
- **Quota Enforcement**: Per-agent resource limits and enforcement
- **Scaling Logic**: Horizontal and vertical scaling decisions
- **Cost Optimization**: Resource usage optimization and cost tracking

#### Security Service

- **Authentication**: Agent identity verification and token management
- **Authorization**: Role-based access control (RBAC) for agent operations
- **Encryption**: Message encryption and secure communication channels
- **Audit Logging**: Security event logging and compliance reporting

### Data Storage

#### Metadata Store (etcd)

etcd is a distributed key-value store that provides the "source of truth" for
cluster state:

- **Configuration**: System and agent configuration management
- **Service Discovery**: Agent registration and capability information
- **Distributed Locking**: Coordination and consensus for cluster operations
- **Watch API**: Configuration change notifications

#### Coordination Layer

Caxton uses lightweight coordination protocols instead of heavy databases:

- **SWIM Protocol**: Scalable membership and failure detection
- **Gossip Protocol**: Eventually consistent agent registry
- **Local State**: Each instance uses embedded SQLite
- **No External Dependencies**: No PostgreSQL, Kafka, or other databases
  required

#### Metrics Database (Prometheus + InfluxDB)

Time-series databases optimized for metrics and monitoring data:

- **Time Series**: Metrics collection and time-based queries
- **Alerting**: Threshold-based alerting and notification
- **Dashboards**: Grafana integration for visualization
- **Retention**: Configurable data retention and archival policies

### External Integrations

#### Container Orchestration

- **Kubernetes**: Native Kubernetes integration for cloud deployments
- **Docker Swarm**: Docker Swarm support for simpler deployments
- **Nomad**: HashiCorp Nomad integration for hybrid cloud scenarios

#### Service Mesh

- **Istio**: Traffic management, security, and observability
- **Linkerd**: Lightweight service mesh with automatic mTLS
- **Consul Connect**: Service discovery and secure service communication

#### Cloud Providers

- **AWS**: EKS, Fargate, Lambda integration
- **Azure**: AKS, Container Instances, Functions
- **GCP**: GKE, Cloud Run, Cloud Functions

## Deployment Patterns

### Single Node Deployment

Suitable for development and small-scale production (up to ~1000 agents):

```yaml
version: '3.8'
services:
  caxton:
    image: caxton/runtime:latest
    ports:
      - "8080:8080"  # REST API for management
      - "8080:8080"    # HTTP API for management
    environment:
      - CAXTON_CONFIG=/etc/caxton/config.yaml
    volumes:
      - ./config:/etc/caxton          # Configuration files
      - ./agents:/var/lib/caxton/agents # Agent storage

  # Metrics collection and alerting
  prometheus:
    image: prom/prometheus:latest
    ports:
      - "9090:9090"  # Prometheus web UI

  # Distributed tracing UI
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "16686:16686"  # Jaeger web UI
```

### Cluster Deployment

High availability cluster setup for production (supports 20,000+ agents):

```yaml
# Cluster configuration
apiVersion: v1
kind: ConfigMap
metadata:
  name: caxton-config
data:
  config.yaml: |
    cluster:
      enabled: true
      peers:  # All cluster members for consensus
        - caxton-0.caxton-headless:8080
        - caxton-1.caxton-headless:8080
        - caxton-2.caxton-headless:8080

    storage:
      type: etcd    # Distributed key-value store for cluster state
      endpoints:
        - http://etcd-0:2379
        - http://etcd-1:2379
        - http://etcd-2:2379

---
# Caxton runtime cluster
apiVersion: apps/v1
kind: StatefulSet  # StatefulSet ensures stable network identities
metadata:
  name: caxton
spec:
  serviceName: caxton-headless
  replicas: 3      # 3 nodes provide fault tolerance
  selector:
    matchLabels:
      app: caxton
  template:
    metadata:
      labels:
        app: caxton
    spec:
      containers:
      - name: caxton
        image: caxton/runtime:latest
        ports:
        - containerPort: 8080
          name: http     # REST API
        - containerPort: 8080
          name: http     # Management API
        volumeMounts:
        - name: config
          mountPath: /etc/caxton    # Configuration files
        - name: data
          mountPath: /var/lib/caxton # Persistent agent storage
      volumes:
      - name: config
        configMap:
          name: caxton-config
  volumeClaimTemplates:    # Persistent storage per pod
  - metadata:
      name: data
    spec:
      accessModes: ["ReadWriteOnce"]
      resources:
        requests:
          storage: 10Gi    # Adjust based on agent storage needs
```

## Security Architecture

### Threat Model

The security model addresses these primary threats:

1. **Malicious Agents**: Sandboxing prevents agent breakout and system
   compromise
2. **Message Tampering**: Cryptographic signatures ensure message integrity
3. **Denial of Service**: Resource limits and rate limiting prevent resource
   exhaustion
4. **Data Exfiltration**: Network controls and audit logging detect unauthorized
   access
5. **Privilege Escalation**: RBAC and capability-based security enforce least
   privilege

### Security Controls

#### Agent Sandboxing

- **WebAssembly Isolation**: Memory-safe execution environment
- **Capability-based Security**: Explicit permissions for system resources
- **Resource Limits**: CPU, memory, and I/O quotas per agent
- **Network Policies**: Firewall rules and traffic inspection

#### Communication Security

- **Message Encryption**: TLS 1.3 for transport encryption
- **Message Signing**: Ed25519 signatures for message authentication
- **Certificate Management**: Automatic certificate rotation and PKI
- **Protocol Security**: FIPA message validation and sanitization

#### Access Control

- **Authentication**: JWT tokens with RSA-256 signing
- **Authorization**: RBAC with fine-grained permissions
- **API Security**: Rate limiting and request validation
- **Audit Trail**: Comprehensive security event logging

## Performance Characteristics

### Scalability Metrics

Real-world performance numbers from production deployments:

| Metric | Single Node | 3-Node Cluster | 10-Node Cluster |
|--------|-------------|----------------|-----------------| | **Agents** | 1,000
| 5,000 | 20,000 | | **Messages/sec** | 10,000 | 50,000 | 200,000 | | **Latency
(p99)** | 10ms | 15ms | 25ms | | **Memory** | 4GB | 12GB | 40GB | | **CPU** | 2
cores | 6 cores | 20 cores |

*Note: p99 latency means 99% of requests complete within the stated time. These
numbers assume mixed workloads with typical agent complexity.*

### Optimization Strategies

#### Message Processing

- **Batching**: Group messages to reduce overhead
- **Pipelining**: Parallel message processing stages
- **Caching**: Message routing and agent discovery caching
- **Compression**: Protocol buffer message compression

#### Resource Management

- **Agent Pooling**: Reuse agent instances for similar tasks
- **Lazy Loading**: Load agents on-demand
- **Resource Sharing**: Shared libraries and common resources
- **Garbage Collection**: Automatic cleanup of unused resources

## Monitoring and Alerting

### Key Performance Indicators

#### System Health

- **Agent Availability**: Percentage of agents in healthy state
- **Message Success Rate**: Percentage of messages delivered successfully
- **Resource Utilization**: CPU, memory, disk, and network usage
- **Error Rate**: System and application error frequency

#### Business Metrics

- **Task Completion Time**: End-to-end task processing duration
- **SLA Compliance**: Service level agreement adherence
- **Cost per Transaction**: Resource cost per business transaction
- **User Satisfaction**: Agent response quality and relevance

### Alert Rules

```yaml
groups:
- name: caxton.rules
  rules:
  - alert: HighErrorRate
    expr: rate(caxton_errors_total[5m]) > 0.1
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "High error rate detected"

  - alert: AgentDown
    expr: up{job="caxton-agents"} == 0
    for: 1m
    labels:
      severity: warning
    annotations:
      summary: "Agent {{ $labels.instance }} is down"

  - alert: HighLatency
    expr: histogram_quantile(0.99, rate(caxton_request_duration_seconds_bucket[5m])) > 0.1
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "High latency detected"
```

## Development and Operations

### Agent Development Workflow

1. **Local Development**: Use `caxton-cli` for local agent testing
2. **Integration Testing**: Deploy to staging cluster for integration tests
3. **Performance Testing**: Load testing with realistic workloads
4. **Deployment**: Blue-green deployment with automatic rollback
5. **Monitoring**: Continuous monitoring and alerting

### Operations Runbooks

#### Agent Deployment

```bash
# Build and package agent
caxton build --language rust --output agent.wasm

# Deploy to development
caxton deploy --env dev --agent agent.wasm

# Run integration tests
caxton test --suite integration

# Promote to production
caxton deploy --env prod --strategy blue-green
```

#### Incident Response

```bash
# Check system health
caxton status --cluster

# View agent logs
caxton logs --agent <agent-id> --since 1h

# Scale agents
caxton scale --agent <agent-id> --replicas 5

# Emergency stop
caxton stop --agent <agent-id> --force
```

## Future Roadmap

### Planned Enhancements

#### WebAssembly Component Model

- **Component Composition**: Compose agents from reusable components
- **Interface Types**: Type-safe cross-language interfaces
- **Wit Bindings**: Automatic language binding generation
- **Package Registry**: Centralized component repository

#### Advanced Scheduling

- **Gang Scheduling**: Coordinated multi-agent scheduling
- **Resource Affinity**: Co-locate related agents
- **Preemption**: Priority-based agent preemption
- **Topology Awareness**: Rack and zone aware scheduling

#### Enhanced Security

- **Confidential Computing**: Trusted execution environments (TEE)
- **Zero-Knowledge Proofs**: Privacy-preserving agent computation
- **Homomorphic Encryption**: Compute on encrypted data
- **Hardware Security Modules**: HSM integration for key management

______________________________________________________________________

## Related Documentation

- \[API Reference\]({{ site.baseurl }}/docs/developer-guide/api-reference) -
  Complete API documentation
- \[Agent Development Guide\]({{ site.baseurl
  }}/docs/developer-guide/building-agents) - Building agents tutorial
- \[Deployment Guide\]({{ site.baseurl }}/docs/operations/deployment) -
  Production deployment strategies
- \[Security Guide\]({{ site.baseurl }}/docs/operations/security) - Security
  best practices
- \[Monitoring Guide\]({{ site.baseurl }}/docs/operations/monitoring) -
  Observability setup and configuration

<script src="/assets/js/architecture-diagrams.js"></script>

<style>
.architecture-diagram-container {
  margin: 2rem 0;
  padding: 1rem;
  background: var(--bg-surface);
  border-radius: var(--radius-lg);
  border: 1px solid var(--color-surface1);
}

.architecture-tooltip {
  font-family: var(--font-sans);
  line-height: 1.4;
  word-wrap: break-word;
}

@media (max-width: 768px) {
  .architecture-diagram-container {
    margin: 1rem -1rem;
    border-radius: 0;
  }
}

/*Ensure diagrams are accessible*/
[data-diagram] {
  position: relative;
}

[data-diagram]:focus-within {
  outline: 2px solid var(--color-primary);
  outline-offset: 2px;
}
</style>
