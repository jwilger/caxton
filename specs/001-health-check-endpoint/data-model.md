# Data Model: Health Check Endpoint

## Core Entities

### HealthResponse

Represents the JSON response returned by the health check endpoint.

**Fields:**

- `status`: HealthStatus - Current health state of the server

**Validation Rules:**

- Must serialize to valid JSON
- Content-Type header must be `application/json`
- Always returns HTTP 200 when server is operational

**Relationships:**

- Contains one HealthStatus value
- Serialized by serde to JSON format

### HealthStatus

Enumerated type representing possible server health states.

**Variants:**

- `Ok` - Server is operational and ready to handle requests

**Serialization:**

- `Ok` serializes to JSON string `"OK"`
- Uses serde rename for consistent API format

**Future Extensions:**

- Additional variants for degraded states (Warning, Critical)
- Metadata fields for detailed health information

### ServerConfig

Configuration structure for HTTP server settings loaded from TOML.

**Fields:**

- `host`: HostAddress - Server bind address with validation
- `port`: ServerPort - Server port with range validation
- `health_path`: EndpointPath - Health endpoint path with format validation

**Domain Types (using nutype):**

#### HostAddress

```rust
#[nutype(
    validate(not_empty),
    derive(Debug, Clone, Serialize, Deserialize, Display)
)]
pub struct HostAddress(String);
```

**Validation Rules:**

- Must not be empty string
- Should be valid IP address or hostname format
- Property tests: valid IPv4, IPv6, hostnames, reject invalid formats

#### ServerPort

```rust
#[nutype(
    validate(greater = 0, less_equal = 65535),
    derive(Debug, Clone, Copy, Serialize, Deserialize, Display)
)]
pub struct ServerPort(u16);
```

**Validation Rules:**

- Must be in range 1-65535 (valid port range)
- Zero is invalid (reserved)
- Property tests: valid ports, boundary values, reject invalid ranges

#### EndpointPath

```rust
#[nutype(
    validate(predicate = |path| path.starts_with('/') && !path.is_empty()),
    derive(Debug, Clone, Serialize, Deserialize, Display)
)]
pub struct EndpointPath(String);
```

**Validation Rules:**

- Must start with '/' (valid HTTP path)
- Must not be empty
- Should be valid URL path format
- Property tests: valid paths, reject malformed paths, boundary cases

**TOML Integration:**

- Loaded from `[server]` section of Caxton configuration
- nutype validation occurs during TOML deserialization
- Validation failures prevent server startup (fail-fast)
- Supports environment variable overrides with validation

## State Transitions

### Server Lifecycle

```
[Startup] → [Initializing] → [Running] → [Shutdown]
    ↓            ↓             ↓           ↓
[No Response] [No Response] [Health OK] [No Response]
```

### Health Check Request Flow

```
HTTP Request → Route Matching → Health Handler → JSON Response
     ↓              ↓               ↓              ↓
[GET/HEAD]    [/health path]   [HealthStatus]  [200 + JSON]
```

## Type Safety Guarantees

### Parse Boundaries

1. **TOML Configuration**: nutype validation during TOML deserialization
2. **HTTP Request Parsing**: Axum handles HTTP request validation
3. **Route Matching**: Type-safe path parameters and method validation
4. **Response Serialization**: Serde ensures valid JSON output

### Domain Constraints (via nutype)

- `HostAddress` cannot be empty or malformed
- `ServerPort` must be in valid port range (1-65535)
- `EndpointPath` must be valid HTTP path format
- `HealthStatus` can only represent valid states
- All validation occurs at construction time (impossible to create invalid instances)

### Error Handling

- Configuration validation errors prevent server startup (fail-fast)
- Invalid HTTP methods return `405 Method Not Allowed`
- Malformed requests return `400 Bad Request` with JSON error
- Server errors return `500 Internal Server Error` with structured response

### Property Testing Strategy

- **HostAddress**: Generate valid IP addresses, hostnames, reject empty/invalid formats
- **ServerPort**: Test boundary values (1, 65535), reject 0 and >65535
- **EndpointPath**: Generate valid paths starting with '/', reject malformed paths
- **Integration**: Test TOML parsing with generated valid/invalid configurations

## Future Extension Points

### Enhanced Health Checks

- Optional dependency health verification
- Performance metrics in response
- Custom health check registration

### Configuration Management

- Runtime configuration updates via API (with nutype validation)
- Per-environment configuration overrides
- Validation of configuration changes before application

### Agent Platform Integration

- Agent runtime health status
- Resource utilization monitoring
- Agent-specific health endpoints

## Implementation Notes

### Rust Type Definitions

```rust
use nutype::nutype;
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
}

#[derive(Serialize)]
pub enum HealthStatus {
    #[serde(rename = "OK")]
    Ok,
}

#[nutype(
    validate(not_empty),
    derive(Debug, Clone, Serialize, Deserialize, Display)
)]
pub struct HostAddress(String);

#[nutype(
    validate(greater = 0, less_equal = 65535),
    derive(Debug, Clone, Copy, Serialize, Deserialize, Display)
)]
pub struct ServerPort(u16);

#[nutype(
    validate(predicate = |path| path.starts_with('/') && !path.is_empty()),
    derive(Debug, Clone, Serialize, Deserialize, Display)
)]
pub struct EndpointPath(String);

#[derive(Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: HostAddress,
    #[serde(default = "default_port")]
    pub port: ServerPort,
    #[serde(default = "default_health_path")]
    pub health_path: EndpointPath,
}

// Default value constructors (guaranteed valid by nutype)
fn default_host() -> HostAddress {
    HostAddress::new("0.0.0.0".to_string()).unwrap()
}

fn default_port() -> ServerPort {
    ServerPort::new(8080).unwrap()
}

fn default_health_path() -> EndpointPath {
    EndpointPath::new("/health".to_string()).unwrap()
}
```

### Property Testing Examples

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn valid_server_ports_always_accepted(port in 1u16..=65535) {
        prop_assert!(ServerPort::new(port).is_ok());
    }

    #[test]
    fn invalid_server_ports_always_rejected(port in 0u16..=0u16) {
        prop_assert!(ServerPort::new(port).is_err());
    }

    #[test]
    fn valid_endpoint_paths_start_with_slash(path in "/[a-zA-Z0-9_/-]*") {
        prop_assume!(!path.is_empty());
        prop_assert!(EndpointPath::new(path).is_ok());
    }
}
```

### Database Considerations

N/A - Health check endpoint requires no persistent storage, uses only in-memory response generation.

### Performance Characteristics

- nutype validation: O(1) for numeric types, O(n) for string validation
- Response generation: O(1) constant time after validation
- Memory usage: Minimal overhead from nutype wrappers
- Serialization: Compile-time JSON generation via serde
- Concurrency: Stateless handler supports unlimited concurrent requests
