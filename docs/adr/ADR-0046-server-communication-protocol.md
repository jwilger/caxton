---
title: ADR-0046 Server Communication Protocol
date: 2025-09-16
status: accepted
layout: adr
categories: [Architecture Decisions]
---

# ADR-0046: Server Communication Protocol

## Status

Accepted

## Context

The Caxton deployment model requires reliable, performant communication between the CLI client and the server for configuration deployment operations. This communication protocol must support:

1. **Performance requirements**: < 500ms single agent, < 2s for 5-10 agents deployment latency
2. **Incremental operations**: Efficient transmission of configuration diffs and binary patches
3. **Real-time feedback**: Progress updates during multi-agent deployments
4. **Network constraints**: Operation through corporate proxies and firewalls
5. **Debugging needs**: Observable and debuggable protocol for operational support
6. **Security requirements**: Authentication, encryption, and access control

The protocol must align with:

- ADR-0043 Manual Deployment Commands establishing explicit deployment control
- ADR-0044 Incremental Deployment Strategy requiring efficient diff transmission
- ADR-0045 Configuration State Management needing version negotiation and conflict detection

## Decision

We will use **HTTP/2 with REST API patterns and JSON payloads** as the primary communication protocol between the Caxton CLI and server.

### Protocol Stack

1. **Transport**: HTTP/2 over TLS
   - Binary framing for efficiency
   - Header compression (HPACK)
   - Request/response multiplexing
   - Server push capabilities
   - Automatic stream prioritization

2. **API Style**: RESTful resource-oriented design
   - Standard HTTP verbs (GET, POST, PUT, DELETE)
   - Resource-based URLs (`/api/v1/workspaces/{id}/agents`)
   - Standard HTTP status codes
   - JSON request/response bodies

3. **Real-time Updates**: Server-Sent Events (SSE)
   - Progress notifications during deployment
   - Agent status changes
   - Error notifications
   - Graceful fallback to polling if SSE unavailable

### Authentication and Security

1. **Authentication**: Bearer token in Authorization header
   - JWT tokens with workspace claims
   - Token refresh mechanism
   - Optional mTLS for high-security environments

2. **Encryption**: TLS 1.3 minimum
   - Certificate pinning optional
   - Forward secrecy required

3. **Access Control**: Workspace-scoped permissions
   - Token includes workspace ID and permissions
   - Server validates all operations against token claims

### Connection Management

1. **Connection Pooling**: HTTP/2 persistent connections
   - Single connection for all requests
   - Automatic reconnection with exponential backoff
   - Connection keep-alive with PING frames

2. **Request Timeouts**:
   - Configuration deployment: 30s timeout
   - Status queries: 5s timeout
   - Bulk operations: 60s timeout

3. **Error Handling**:
   - Automatic retry with exponential backoff for transient errors
   - Circuit breaker pattern for server unavailability
   - Clear error messages with remediation hints

### Payload Optimization

1. **Configuration Transfer**:
   - JSON for small configs (< 10KB)
   - Gzip compression for larger payloads
   - Binary patches (xdelta3) transmitted as base64 in JSON

2. **Batch Operations**:
   - Array of operations in single request
   - Transaction semantics for atomicity
   - Partial failure reporting with rollback

## Consequences

### Positive

1. **Standard Tooling**: Works with curl, httpie, Postman for debugging
2. **Proxy Compatibility**: HTTP/2 degrades gracefully to HTTP/1.1 through proxies
3. **Observable**: Standard HTTP logging, metrics, and tracing
4. **Browser Compatible**: Future web UI can use same API
5. **Efficient**: HTTP/2 multiplexing reduces connection overhead
6. **Simple Implementation**: Well-supported in Rust (hyper, reqwest, axum)

### Negative

1. **Text Overhead**: JSON less efficient than binary protocols
2. **Streaming Complexity**: SSE requires additional connection management
3. **HTTP/2 Adoption**: Some corporate proxies may not support HTTP/2

### Mitigation

1. **Compression**: Gzip/Brotli compression reduces JSON overhead
2. **Graceful Degradation**: Fall back to HTTP/1.1 when HTTP/2 unavailable
3. **Polling Fallback**: Support polling if SSE blocked by proxies

## Implementation Considerations

### Key Requirements

1. Content negotiation via Accept headers
2. Version negotiation via API path (`/api/v1/`)
3. Idempotency keys for retry safety
4. Request IDs for distributed tracing
5. Rate limiting with standard headers

### API Endpoints

Core deployment endpoints to implement:

- `POST /api/v1/workspaces/{id}/deploy` - Deploy configuration changes
- `GET /api/v1/workspaces/{id}/status` - Get deployment status
- `GET /api/v1/workspaces/{id}/events` - SSE endpoint for real-time updates
- `POST /api/v1/workspaces/{id}/rollback` - Rollback deployment (future)

## Alternatives Considered

1. **gRPC**: Better performance but poor proxy support and debugging complexity
2. **WebSockets**: Bidirectional but complex connection management
3. **GraphQL**: Flexible queries but overkill for simple deployment operations
4. **Custom TCP**: Maximum control but high implementation complexity

## References

- ADR-0043: Manual Deployment Commands
- ADR-0044: Incremental Deployment Strategy
- ADR-0045: Configuration State Management
- ADR-0026: Simplified Management API Protocol (establishes REST precedent)
- EVENT_MODEL.md v2.1: Deployment workflow requirements
