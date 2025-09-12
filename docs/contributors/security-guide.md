---
title: "Security Developer Guide"
date: 2025-01-14
layout: page
categories: [Developer Guide, Security]
---

This guide covers security features and best practices for developing with
Caxton. For the complete security architecture, see
[ADR-0016: Security Architecture](../adrs/0016-security-architecture.md).

## Overview

Caxton implements defense-in-depth with multiple security layers:

1. **Configuration Agent Security**: Tool access control and permission management
2. **MCP Server Isolation**: WebAssembly sandboxing for tool implementations
3. **Message Security**: Capability-based routing and authorization
4. **API Security**: Multiple authentication methods
5. **Memory Security**: Embedded memory system with access controls
6. **Audit Logging**: Comprehensive security event tracking

### Security Model for Configuration Agents

Configuration agents (defined in markdown with YAML frontmatter) represent the
primary user experience. Security is implemented through:

- **Tool Permission Management**: Agents declare required tools and capabilities
- **MCP Server Sandboxing**: Actual functionality runs in isolated WebAssembly containers
- **Memory Access Controls**: Agent memory is scoped and access-controlled
- **Message Authentication**: All inter-agent communication is authenticated

## Agent Security

### Configuration Agent Security (Primary)

Configuration agents are secured through tool permission management and MCP
server isolation:

```yaml
---
name: DataAnalyzer
version: "1.0.0"
capabilities:
  - data-analysis
  - report-generation

# Tool permissions (enforced at runtime)
tools:
  - http_client        # HTTP requests via MCP server
  - csv_parser         # File parsing via MCP server
  - database_reader    # Database access via MCP server

# Memory settings
memory:
  enabled: true
  scope: workspace     # Shared within workspace
  max_entities: 10000  # Prevent memory abuse

# Security constraints
security:
  rate_limit: 100/minute
  require_auth: true
  allowed_capabilities: ["data-analysis", "report-generation"]
---

# Agent implementation in markdown...
```

Security is enforced through:

- **Tool Allowlist**: Only declared tools are accessible
- **MCP Server Isolation**: Tools run in WebAssembly sandboxes
- **Memory Scope Control**: Agent memory is workspace or agent-scoped
- **Rate Limiting**: Prevents abuse of agent resources
- **Capability Validation**: Runtime verification of declared capabilities

### Agent Capabilities

#### Configuration Agent Capabilities

Configuration agents declare capabilities in their YAML frontmatter:

```yaml
---
name: CustomerAnalyzer
capabilities:
  - message-send      # Can send agent messages
  - message-receive   # Can receive agent messages
  - tool-access       # Can use declared tools
  - memory-read       # Can read from agent memory
  - memory-write      # Can write to agent memory

tools:
  - database_reader   # Specific tool capability
  - email_sender     # Another tool capability
---
```

Capabilities are validated when agents attempt to use them. The runtime
prevents unauthorized operations.

### Secure Agent Deployment

#### Configuration Agent Deployment

Configuration agents are deployed directly from markdown files:

```bash
# Deploy configuration agent with validation
caxton deploy config-agent.md \
  --validate-schema \
  --check-capabilities \
  --verify-tools

# Deploy with restricted permissions
caxton deploy config-agent.md \
  --memory-scope workspace \
  --rate-limit 50/minute \
  --audit-mode strict
```

Security validations include:

- **Schema Validation**: YAML frontmatter must be valid
- **Capability Verification**: Declared capabilities must be available
- **Tool Authorization**: Requested tools must be permitted
- **Memory Scope**: Memory access is properly scoped

## Message Security

### Message Authentication

All messages include authentication:

```rust
use caxton::security::{MessageSigner, MessageVerifier};

// Signing messages
let signer = MessageSigner::new(private_key);
let signed_message = signer.sign(message)?;

// Verifying messages
let verifier = MessageVerifier::new(public_key);
let verified = verifier.verify(signed_message)?;
```

### Message Authorization

Control agent communication through capability-based policies:

