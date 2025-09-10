---
title: "0010. External Agent Routing API"
date: 2025-08-03
status: proposed
layout: adr
categories: [Architecture, Technology]
deciders: [Platform Systems Architect, UX Research Expert, Async Rust Expert, Observability Expert]
---

Date: 2025-01-03 Status: Proposed Deciders: Platform Systems Architect, UX
Research Expert, Async Rust Expert, Observability Expert

## Context and Problem Statement

Caxton needs to support "externally routable agents" - allowing external API
clients to invoke agents asynchronously. This requires designing:

1. API patterns for agent invocation
2. Concurrency model for multiple requests to the same agent
3. Async behavior handling for long-running tasks
4. Security standards for external access
5. Observability for production debugging

The solution must align with Caxton's application server architecture and "3 AM
debugging" philosophy.

## Decision Drivers

- **Production Debugging**: API responses must contain enough context for
  independent troubleshooting
- **Developer Experience**: API should match developer mental models for service
  invocation
- **Performance**: Target < 1ms overhead for local API calls
- **Security**: Support industry standards from development to production
- **Observability**: Full request lifecycle visibility with correlation IDs
- **Concurrency**: Handle multiple clients calling the same agent safely
- **WebAssembly Integration**: Work within WASM execution constraints

## Considered Options

### API Design Options

### Option A: Single REST Endpoint

- `/api/v1/route/{agent_id}` with POST payload
- Simple but implies infrastructure routing
- Misaligns with developer mental models

### Option B: Resource-Oriented REST

- `/api/v1/agents/{agent_id}/invoke` for invocation
- `/api/v1/jobs/{job_id}` for async job tracking
- Matches developer expectations for service calls

### Option C: GraphQL

- Single endpoint with query flexibility
- Complex for simple agent invocation use cases
- Adds unnecessary complexity for core use case

### Option D: Dual Protocol

- gRPC primary for performance (ExternalAgentRouter service)
- REST gateway for accessibility and ecosystem compatibility
- Best of both worlds approach

### Concurrency Models

### Option A: Shared Agent Instances

- Single agent instance handles multiple requests
- Complex state management and potential conflicts
- Poor fault isolation

### Option B: Request-Per-Instance

- New agent instance for each request
- High resource overhead and slow startup
- Simple but wasteful

### Option C: Actor-Per-Agent

- Dedicated tokio task per agent with message queues
- Natural back-pressure through bounded channels
- Good fault isolation and resource management

### Security Approaches

### Option A: API Keys Only

- Simple for development
- Insufficient for production requirements
- No fine-grained access control

### Option B: OAuth2/JWT

- Industry standard for external APIs
- Complex setup for simple use cases
- Good ecosystem compatibility

### Option C: Progressive Security

- API keys for development (`cax_dev_{random}_{clientname}`)
- mTLS + RBAC for production
- Matches user journey from dev to production

## Decision Outcome

### Chosen: Dual Protocol + Actor-Per-Agent + Progressive Security

### API Design

- **Primary**: gRPC `ExternalAgentRouter` service with methods:
  - `InvokeSync()` - synchronous agent calls
  - `InvokeAsync()` - asynchronous with job tracking
  - `StreamInvoke()` - streaming responses
- **Secondary**: REST gateway at `/api/v1/agents/{agent_id}/invoke`
- **Job Tracking**: `/api/v1/jobs/{job_id}` for async status/results

### Concurrency Model

- **Actor-per-agent** with dedicated tokio tasks
- **Bounded MPSC channels** for natural back-pressure
- **Hierarchical cancellation** using tokio::CancellationToken
- **Multi-layer protection**: global semaphore, rate limiting, circuit breakers

### Async Behavior

- **Job lifecycle**: `SUBMITTED → QUEUED → ASSIGNED → RUNNING → COMPLETED`
- **Progress tracking** with estimated completion times
- **Configurable TTL** for job results storage
- **WebAssembly cooperative scheduling** using fuel-based yield points

### Security Model

- **Development**: API keys with structure `cax_dev_{random}_{clientname}`
- **Production**: mTLS client certificates with RBAC authorization
- **Rate limiting** with standard HTTP headers (X-RateLimit-\*)
- **CORS support** for browser-based clients

### Observability Strategy

- **OpenTelemetry spans** covering full request lifecycle
- **Structured error responses** following What/Why/How/Debug pattern
- **Correlation IDs** in all logs and traces
- **Canonical log line** per request with complete debugging context
- **Four golden signals metrics** with high-cardinality dimensions:
  - Latency (p50, p90, p99, p99.9)
  - Traffic (requests/sec by agent, route, user)
  - Errors (rate by type, agent, cause)
  - Saturation (queue depth, CPU, memory)

## Consequences

### Positive

- **Self-debugging API**: Responses contain enough context for independent
  troubleshooting
- **Production ready**: Comprehensive observability and error handling
- **Performance optimized**: gRPC primary path with \<1ms overhead target
- **Developer friendly**: REST gateway for ecosystem compatibility
- **Scalable**: Actor model handles concurrency naturally
- **Secure**: Progressive security matches deployment patterns

### Negative

- **Complexity**: Dual protocol increases implementation complexity
- **Resource usage**: Actor-per-agent model uses more memory
- **Learning curve**: gRPC may be unfamiliar to some developers
- **Configuration**: Security model requires proper operational setup

### Risks and Mitigations

- **Risk**: WebAssembly execution blocking event loop
  - **Mitigation**: Fuel-based cooperative scheduling with yield points
- **Risk**: Resource exhaustion under load
  - **Mitigation**: Multi-layer protection with bounded queues and circuit
    breakers
- **Risk**: Security misconfiguration
  - **Mitigation**: Secure defaults and clear configuration documentation

## Implementation Notes

### Phase 1: Core External Routing

- gRPC service definition and server implementation
- Actor-per-agent concurrency model
- Basic job tracking and lifecycle management
- API key authentication for development

### Phase 2: Production Features

- REST gateway via grpc-gateway
- mTLS and RBAC authorization
- Advanced observability and debugging APIs
- Performance optimizations and benchmarking

### Phase 3: Advanced Patterns

- Streaming invocation patterns
- Batch job processing
- Advanced rate limiting and quotas
- Integration with cloud provider auth systems

## Links

- [ADR-0006: Application Server Architecture](0006-application-server-architecture.md)
- [ADR-0007: Management API Design](0007-management-api-design.md)
- [ADR-0001: Observability First Architecture](0001-observability-first-architecture.md)
- [FIPA Agent Communication Language Specification](http://www.fipa.org/specs/fipa00061/)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/reference/specification/trace/semantic_conventions/)
