---
title: "0008. Agent Deployment Model"
date: 2025-08-03
status: proposed
layout: adr
categories: [Architecture]
---

# 0008. Agent Deployment Model

Date: 2025-08-03

## Status

Proposed

## Context

With Caxton as an application server (ADR-0006), we need a deployment model that enables continuous delivery of agents while maintaining system stability. The model must support modern deployment practices like canary releases, blue-green deployments, and instant rollbacks.

Traditional application servers often require restarts for deployments, causing downtime and limiting iteration speed. In a multi-agent system, we need to deploy, update, and remove agents without affecting other running agents or dropping messages.

## Decision Drivers

- **Zero downtime**: Agent updates must not disrupt running agents
- **Fast iteration**: Deploy changes in seconds, not minutes
- **Safety**: Validate agents before they process production messages
- **Rollback capability**: Instant reversion to previous versions
- **Testing in production**: Shadow deployments and canary releases
- **Resource isolation**: One agent's failure shouldn't affect others

## Decision

We will implement a hot-reload deployment model with multi-stage validation:

### 1. Deployment API

```protobuf
service AgentDeployment {
  // Direct deployment (development/testing)
  rpc DeployAgent(DeployRequest) returns (DeployResponse);

  // Production deployment strategies
  rpc CreateCanaryDeployment(CanaryRequest) returns (CanaryResponse);
  rpc PromoteCanary(PromoteRequest) returns (PromoteResponse);
  rpc RollbackDeployment(RollbackRequest) returns (RollbackResponse);

  // Shadow deployment for testing
  rpc CreateShadowDeployment(ShadowRequest) returns (ShadowResponse);
}
```

### 2. Validation Pipeline

Every deployment goes through mandatory validation:

```rust
async fn validate_agent(wasm_module: &[u8]) -> Result<ValidationReport> {
    // Stage 1: Static validation
    let static_report = validate_wasm_module(wasm_module)?;

    // Stage 2: Sandbox execution
    let sandbox_report = run_in_sandbox(wasm_module).await?;

    // Stage 3: Contract verification
    let contract_report = verify_message_contracts(wasm_module).await?;

    // Stage 4: Resource profiling
    let resource_report = profile_resource_usage(wasm_module).await?;

    Ok(ValidationReport {
        static: static_report,
        sandbox: sandbox_report,
        contracts: contract_report,
        resources: resource_report,
    })
}
```

### 3. Deployment Strategies

**Blue-Green Deployment**:
```yaml
deployment:
  strategy: blue-green
  stages:
    - validate: 30s
    - warm_up: 60s
    - switch: instant
    - cleanup: 300s
```

**Canary Deployment**:
```yaml
deployment:
  strategy: canary
  stages:
    - validate: 30s
    - canary: 5%    # 5 minutes
    - canary: 25%   # 10 minutes
    - canary: 50%   # 10 minutes
    - full: 100%
  rollback_on:
    - error_rate > 1%
    - p99_latency > 100ms
```

**Shadow Deployment**:
```yaml
deployment:
  strategy: shadow
  duration: 1h
  compare:
    - output_equivalence
    - performance_metrics
    - resource_usage
```

### 4. State Management

**Deployment States**:
```rust
enum DeploymentState {
    Validating,      // Running validation pipeline
    Deploying,       // Active deployment in progress
    Running,         // Stable and processing messages
    Draining,        // Preparing for removal
    Failed,          // Deployment failed validation
    RolledBack,      // Reverted to previous version
}
```

**Message Handling During Deployment**:
- New agents start receiving messages only after warm-up
- Old agents continue processing during deployment
- Message handoff coordinated by router
- Zero message loss guaranteed

### 5. Rollback Mechanism

Instant rollback with state preservation:

```rust
async fn rollback_deployment(deployment_id: &str) -> Result<()> {
    // 1. Stop routing messages to new version
    router.disable_agent(deployment_id).await?;

    // 2. Drain in-flight messages (max 30s)
    drain_messages(deployment_id).await?;

    // 3. Restore previous version routing
    router.enable_previous_version(deployment_id).await?;

    // 4. Clean up failed deployment
    cleanup_deployment(deployment_id).await?;

    Ok(())
}
```

## Consequences

### Positive

- **Zero downtime**: Hot reload without service interruption
- **Fast iteration**: Deploy and test in seconds
- **Production safety**: Multi-stage validation catches issues early
- **Progressive rollout**: Reduce blast radius of bad deployments
- **A/B testing**: Built-in shadow deployment support
- **Instant rollback**: Revert in under 1 second
- **Resource isolation**: cgroup-based isolation per agent

### Negative

- **Memory overhead**: Running multiple versions temporarily
- **Complexity**: Sophisticated routing and state management
- **Validation time**: 30-60 second validation adds latency
- **State migration**: Complex for stateful agents

### Mitigation Strategies

**Memory Overhead**:
- Aggressive cleanup of old versions
- Resource limits during deployment
- Deployment queuing to prevent overload

**Complexity**:
- Comprehensive observability for deployments
- Clear state machine for deployment lifecycle
- Automated testing of deployment scenarios

**State Migration**:
- Event sourcing for stateful agents
- Backward-compatible message formats
- State versioning and migration tools

## Deployment Examples

### Simple Development Deployment
```bash
caxton deploy agent processor.wasm
✓ Validation passed (0.8s)
✓ Agent deployed (0.2s)
→ Agent ID: proc-v2-7f8d9
→ Receiving messages
```

### Production Canary Deployment
```bash
caxton deploy agent processor.wasm --strategy canary
✓ Validation passed (0.8s)
✓ Canary 5% started (0.2s)
→ Monitoring for 5 minutes...
  ✓ Error rate: 0.01% (threshold: 1%)
  ✓ P99 latency: 12ms (threshold: 100ms)
✓ Promoting to 25%...
```

### Emergency Rollback
```bash
caxton rollback proc-v2-7f8d9
✓ Messages diverted (0.1s)
✓ Draining in-flight (2.3s)
✓ Previous version restored (0.1s)
→ Rollback complete in 2.5s
```

## Observability

Every deployment emits:

**Metrics**:
```
caxton_deployment_duration_seconds{strategy="canary", status="success"}
caxton_deployment_validation_time_seconds{stage="sandbox"}
caxton_deployment_rollback_total{reason="high_error_rate"}
caxton_agent_version_active{agent="processor", version="v2"}
```

**Traces**:
```
deployment.create (25.3s)
├── validation.static (0.3s)
├── validation.sandbox (2.1s)
├── validation.contracts (0.5s)
├── canary.5_percent (300s)
├── canary.25_percent (600s)
└── deployment.finalize (0.2s)
```

**Structured Logs**:
```json
{
  "timestamp": "2024-11-20T10:30:00Z",
  "level": "info",
  "deployment_id": "dep-123",
  "agent_id": "processor",
  "version": "v2",
  "stage": "canary",
  "percentage": 25,
  "metrics": {
    "error_rate": 0.01,
    "p99_latency_ms": 12
  }
}
```

## Related Decisions

- ADR-0002: WebAssembly for Agent Isolation - Enables hot reload
- ADR-0006: Application Server Architecture - Requires deployment model
- ADR-0007: Management API Design - Deployment API specification
- ADR-0009: CLI Tool Design - User interface for deployments

## References

- [Continuous Delivery](https://continuousdelivery.com/) by Jez Humble
- [Progressive Delivery](https://launchdarkly.com/blog/what-is-progressive-delivery/)
- [Canary Deployments at Netflix](https://netflixtechblog.com/automated-canary-analysis-at-netflix-with-kayenta-3260bc7acc69)
- Kubernetes deployment strategies