```yaml
# Agent authorization policy
authorization:
  agent_id: order-processor

  # Capabilities this agent provides
  provides_capabilities:
    - order-processing
    - inventory-check
    - payment-validation

  # Who can request these capabilities
  capability_access:
    order-processing:
      allowed_requesters:
        - customer-interface
        - pattern: "admin-*"
    inventory-check:
      allowed_requesters:
        - "*"  # Public capability

  # Rate limiting per capability
  rate_limits:
    order-processing: 50/minute
    inventory-check: 200/minute

  # Message authentication requirements
  require_authentication: true
  require_conversation_tracking: true
```

#### Legacy Agent-to-Agent Authorization

Direct agent communication (still supported):

```yaml
# Traditional agent-to-agent policy
authorization:
  agent_id: order-processor

  allowed_senders:
    - inventory-manager
    - payment-processor
    - pattern: "customer-*"
```

### Encrypted Communication

Enable end-to-end encryption for sensitive messages:

```rust
use caxton::crypto::{encrypt_message, decrypt_message};

// Encrypt sensitive data
let encrypted = encrypt_message(
    &message,
    &recipient_public_key,
    EncryptionAlgorithm::ChaCha20Poly1305,
)?;

// Recipient decrypts
let decrypted = decrypt_message(
    &encrypted,
    &private_key,
)?;
```

## Memory Security

Configuration agents use the embedded memory system with built-in security controls:

### Memory Scope Control

```yaml
---
name: CustomerAgent
memory:
  enabled: true
  scope: workspace      # Options: agent, workspace, global
  max_entities: 10000   # Prevent memory exhaustion
  max_relations: 50000  # Limit relationship complexity
---
```

Memory scoping provides isolation:

- **Agent scope**: Private memory, only this agent can access
- **Workspace scope**: Shared within workspace, multiple agents can access
- **Global scope**: System-wide shared memory (admin-only)

### Memory Access Controls

```rust
// Memory operations are automatically access-controlled
use caxton::memory::{MemoryClient, AccessError};

let client = MemoryClient::for_agent("customer-agent").await?;

// This will fail if agent lacks memory-write capability
match client.store_entity(entity).await {
    Ok(id) => println!("Stored entity: {}", id),
    Err(AccessError::InsufficientCapability) => {
        // Agent needs memory-write capability
    },
    Err(AccessError::ScopeViolation) => {
        // Agent trying to access out-of-scope memory
    },
}
```

### Memory Encryption

Sensitive data in memory is automatically encrypted:

```yaml
# Memory encryption settings
memory:
  encryption:
    enabled: true
    algorithm: "AES-256-GCM"
    key_rotation: "24h"

  # Classify sensitive data
  sensitive_patterns:
    - "password"
    - "api_key"
    - "credit_card"
    - pattern: "ssn:\\d{3}-\\d{2}-\\d{4}"
```

### Memory Audit Trail

All memory operations are logged:

```rust
// Memory operations generate audit logs automatically
client.store_entity(sensitive_data).await?;
// AUDIT: Agent customer-agent stored entity customer_data_001

client.search_memory("customer preferences").await?;
// AUDIT: Agent customer-agent searched memory for "customer preferences"
```

## API Security

### Authentication Methods

Caxton supports multiple authentication methods:

#### API Key Authentication

```bash
# Generate API key
caxton auth create-key \
  --name "ci-deployment" \
  --permissions "agent:deploy,agent:list" \
  --expires-in 90d

# Use API key
curl -H "X-Caxton-Api-Key: cax_live_xxxxxxxxxxx" \
  https://api.caxton.example.com/agents
```

#### JWT Authentication

```javascript
// Client-side JWT usage
const token = jwt.sign(
  {
    sub: 'user-123',
    permissions: ['agent:read', 'message:send']
  },
  process.env.JWT_SECRET,
  { expiresIn: '1h' }
);

fetch('https://api.caxton.example.com/agents', {
  headers: {
    'Authorization': `Bearer ${token}`
  }
});
```

#### mTLS Client Certificates

