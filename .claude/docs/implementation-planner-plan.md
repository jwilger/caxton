# Implementation Design Options: HTTP API Foundation with TOML Integration

## Executive Summary

The HTTP API foundation requires deep integration with Caxton's TOML configuration system, following Constitutional Principle I (TOML as the standard). Design options range from minimal std::net approaches that honor zero-dependency principles to type-safe frameworks that enable robust agent platform APIs. Key tensions exist between constitutional compliance and practical API development needs. All approaches must prioritize TOML-first configuration, modular API organization, and type-safe request/response handling for agent operations.

## Type Design Options

### Option A: Domain-Driven Configuration Types

Types needed:

- `ServerConfig` - HTTP server settings from TOML
- `ApiModuleConfig` - Per-module configuration
- `AgentApiConfig` - Agent-specific API settings
- `TomlConfigResponse<T>` - Typed TOML config responses
- `HealthResponse` - Health check response type
- `ConfigUpdateRequest<T>` - TOML config modification requests

Benefits: Strong type safety, clear domain boundaries, configuration-first design
Tradeoffs: More upfront modeling, potential over-engineering for simple cases

### Option B: Lightweight Response Wrappers

Types needed:

- `ApiResponse<T>` - Generic success/error wrapper
- `HealthStatus` - Simple status enum
- `ConfigSection` - Raw TOML section wrapper
- `ErrorResponse` - Standardized error format

Benefits: Minimal type overhead, flexible for rapid development
Tradeoffs: Less type safety, weaker domain modeling, more runtime validation

### Option C: Hybrid Configuration-Response Types

Types needed:

- `ServerConfig` + `ConfigResponse<T>` - Configuration and API response separation
- `HealthResponse` - Dedicated health check type
- `AgentConfigData` - Agent-specific configuration wrapper
- `TomlValidationError` - Configuration parsing errors

Benefits: Balance between safety and simplicity, clear config/API separation
Tradeoffs: Medium complexity, requires careful boundary management

## Parse Strategy Options

### Eager Parsing at TOML Load

Parse at: Application startup, configuration file changes
Advantages: Fail fast on invalid config, type safety throughout system, clear error reporting
Considerations: Memory usage for large configs, restart required for changes

### Lazy Parsing at API Request

Parse at: Individual API endpoint handlers
Advantages: Flexible configuration updates, memory efficient, partial config validation
Considerations: Runtime errors in production, complex error handling in handlers

### Hybrid: Config Parse + Request Validation

Parse at: Config load for structure, request time for business rules
Advantages: Early structural validation, runtime business logic validation
Considerations: Dual validation logic, potential inconsistencies

## Error Handling Approaches

### Result Types Throughout

Pattern: Every config operation returns `Result<T, ConfigError>`
Benefits: Explicit error handling, composable with `?` operator, type-safe
Costs: Verbosity in simple cases, learning curve for team

### Parse Boundaries + Exceptions

Pattern: Parse config at startup, panic on invalid runtime state
Benefits: Simpler internal API code, clear fail-fast behavior
Costs: Less graceful degradation, harder testing of error cases

### Hybrid: Results for Config, Panics for Programmer Errors

Pattern: `Result` for user config errors, `panic!` for internal invariant violations
Benefits: User-friendly config errors, fast failure for bugs
Costs: Requires careful categorization of error types

## HTTP Framework Architecture Options

### Option A: std::net + Manual HTTP Parsing (Constitutional Compliance)

Structure:

```
src/
├── http/
│   ├── server.rs       // std::net::TcpListener wrapper
│   ├── request.rs      // Manual HTTP parsing
│   ├── response.rs     // Manual HTTP formatting
│   └── router.rs       // Simple path matching
├── api/
│   ├── health.rs       // Health endpoint
│   ├── agents/         // Future agent APIs
│   └── config/         // Future config APIs
└── config/
    ├── toml_parser.rs  // TOML configuration loading
    └── server_config.rs // HTTP server configuration
```

Benefits: Zero external dependencies, full control, constitutional compliance
Considerations: High development overhead, limited HTTP features, security concerns

### Option B: Minimal Framework (tiny_http compromise)

Structure:

```
src/
├── http/
│   ├── server.rs       // tiny_http wrapper
│   ├── middleware.rs   // Custom middleware pipeline
│   └── routing.rs      // Route registration system
├── api/
│   ├── modules/        // Modular API organization
│   │   ├── health.rs
│   │   ├── agents.rs
│   │   └── config.rs
│   └── responses.rs    // Standardized response types
└── config/
    ├── toml_config.rs  // Deep TOML integration
    └── api_config.rs   // API-specific configuration
```

