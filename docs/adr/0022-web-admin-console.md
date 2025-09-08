---
title: "0022. Web-based Admin Console"
date: 2025-09-03
status: accepted
layout: adr
categories: [Architecture, Technology, User Interface]
---


Date: 2025-09-03

## Status

Accepted

## Context

Caxton requires an administrative interface for managing agent
deployments, monitoring system health, and debugging multi-agent
interactions. Our architectural journey has evolved from complex
distributed solutions toward simplicity and operational excellence.

**Original Plan**: gRPC + protobuf API with multi-language SDK
generation and separate CLI client distribution (ADR-0007)

**First Revision**: HTTP/JSON REST API with dedicated Rust CLI
client, requiring separate binary distribution and version coordination

**Current Need**: Simple, accessible management interface that
eliminates client distribution complexity while maintaining professional
operational capabilities

## Decision Drivers

- **Operational Simplicity**: Eliminate need to distribute and
  version-sync separate client binaries
- **Universal Access**: Any modern web browser can access the
  interface without additional software
- **Development Efficiency**: Single codebase reduces maintenance
  overhead and development complexity
- **Professional User Experience**: Rich, interactive interface
  superior to CLI for visual system monitoring
- **Real-time Monitoring**: Live updates of agent status, message
  flows, and system health
- **Security Control**: Server-controlled authentication without
  client-side key management

## Decision

We will implement a **web-based admin console** using server-side rendering
with progressive enhancement instead of gRPC API + CLI client.

### Core Architecture

**Web Stack Selection**:

```rust
// Web framework with static file serving
axum::Router::new()
    .route("/api/v1/*path", api_routes)
    .route_service("/", ServeDir::new("static")
        .not_found_service(ServeFile::new("static/index.html")))
```

**Technology Components**:

- **Axum + Tower-HTTP**: Web server with static file serving and API endpoints
- **Askama Templates**: Server-side HTML rendering with type-safe templating
- **HTMX**: Dynamic interactions without complex JavaScript frameworks
- **Server-Sent Events (SSE)**: Real-time updates for agent status and
  message monitoring
- **Alpine.js**: Minimal client-side interactivity for UI components

### Interface Design

**Dashboard Overview**:

- Live agent status grid with health indicators
- Real-time message flow visualization
- System resource usage graphs
- Recent activity timeline

**Agent Management**:

- WASM module upload with drag-and-drop interface
- Agent deployment with resource limit configuration
- Live deployment status with progress indicators
- Agent lifecycle controls (start, stop, restart, undeploy)

**Message Monitoring**:

- Live message stream with filtering capabilities
- Conversation thread visualization
- Message content inspection with JSON pretty-printing
- Performance metrics per agent

**System Health**:

- Resource usage dashboards (CPU, memory, message throughput)
- System logs with real-time streaming
- Health checks with alert status
- Configuration management interface

### Implementation Benefits

**Single Binary Deployment**:

```bash
# Complete Caxton deployment with integrated admin console
caxton server
```

**Real-time Updates via SSE**:

```javascript
// HTMX + SSE integration for live status
<div hx-ext="sse" sse-connect="/api/v1/events">
    <div sse-swap="agent-status" hx-swap="innerHTML">
        Loading agent status...
    </div>
</div>
```

**Simplified Development**:

- No protobuf schema management
- No multi-language SDK generation
- No client version coordination
- Single codebase for both API and UI

## Architecture Details

### Server-Side Rendering Rationale

**Why SSR over SPA**:

- **Immediate Functionality**: Pages work without JavaScript, enhanced
  progressively
- **Server-Controlled State**: Eliminates client-server state
  synchronization complexity
- **Built-in Security**: CSRF protection, secure session management,
  no client-side secrets
- **Simple Deployment**: No build pipeline coordination, assets served directly
- **Professional Polish**: Server-rendered templates with consistent styling

### Real-time Communication Strategy