```bash
# Generate client certificate
caxton security gen-client-cert \
  --cn "client-app" \
  --output client.pem

# Use with curl
curl --cert client.pem --key client-key.pem \
  https://api.caxton.example.com/agents
```

### Role-Based Access Control (RBAC)

Define roles and permissions:

```yaml
# roles.yaml
roles:
  - name: developer
    permissions:
      - agent:deploy
      - agent:list
      - agent:logs
      - message:send

  - name: operator
    permissions:
      - agent:*
      - cluster:*
      - config:*

  - name: viewer
    permissions:
      - agent:list
      - agent:read
      - metrics:read
```

Assign roles to users:

```bash
# Assign role to user
caxton auth assign-role \
  --user user@example.com \
  --role developer

# Check user permissions
caxton auth check \
  --user user@example.com \
  --permission agent:deploy
```

## Secret Management

### Using External Secrets

Never hardcode secrets. Use MCP tools for secret access:

```rust
// In your agent code
use caxton::mcp::SecretProvider;

pub async fn get_api_key() -> Result<String> {
    // Secrets fetched securely at runtime
    let provider = SecretProvider::connect().await?;
    let secret = provider.get_secret("external-api-key").await?;
    Ok(secret.value)
}
```

### Environment-Based Secrets

For configuration, use environment variables:

```yaml
# config.yaml
api:
  key: ${API_KEY}
  endpoint: ${API_ENDPOINT:-https://api.example.com}

database:
  password: ${DB_PASSWORD}
  connection_string: ${DATABASE_URL}
```

### Vault Integration

For production, integrate with HashiCorp Vault:

```rust
use caxton::secrets::VaultProvider;

let vault = VaultProvider::new(
    "https://vault.example.com",
    auth_token,
)?;

let db_creds = vault.get_database_credentials("myapp").await?;
```

## Audit Logging

### Security Events

All security events are automatically logged:

```rust
// These actions generate audit logs automatically
agent.deploy()?;        // AUDIT: Agent deployed by user-123
message.send()?;        // AUDIT: Message sent from agent-a to agent-b
config.update()?;       // AUDIT: Configuration changed by admin
```

### Custom Audit Events

Add custom security events:

```rust
use caxton::audit::{AuditLog, SecurityEvent};

// Log custom security event
AuditLog::record(SecurityEvent {
    event_type: "data_export",
    identity: current_user(),
    resource: "customer_database",
    outcome: "success",
    metadata: json!({
        "records_exported": 1000,
        "destination": "s3://backups/"
    }),
})?;
```

### Querying Audit Logs

```bash
# Find all failed authentication attempts
caxton audit query \
  --event-type login_failure \
  --last 24h

# Track configuration changes
caxton audit query \
  --event-type config_change \
  --user admin \
  --last 7d

# Export for SIEM
caxton audit export \
  --format json \
  --output audit-export.json \
  --from 2024-01-01
```

## Network Security

### Cluster Communication

Inter-node communication uses mTLS:

```yaml
# Cluster security configuration
security:
  cluster:
    mtls:
      enabled: true
      ca_cert: /etc/caxton/ca.crt
      node_cert: /etc/caxton/node.crt
      node_key: /etc/caxton/node.key

    # Gossip encryption
    gossip:
      encryption: true
      key_rotation_interval: 24h
```

### API TLS Configuration

```yaml
# API TLS settings
api:
  tls:
    enabled: true
    cert: /etc/caxton/api.crt
    key: /etc/caxton/api.key

    # TLS version and ciphers
    min_version: "1.3"
    cipher_suites:
      - TLS_AES_128_GCM_SHA256
      - TLS_AES_256_GCM_SHA384
      - TLS_CHACHA20_POLY1305_SHA256
```

## Security Best Practices

### Input Validation

Always validate inputs in your agents:

