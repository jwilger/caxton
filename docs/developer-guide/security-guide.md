---
title: "Security Developer Guide"
date: 2025-01-14
layout: page
categories: [Developer Guide, Security]
---

This guide covers security features and best practices for developing with
Caxton. For the complete security architecture, see
[ADR-0016: Security Architecture](../adr/0016-security-architecture.md).

## Overview

Caxton implements defense-in-depth with multiple security layers:

1. **Inter-node Security**: mTLS for cluster communication
2. **Agent Isolation**: WebAssembly sandboxing
3. **Message Security**: Authentication and authorization
4. **API Security**: Multiple authentication methods
5. **Audit Logging**: Comprehensive security event tracking

## Agent Security

### WebAssembly Isolation

Every agent runs in a secure WebAssembly sandbox:

```rust
// Agent configuration with security limits
let config = AgentConfig {
    // Memory limits prevent exhaustion
    memory_limit: 50 * 1024 * 1024,  // 50MB

    // CPU limits prevent infinite loops
    cpu_time_limit: Duration::from_secs(10),

    // Network access disabled by default
    network_access: false,

    // Filesystem access restricted
    filesystem_access: FileSystemAccess::None,
};
```

### Agent Capabilities

Agents declare their required capabilities:

```rust
// In your agent code
#[agent_capabilities]
pub fn declare_capabilities() -> Vec<Capability> {
    vec![
        Capability::MessageSend,
        Capability::MessageReceive,
        Capability::ToolAccess("database"),
        Capability::TopicSubscribe("events"),
    ]
}

// Capabilities are enforced at runtime
pub fn handle_message(msg: Message) -> Result<Response> {
    // This will fail if agent lacks database capability
    let data = query_database("SELECT * FROM users")?;

    Ok(Response::new(data))
}
```

### Secure Agent Deployment

Sign and verify agents before deployment:

```bash
# Sign your agent with your private key
caxton agent sign \
  --wasm my-agent.wasm \
  --key ~/.caxton/keys/developer.key \
  --output my-agent-signed.wasm

# Deploy with signature verification
caxton deploy my-agent-signed.wasm \
  --verify-signature \
  --trusted-keys /etc/caxton/trusted-keys/
```

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

Control which agents can communicate:

```yaml
# Agent authorization policy
authorization:
  agent_id: order-processor

  # Who can send messages to this agent
  allowed_senders:
    - inventory-manager
    - payment-processor
    - pattern: "customer-*"  # Wildcard patterns

  # Rate limiting per sender
  rate_limits:
    default: 100/minute
    payment-processor: 1000/minute

  # Required message signatures
  require_signatures: true
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

- [ ] All agents signed and verified
- [ ] mTLS enabled for cluster communication
- [ ] API authentication configured
- [ ] Rate limiting enabled
- [ ] Audit logging active
- [ ] Secrets externalized
- [ ] Resource limits set
- [ ] Input validation implemented
- [ ] Security tests passing
- [ ] Certificates not expired
- [ ] RBAC policies defined
- [ ] Network policies configured

## References

- [ADR-0016: Security Architecture](../adr/0016-security-architecture.md)
- [DevOps Security Guide](../operations/devops-security-guide.md)
- [Testing Strategy](../development/testing-strategy.md)
- [Operational Runbook](../operations/operational-runbook.md)