Benefits: Minimal dependency footprint, basic HTTP features, reasonable development speed
Considerations: Limited ecosystem, potential framework limitations

### Option C: Type-Safe Framework (axum/hyper - dependency compromise)

Structure:

```
src/
├── http/
│   ├── server.rs       // Axum server setup
│   ├── extractors.rs   // Custom TOML config extractors
│   └── middleware.rs   // Observability, validation
├── api/
│   ├── health/         // Health check module
│   ├── agents/         // Agent management APIs
│   ├── config/         // TOML config CRUD APIs
│   └── observability/  // Metrics/tracing APIs
├── config/
│   ├── toml_integration.rs // Deep TOML parsing/validation
│   └── api_state.rs    // Shared application state
└── types/
    ├── requests.rs     // API request types
    └── responses.rs    // API response types
```

Benefits: Type safety, rich ecosystem, excellent error handling, middleware support
Considerations: Violates dependency preferences, larger binary size

## TOML Configuration Integration Patterns

### Pattern A: Direct TOML Embedding

```rust
#[derive(Deserialize)]
struct CaxtonConfig {
    server: ServerConfig,
    agents: AgentConfig,
    apis: ApiConfig,
}

#[derive(Deserialize)]
struct ServerConfig {
    host: String,
    port: u16,
    tls: Option<TlsConfig>,
}
```

Benefits: Simple, direct mapping, single source of truth
Considerations: TOML structure drives code structure, harder to evolve

### Pattern B: Configuration Layering

```rust
trait ConfigSection {
    fn load_from_toml(toml: &toml::Value) -> Result<Self, ConfigError>;
    fn section_name() -> &'static str;
}

struct ServerConfig;
impl ConfigSection for ServerConfig { /* ... */ }
```

Benefits: Modular configuration, easy to extend, clear ownership
Considerations: More complex initialization, potential duplication

### Pattern C: TOML-First API Design

```rust
// API endpoints mirror TOML structure
// GET /api/config/server -> ServerConfig section
// PUT /api/config/agents/foo -> Update agent foo config
// PATCH /api/config/server/port -> Update specific setting
```

Benefits: Natural API design, direct TOML manipulation, clear semantics
Considerations: Exposes internal structure, validation complexity

## Testing Strategy Options

### Outside-In Integration Testing

Start with: Full HTTP request/response integration tests
Then: Work inward to individual modules and parsers
Good for: Ensuring API contracts work end-to-end

Test Structure:

```rust
#[test]
fn health_endpoint_returns_json() {
    let server = test_server_with_config("test-config.toml");
    let response = server.get("/health");
    assert_eq!(response.status(), 200);
    assert_eq!(response.json(), {"status": "OK"});
}
```

### Inside-Out Domain Testing

Start with: Configuration parsing and domain types
Then: Build outward to HTTP handlers and integration
Good for: Validating TOML integration and type safety

Test Structure:

```rust
#[test]
fn server_config_parses_valid_toml() {
    let toml = r#"
        [server]
        host = "0.0.0.0"
        port = 8080
    "#;
    let config = ServerConfig::from_toml(toml).unwrap();
    assert_eq!(config.port, 8080);
}
```

### Hybrid Testing Approach

Start with: Core TOML parsing and key API endpoints
Then: Fill in middle layers with focused unit tests
Good for: Balancing coverage with development speed

## Integration Considerations

### With Existing Code

- Challenges: Minimal existing codebase, need to establish patterns
- Approaches: Start with health endpoint as template, build modular foundation
- Migration path: N/A - greenfield development

### With External Systems

- Boundaries needed: TOML file watching, future agent runtime integration
- API contracts: JSON responses, REST conventions, error standardization
- Security: Input validation, configuration sanitization

### Future Agent Platform

- Agent lifecycle APIs: Create, start, stop, configure agents
- Configuration management: Per-agent TOML sections, runtime updates
- Observability integration: Metrics, tracing, health monitoring

## Questions to Consider

- What's most important: Constitutional compliance (zero deps) or platform foundation (type safety)?
- How much API complexity is acceptable for TOML integration depth?
- Should we optimize for current simplicity or future agent platform needs?
- What's the team's tolerance for HTTP framework learning curve vs manual implementation?
- How critical is runtime configuration updates vs restart-based changes?

## Recommended Decision Framework

1. **Constitutional Priority**: Start with zero-dependency approach if possible
2. **Pragmatic Compromise**: Consider minimal framework if std::net proves too limiting
3. **Future Platform**: Design API structure for agent management even if starting simple
4. **TOML Integration**: Prioritize deep TOML integration over generic configuration abstraction
5. **Type Safety**: Use strong types even in simple implementations