```rust
use caxton::validation::{validate_json, validate_schema};

pub fn handle_request(input: &str) -> Result<Response> {
    // Validate JSON structure
    let parsed = validate_json(input)?;

    // Validate against schema
    let validated = validate_schema(parsed, &REQUEST_SCHEMA)?;

    // Sanitize strings
    let sanitized = sanitize_input(validated)?;

    process_request(sanitized)
}
```

### Rate Limiting

Implement rate limiting for agents:

```rust
use caxton::ratelimit::RateLimiter;

let limiter = RateLimiter::new(
    100,  // requests
    Duration::from_secs(60),  // per minute
);

pub fn handle_message(msg: Message) -> Result<Response> {
    // Check rate limit
    if !limiter.check(&msg.sender)? {
        return Err(Error::RateLimitExceeded);
    }

    process_message(msg)
}
```

### Secure Defaults

Use secure defaults in your configurations:

```yaml
# Secure by default
security:
  authentication:
    enabled: true  # Always on
    require_tls: true
    session_timeout: 15m

  agents:
    require_signed_deployment: true
    capability_enforcement: strict
    resource_limits: enforced

  audit:
    enabled: true
    retention_days: 90
```

## Common Security Patterns

### Secure Service Communication

```rust
// Pattern: Authenticated service calls
pub struct SecureServiceClient {
    client: HttpClient,
    auth_token: String,
}

impl SecureServiceClient {
    pub async fn call(&self, endpoint: &str, data: &Value) -> Result<Response> {
        // Add authentication
        let request = self.client
            .post(endpoint)
            .header("Authorization", &self.auth_token)
            .json(data);

        // Add timeout for security
        let response = timeout(
            Duration::from_secs(30),
            request.send()
        ).await??;

        // Validate response
        self.validate_response(response)
    }
}
```

### Secure Data Handling

```rust
// Pattern: Secure data processing
pub struct SecureDataProcessor {
    encryptor: DataEncryptor,
    validator: DataValidator,
}

impl SecureDataProcessor {
    pub async fn process(&self, data: SensitiveData) -> Result<ProcessedData> {
        // Validate input
        self.validator.validate(&data)?;

        // Process in memory only
        let processed = self.transform(data)?;

        // Encrypt before storage
        let encrypted = self.encryptor.encrypt(processed)?;

        // Audit the operation
        AuditLog::record_data_processing(&encrypted.id)?;

        Ok(encrypted)
    }
}
```

## Security Testing

### Security Test Examples

```rust
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_unauthorized_access_denied() {
        let agent = TestAgent::new();

        // Try to access without auth
        let result = agent.send_message_unauthenticated(test_message());

        assert!(matches!(result, Err(Error::Unauthorized)));
    }

    #[test]
    fn test_resource_limits_enforced() {
        let agent = TestAgent::new();

        // Try to exceed memory limit
        let result = agent.allocate_memory(100 * 1024 * 1024); // 100MB

        assert!(matches!(result, Err(Error::MemoryLimitExceeded)));
    }

    #[test]
    fn test_sql_injection_prevented() {
        let input = "'; DROP TABLE users; --";

        let result = process_user_input(input);

        // Should sanitize, not execute
        assert!(result.is_ok());
        assert!(!result.unwrap().contains("DROP"));
    }
}
```

## Security Checklist

Before deploying to production:

### Configuration Agents

- [ ] Agent YAML schemas validated
- [ ] Tool allowlists properly configured
- [ ] Memory scopes appropriately set
- [ ] Capability declarations verified
- [ ] Rate limiting configured
- [ ] MCP servers properly sandboxed

### System-wide Security

- [ ] API authentication configured
- [ ] Audit logging active
- [ ] Secrets externalized
- [ ] Input validation implemented
- [ ] Security tests passing
- [ ] Certificates not expired
- [ ] RBAC policies defined
- [ ] Memory encryption enabled
- [ ] Capability-based routing configured

## References

- [ADR-0016: Security Architecture](../adrs/0016-security-architecture.md)
- [DevOps Security Guide](../operations/devops-security-guide.md)
- [Testing Strategy](../development/testing-strategy.md)
- [Operational Runbook](../operations/operational-runbook.md)
