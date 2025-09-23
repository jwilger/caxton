# Relevant Patterns & Practices: Rust HTTP Framework Options for Minimal Health Check

## Executive Summary

For a zero-dependency preference with minimal health check requirements, std::net provides the most aligned solution with no external dependencies. However, practical considerations favor lightweight frameworks like tiny_http or hyper for production readiness. Axum and warp offer excellent type safety but come with heavier dependency trees that conflict with the constitutional requirement.

## Applicable Patterns

### Zero-Dependency HTTP Server (std::net)

**Context**: When absolute minimal dependencies are required
**Concept**: Use Rust's standard library TcpListener for raw HTTP parsing
**Example Shape**:

```rust
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

fn handle_request(mut stream: TcpStream) {
    let response = "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\n\r\n{\"status\":\"OK\"}";
    stream.write_all(response.as_bytes()).unwrap();
}
```

**Real-world use**: Early-stage microservices, embedded systems
**Considerations**: Manual HTTP parsing, no request routing, security concerns

### Minimal Framework Pattern (tiny_http)

**Context**: When minimal dependencies are preferred but some conveniences needed
**Concept**: Ultra-lightweight HTTP server with basic parsing
**Example Shape**:

```rust
use tiny_http::{Server, Response};

let server = Server::http("localhost:3000").unwrap();
for request in server.incoming_requests() {
    let response = Response::from_string("{\"status\":\"OK\"}")
        .with_header("Content-Type: application/json".parse().unwrap());
    request.respond(response);
}
```

**Real-world use**: AWS Lambda cold starts, minimal Docker containers
**Considerations**: 1 core dependency, basic routing, limited ecosystem

### Type-Safe Framework Pattern (Axum)

**Context**: When type safety and ergonomics are prioritized over dependency count
**Concept**: Leverage Rust's type system for compile-time correctness
**Example Shape**:

```rust
use axum::{Json, response::Json as ResponseJson, routing::get, Router};
use serde::Serialize;

#[derive(Serialize)]
struct HealthResponse { status: String }

async fn health() -> ResponseJson<HealthResponse> {
    ResponseJson(HealthResponse { status: "OK".to_string() })
}

let app = Router::new().route("/health", get(health));
```

**Real-world use**: Production APIs, companies like Discord, Fly.io
**Considerations**: ~15-20 dependencies, excellent type safety, strong ecosystem

## Testing Insights

### Outside-In Testing

- Concept: Start from HTTP client perspective
- Benefits: Validates actual HTTP contract
- Example: Integration tests making real HTTP requests

### Property-Based Testing

- Concept: Define HTTP response invariants
- Benefits: Catches edge cases in parsing/serialization
- Example properties: "health endpoint always returns 200", "response is valid JSON"

## Framework Comparison Analysis

### Dependency Footprint (Smallest to Largest)

1. **std::net**: 0 dependencies
   - Pros: Constitutionally compliant, maximum control
   - Cons: Manual HTTP parsing, security risks, significant development overhead

2. **tiny_http**: 1 dependency (ascii)
   - Pros: Minimal footprint, basic HTTP handling
   - Cons: Limited routing, basic feature set

3. **hyper**: ~8-10 dependencies
   - Pros: Industry standard, excellent performance, HTTP/2 support
   - Cons: Lower-level API, requires more boilerplate

4. **warp**: ~15 dependencies
   - Pros: Functional composition, good type safety
   - Cons: Steep learning curve, complex error messages

5. **axum**: ~15-20 dependencies
   - Pros: Excellent ergonomics, strong type safety, active development
   - Cons: Larger dependency tree, async complexity

### Performance Characteristics

- **std::net**: Fastest raw performance, manual optimization required
- **tiny_http**: Good performance for simple cases, single-threaded
- **hyper**: Excellent performance, highly optimized, HTTP/2
- **axum/warp**: Very good performance, async overhead acceptable

### Type Safety Features

- **std::net**: Manual type handling required
- **tiny_http**: Basic type conversions
- **hyper**: Good type safety with manual work
- **axum**: Excellent type safety with extractors and serialization
- **warp**: Good type safety with filter composition

## Common Pitfalls (Awareness)

- **std::net**: Often fails when HTTP parsing edge cases arise (malformed requests, chunked encoding)
- **tiny_http**: Watch out for lack of async support limiting scalability
- **Heavy frameworks**: Dependency bloat can conflict with zero-dependency goals

## Language-Specific Notes

### Rust HTTP Ecosystem

- Idiom: Async/await is standard for HTTP servers
- Library: serde for JSON serialization is near-universal
- Pattern: Tower middleware for cross-cutting concerns

## Case Studies

- **Discord** used axum for their new Rust services
- Result: Significant performance improvements over Python
- Lesson: Type safety reduced runtime errors substantially

- **Fly.io** infrastructure uses hyper extensively
- Result: Sub-millisecond HTTP routing performance
- Lesson: Lower-level control enables optimization

- **Cloudflare Workers** use std-only approach for edge computing
- Result: Minimal cold start times
- Lesson: Zero dependencies critical for edge deployment

## Learning Resources

- **Rust HTTP Workshop**: Covers hyper → axum progression
- Depth: Intermediate to Advanced
- Time investment: 2-3 days to become productive

- **Zero to Production in Rust**: Comprehensive web development guide
- Depth: Beginner to Advanced
- Time investment: 1-2 weeks for full coverage

## Recommendations by Priority

### Constitutional Compliance (Zero Dependencies)

**Recommendation**: std::net

- Rationale: Only option that fully meets zero-dependency requirement
- Trade-off: Significant development overhead and security considerations

### Practical Minimum (Minimal Dependencies)

**Recommendation**: tiny_http

- Rationale: 1 dependency, good enough for health checks
- Trade-off: Limited growth potential

### Type-Safe Growth (Long-term Platform)

**Recommendation**: axum

- Rationale: Excellent type safety, strong ecosystem, future-proof
- Trade-off: 15-20 dependencies conflicts with constitution

### Performance Critical (Sub-100ms requirement)

**Recommendation**: hyper

- Rationale: Proven performance, HTTP/2 support
- Trade-off: More complex API

## Questions Pattern Raises

- Does constitutional zero-dependency requirement override practical considerations?
- Would team benefit from async/await patterns for future platform growth?
- Is security handling acceptable with manual HTTP parsing?
- How important is ecosystem compatibility vs dependency minimalism?

## Constitutional Consideration

The key tension is between the constitutional preference for zero dependencies and practical software engineering. For a health check endpoint specifically, std::net can work, but for a "foundation for larger platform," the technical debt may be prohibitive.

**Recommendation**: Consider constitutional amendment to allow "essential infrastructure dependencies" for HTTP handling, or implement std::net solution with plan to migrate when constitutional requirements allow.
