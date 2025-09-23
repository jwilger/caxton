# ADR-001: HTTP Framework Selection for Caxton Platform

## Status

Accepted

## Context

Caxton requires HTTP server infrastructure to provide:

- Health check endpoints for monitoring and load balancing
- Future API endpoints for agent management, configuration, and observability
- Foundation for the extensible AI agent platform architecture

The choice of HTTP framework affects type safety, performance, dependency footprint, development velocity, and long-term maintainability of the platform.

### Constitutional Requirements

- **Zero External Service Dependencies**: No external services required for basic functionality
- **Type-Driven Safety**: All external inputs parsed at boundaries with comprehensive validation
- **Minimal Core Architecture**: HTTP server is justified core infrastructure
- **Observability First**: All operations must include tracing and logging

### Technical Requirements

- Sub-100ms response time for health checks
- Support for GET and HEAD methods initially
- JSON response format with proper Content-Type headers
- Extensible architecture for future API endpoints
- Integration with TOML configuration system
- Error handling with structured JSON responses

## Decision

We will use **Axum** as the HTTP framework for Caxton.

### Dependencies Selected

- `axum` - Core HTTP framework with type-safe routing
- `tokio` - Async runtime (industry standard for Rust HTTP servers)
- `serde` - JSON serialization (universal Rust pattern)
- `tower` - Middleware ecosystem (included with axum)

## Alternatives Considered

### 1. std::net (Manual HTTP Implementation)

- **Pros**: Zero external dependencies, full control, minimal binary size
- **Cons**: High development overhead, security risks, limited HTTP features, no async support
- **Verdict**: Violates practical engineering needs despite constitutional alignment

### 2. tiny_http

- **Pros**: Minimal dependency footprint (1 dependency), basic HTTP handling
- **Cons**: No async support, limited ecosystem, basic feature set, poor scalability
- **Verdict**: Insufficient for platform foundation requirements

### 3. hyper

- **Pros**: High performance, HTTP/2 support, lower-level control, proven in production
- **Cons**: More complex API, significant boilerplate, steeper learning curve
- **Verdict**: Over-engineered for current needs, complex for rapid development

### 4. warp

- **Pros**: Functional composition, good type safety, filter-based architecture
- **Cons**: Steep learning curve, complex error messages, less intuitive API
- **Verdict**: Learning curve overhead outweighs benefits

### 5. actix-web

- **Pros**: Mature ecosystem, high performance, extensive middleware
- **Cons**: Heavier dependency tree, more complex actor-based architecture
- **Verdict**: Over-engineered for current platform needs

## Rationale

### Constitutional Compliance

- **Zero External Service Dependencies**: ✅ Axum requires no external services, only Rust crates
- **Type-Driven Safety**: ✅ Excellent compile-time guarantees, type-safe extractors and responses
- **Minimal Core Architecture**: ✅ HTTP server is justified core infrastructure for monitoring
- **Observability First**: ✅ Built-in tracing support, middleware for structured logging

### Technical Alignment

- **Performance**: Async-first design easily meets sub-100ms requirement
- **Type Safety**: Compile-time route validation, type-safe request/response handling
- **Extensibility**: Modular design supports future API expansion
- **Ecosystem**: Strong community, active development, production-proven
- **Learning Curve**: Intuitive API reduces development overhead

### Platform Foundation

- **Future Growth**: Router composition supports complex API structures
- **Middleware**: Tower ecosystem provides cross-cutting concerns (auth, logging, metrics)
- **Testing**: Built-in testing utilities for integration and contract testing
- **Documentation**: Excellent documentation and community resources

### Constitutional Clarification Impact

The constitutional amendment (v1.2.1) clarifying that "Zero External Service Dependencies" applies to services (databases, vector stores) not Rust crates enabled this decision. Without this clarification, we would have been forced into manual std::net implementation with significant technical debt.

## Consequences

### Positive

- **Rapid Development**: High-level API reduces boilerplate and development time
- **Type Safety**: Compile-time guarantees prevent common HTTP server vulnerabilities
- **Performance**: Async architecture supports high-concurrency workloads
- **Maintainability**: Clear, idiomatic Rust code with excellent error messages
- **Ecosystem**: Access to Tower middleware for authentication, logging, metrics
- **Testing**: Built-in testing support for integration and contract validation

### Negative

- **Dependency Count**: ~15-20 transitive dependencies increase binary size
- **Learning Curve**: Team must learn async Rust and axum patterns
- **Version Risk**: Dependency on external crate evolution and maintenance
- **Debugging Complexity**: Async stack traces can be more complex to debug

### Mitigation Strategies

- **Dependency Management**: Pin versions for stability, regular security audits
- **Knowledge Building**: Team training on async Rust and axum patterns
- **Testing Strategy**: Comprehensive integration tests to catch framework changes
- **Migration Planning**: Modular HTTP layer design enables future framework changes

## Implementation Notes

### Architecture Pattern

```rust
// Type-safe route definitions
#[derive(Serialize)]
struct HealthResponse {
    status: HealthStatus,
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse { status: HealthStatus::Ok })
}

// Router composition for extensibility
fn health_router() -> Router {
    Router::new()
        .route("/health", get(health_handler).head(health_handler))
}
```

### Configuration Integration

- TOML configuration parsed into type-safe structures
- Server configuration via `ServerConfig` with nutype validation
- Environment variable overrides with validation

### Testing Strategy

- Unit tests for response serialization
- Integration tests for HTTP contract compliance
- Property tests for configuration validation
- Performance benchmarks for response time requirements

## Review Date

This decision should be reviewed when:

- Performance requirements exceed axum capabilities
- Security vulnerabilities require framework changes
- Major axum version changes affect API compatibility
- Alternative frameworks provide significant advantages

---

**Decision Date**: 2025-09-22
**Participants**: Architecture review, Constitutional compliance verification
**Next Review**: Major version upgrade or performance issues