**Server-Sent Events (SSE) Over WebSockets**:

- Simpler implementation and debugging
- Automatic reconnection handling by browsers
- One-way server-to-client communication (appropriate for monitoring)
- Built-in with Axum via `axum::response::sse`
- HTMX integration for seamless DOM updates

### Security Architecture

**Authentication & Authorization**:

- Server-side session management with secure cookies
- CSRF protection for all state-changing operations
- Role-based access control for different admin functions
- Rate limiting for file uploads and API endpoints

**File Upload Security**:

```rust
// Secure WASM upload with validation
async fn upload_wasm(
    multipart: Multipart,
    Extension(validator): Extension<WasmValidator>,
) -> Result<Json<UploadResponse>, AppError> {
    // Size limits, content validation, streaming processing
}
```

## Consequences

### Positive

- **Zero Client Distribution**: Eliminates binary distribution and
  version coordination complexity
- **Universal Access**: Any web browser provides immediate access
  without installation
- **Rich User Experience**: Visual interfaces superior to CLI for
  system monitoring and debugging
- **Development Simplicity**: Single Rust codebase, no multi-language
  SDK maintenance
- **Real-time Monitoring**: Live updates provide immediate system state
  visibility
- **Professional Operations**: Dashboard-style interfaces familiar to
  operations teams
- **Built-in Security**: Server-controlled authentication and CSRF protection
- **Simple Deployment**: Single binary includes complete management interface

### Negative

- **JavaScript Dependency**: Requires browsers with JavaScript enabled
  (though gracefully degrades)
- **Network Requirement**: Cannot function offline like a local CLI client
- **Single Point of Access**: Web interface ties administration to
  server availability
- **Browser Compatibility**: Must support multiple browser versions and
  capabilities

### Mitigation Strategies

**JavaScript Graceful Degradation**:

- Core functionality works with server-side rendering only
- Progressive enhancement adds interactivity
- Clear fallbacks for essential operations

**Network Resilience**:

- Offline detection with appropriate messaging
- Local storage for non-critical user preferences
- Automatic reconnection for real-time features

**Browser Compatibility**:

- Modern web standards with broad support
- Feature detection for progressive enhancement
- Responsive design for various screen sizes

## User Experience Examples

### Agent Deployment Flow

1. **Upload WASM Module**: Drag-and-drop interface with validation feedback
2. **Configure Resources**: Visual sliders for memory and CPU limits
   with recommendations
3. **Deploy Agent**: Progress indicator with real-time status updates
4. **Monitor Health**: Live dashboard showing agent startup and health checks

### Message Flow Debugging

1. **Live Message Stream**: Real-time display of all agent messages
   with filtering
2. **Conversation Threading**: Visual representation of request-response chains
3. **Content Inspection**: Expandable message content with JSON syntax
   highlighting
4. **Performance Metrics**: Per-agent message rates and processing times

### System Monitoring Dashboard

1. **Resource Usage Graphs**: Live CPU, memory, and storage utilization
2. **Agent Status Grid**: Visual health indicators with click-for-details
3. **Activity Timeline**: Recent deployments, errors, and system events
4. **Alert Integration**: Highlighted issues with investigation links

## Related Decisions

- ADR-0001: Observability-First Architecture - Provides metrics and
  tracing for dashboard
- ADR-0006: Application Server Architecture - Established Caxton as
  standalone server
- ADR-0007: Management API Design - Original gRPC approach superseded
  by this decision
- ADR-0016: Security Architecture - Defines authentication and
  authorization patterns

## References

- [HTMX Documentation](https://htmx.org/) - Dynamic interactions
  without complex JavaScript
- [Server-Sent Events Specification](
  https://html.spec.whatwg.org/multipage/server-sent-events.html)
- [Axum Web Framework](https://docs.rs/axum/) - Rust web framework
  with excellent static file support
- [Askama Templates](https://docs.rs/askama/) - Type-safe HTML
  templating for Rust
