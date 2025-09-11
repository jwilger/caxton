---
title: "ADR-0008: Agent Deployment"
date: 2025-08-03
status: superseded
superseded_by: "ADR-0028: Configuration-Driven Agent Architecture"
layout: adr
categories: [Architecture]
---


## Status

**Superseded** by [ADR-0028: Configuration-Driven Agent Architecture](0028-configuration-driven-agent-architecture.md)

This ADR defined complex deployment strategies for compiled WASM modules.
ADR-0028 shifts the primary user experience to configuration-driven agents
(markdown files), which require a completely different deployment model based
on configuration files rather than binary modules.

## Context

With Caxton as an application server (ADR-0006), we need a deployment model that
enables continuous delivery of agents while maintaining system stability. The
model must support modern deployment practices like canary releases, blue-green
deployments, and instant rollbacks.

Traditional application servers often require restarts for deployments, causing
downtime and limiting iteration speed. In a multi-agent system, we need to
deploy, update, and remove agents without affecting other running agents or
dropping messages.

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

Instant rollback capability with state preservation:

- **Traffic diversion**: Immediately stop routing messages to problematic
  version
- **Message draining**: Complete processing of in-flight messages with timeout
- **Previous version restoration**: Restore message routing to last known good
  version
- **Resource cleanup**: Clean up failed deployment resources and state

Rollback operations complete in seconds, minimizing impact of problematic
deployments.

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

## User Experience Principles

**Simple Development Workflow**:

- Single-command deployment for development scenarios
- Fast validation and deployment feedback (sub-second for simple agents)
- Clear success/failure indicators with actionable error messages

**Production Safety**:

- Progressive rollout strategies with automatic monitoring
- Real-time health checks and performance metrics
- Automatic rollback triggers based on configurable thresholds

**Emergency Response**:

- Instant rollback capability for critical issues
- Clear status reporting throughout rollback process
- Minimal downtime (target: under 3 seconds for rollback completion)

## Observability

Every deployment provides comprehensive observability:

**Metrics Collection**:

- Deployment duration and success rates by strategy type
- Validation timing for each pipeline stage
- Rollback frequency and triggering conditions
- Active agent versions and deployment status

**Distributed Tracing**:

- End-to-end deployment operation tracing
- Validation pipeline stage instrumentation
- Canary rollout phase tracking with timing data
- Rollback operation trace correlation

**Structured Logging**:

- Deployment lifecycle events with full context
- Stage transitions with timing and health metrics
- Error conditions with actionable diagnostic information
- Audit trail for compliance and debugging

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
