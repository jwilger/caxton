---
layout: adr
title: "0016. Security Architecture"
status: accepted
date: 2025-08-09
---

# ADR-0016: Security Architecture

## Status
Accepted

## Context
Caxton's distributed architecture requires comprehensive security measures to protect agent communications, prevent unauthorized access, and ensure data confidentiality. With the coordination-first architecture (ADR-0014) and distributed protocols (ADR-0015), we need clear security boundaries and authentication mechanisms.

The security model must balance strong protection with operational simplicity, avoiding complexity that would violate our minimal core philosophy (ADR-0004).

## Decision

### Security Layers

#### 1. Inter-Node Communication (mTLS)
All communication between Caxton instances uses mutual TLS authentication:

```rust
pub struct NodeSecurity {
    // Each node has a unique certificate
    node_cert: Certificate,
    node_key: PrivateKey,

    // CA certificate for validating peers
    ca_cert: Certificate,

    // Certificate rotation support
    cert_refresh_interval: Duration,
}

impl NodeSecurity {
    pub async fn establish_secure_connection(&self, peer: &NodeId) -> Result<SecureChannel> {
        let tls_config = rustls::ClientConfig::builder()
            .with_safe_defaults()
            .with_root_certificates(self.ca_cert.clone())
            .with_client_cert_resolver(Arc::new(self.node_cert.clone()));

        // Verify peer certificate CN matches expected node ID
        let connection = TlsConnector::new(tls_config)
            .connect(peer.address())
            .await?;

        self.verify_peer_identity(&connection, peer)?;
        Ok(SecureChannel::new(connection))
    }
}
```

#### 2. Agent Authentication
Agents are authenticated using capability-based security:

```rust
pub struct AgentIdentity {
    // Unique agent identifier
    agent_id: AgentId,

    // Signed capabilities (what the agent can do)
    capabilities: SignedCapabilities,

    // Deployment signature (who deployed it)
    deployment_auth: DeploymentSignature,

    // Optional: tenant/namespace isolation
    tenant_id: Option<TenantId>,
}

pub struct SignedCapabilities {
    capabilities: Vec<Capability>,
    signature: Ed25519Signature,
    expires_at: Timestamp,
}
```

#### 3. Agent-to-Agent Authorization
Fine-grained authorization using capability tokens:

```rust
pub struct AuthorizationPolicy {
    // Who can send messages to this agent
    allowed_senders: AllowList,

    // What operations are permitted
    permitted_operations: Vec<Operation>,

    // Rate limiting per sender
    rate_limits: HashMap<AgentId, RateLimit>,
}

impl MessageRouter {
    pub async fn authorize_message(&self, msg: &FipaMessage) -> Result<()> {
        let sender_caps = self.get_agent_capabilities(&msg.sender)?;
        let receiver_policy = self.get_agent_policy(&msg.receiver)?;

        // Check if sender is authorized
        if !receiver_policy.allows_sender(&msg.sender, &sender_caps) {
            return Err(SecurityError::Unauthorized);
        }

        // Check rate limits
        if !self.rate_limiter.check(&msg.sender, &msg.receiver).await? {
            return Err(SecurityError::RateLimitExceeded);
        }

        // Verify message signature if required
        if receiver_policy.requires_signed_messages() {
            self.verify_message_signature(msg)?;
        }

        Ok(())
    }
}
```

### Secret Management

#### 1. Node Secrets
```yaml
# /etc/caxton/node-security.yaml
node:
  certificate_path: /etc/caxton/certs/node.crt
  private_key_path: /etc/caxton/certs/node.key
  ca_certificate_path: /etc/caxton/certs/ca.crt

  # Automatic rotation
  rotation:
    enabled: true
    check_interval: 24h
    renewal_threshold: 30d
```

#### 2. Agent Secrets (via MCP Tools)
```rust
pub trait SecretProvider: Send + Sync {
    async fn get_secret(&self, key: &str) -> Result<SecretValue>;
    async fn rotate_secret(&self, key: &str) -> Result<SecretValue>;
}

// Agents access secrets through MCP tools, not directly
impl Agent {
    pub async fn get_api_key(&self, service: &str) -> Result<String> {
        self.mcp_tools
            .secret_provider()
            .get_secret(&format!("{}/api_key", service))
            .await
    }
}
```

### Network Security

#### 1. SWIM Gossip Encryption
```rust
pub struct SecureGossip {
    // Encrypt gossip payloads
    encryption_key: SymmetricKey,

    // Rotate keys periodically
    key_rotation_interval: Duration,

    // Verify message authenticity
    mac_key: HmacKey,
}

impl SecureGossip {
    pub fn encrypt_gossip(&self, data: &GossipData) -> Result<EncryptedGossip> {
        let nonce = generate_nonce();
        let ciphertext = self.encryption_key.encrypt(data, &nonce)?;
        let mac = self.mac_key.sign(&ciphertext)?;

        Ok(EncryptedGossip {
            ciphertext,
            nonce,
            mac,
            key_version: self.current_key_version(),
        })
    }
}
```

#### 2. Transport Security
- All external APIs use TLS 1.3 minimum
- HTTP/2 with ALPN negotiation
- Optional QUIC transport for better performance
- Configurable cipher suites (secure defaults)

### Access Control

#### 1. Management API Authentication
```rust
pub enum AuthMethod {
    // API key authentication
    ApiKey(String),

    // JWT tokens
    JWT(JsonWebToken),

    // mTLS client certificates
    ClientCertificate(Certificate),

    // OAuth2 integration
    OAuth2(OAuth2Token),
}

impl ManagementApi {
    pub async fn authenticate(&self, req: &Request) -> Result<Identity> {
        match self.extract_auth_method(req)? {
            AuthMethod::ApiKey(key) => self.validate_api_key(key).await,
            AuthMethod::JWT(token) => self.validate_jwt(token).await,
            AuthMethod::ClientCertificate(cert) => self.validate_client_cert(cert).await,
            AuthMethod::OAuth2(token) => self.validate_oauth2(token).await,
        }
    }
}
```

#### 2. Role-Based Access Control (RBAC)
```rust
pub struct Role {
    name: String,
    permissions: Vec<Permission>,
}

pub enum Permission {
    // Agent management
    DeployAgent,
    RemoveAgent,
    UpdateAgent,

    // System management
    ViewMetrics,
    ModifyConfig,
    ManageCluster,

    // Message operations
    SendMessage,
    ReadMessages,
    TraceMessages,
}

impl AuthorizationService {
    pub fn authorize(&self, identity: &Identity, action: &Action) -> Result<()> {
        let role = self.get_role(identity)?;

        if !role.has_permission(action.required_permission()) {
            return Err(SecurityError::InsufficientPermissions);
        }

        // Additional context-based checks
        self.check_resource_access(identity, action.resource())?;

        Ok(())
    }
}
```

### Audit Logging

```rust
pub struct SecurityAuditLog {
    // What happened
    event_type: SecurityEventType,

    // Who did it
    identity: Identity,

    // When it happened
    timestamp: Timestamp,

    // Where it happened
    source_ip: IpAddr,
    node_id: NodeId,

    // Result
    outcome: Outcome,

    // Additional context
    metadata: HashMap<String, Value>,
}

pub enum SecurityEventType {
    // Authentication events
    LoginSuccess,
    LoginFailure,
    TokenRefresh,

    // Authorization events
    AccessGranted,
    AccessDenied,
    PrivilegeEscalation,

    // Agent events
    AgentDeployed,
    AgentRemoved,
    AgentCompromised,

    // System events
    ConfigurationChanged,
    SecurityPolicyUpdated,
    CertificateRotated,
}
```

## Implementation Details

### Certificate Management

```bash
# Generate CA certificate (one-time setup)
caxton security init-ca \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key

# Generate node certificate
caxton security gen-cert \
  --node-id node-1 \
  --ca-cert /etc/caxton/ca.crt \
  --ca-key /etc/caxton/ca.key \
  --output /etc/caxton/certs/

# Automatic renewal
caxton security renew-cert \
  --before-expiry 30d
```

### Security Configuration

```yaml
security:
  # Inter-node security
  cluster:
    mtls:
      enabled: true
      ca_cert: /etc/caxton/ca.crt
      node_cert: /etc/caxton/certs/node.crt
      node_key: /etc/caxton/certs/node.key
      verify_peer: true

    gossip:
      encryption: true
      key_rotation_interval: 24h

  # API security
  api:
    tls:
      enabled: true
      cert: /etc/caxton/api.crt
      key: /etc/caxton/api.key
      min_version: "1.3"

    authentication:
      methods:
        - api_key
        - jwt
      jwt:
        issuer: "https://auth.example.com"
        audience: "caxton-api"

    rate_limiting:
      enabled: true
      default_limit: 1000/min

  # Agent security
  agents:
    require_signed_deployment: true
    capability_enforcement: strict
    sandbox_enforcement: strict

  # Audit logging
  audit:
    enabled: true
    log_path: /var/log/caxton/audit.log
    retention_days: 90
    forward_to_siem: true
```

## Security Threat Model

### Threats Addressed

1. **Man-in-the-Middle**: mTLS prevents interception
2. **Unauthorized Access**: Authentication and RBAC
3. **Agent Compromise**: Capability-based security limits blast radius
4. **Data Exfiltration**: Encryption at rest and in transit
5. **Denial of Service**: Rate limiting and resource limits
6. **Replay Attacks**: Nonce-based message verification

### Defense in Depth

1. **Network Layer**: TLS, firewall rules
2. **Authentication Layer**: mTLS, API keys, JWT
3. **Authorization Layer**: RBAC, capability tokens
4. **Application Layer**: Input validation, rate limiting
5. **Runtime Layer**: WASM sandboxing, resource limits

## Consequences

### Positive
- **Strong Security**: Multiple layers of protection
- **Zero-Trust Model**: No implicit trust between components
- **Audit Trail**: Complete security event logging
- **Flexible Authentication**: Multiple auth methods supported
- **Minimal Attack Surface**: WASM isolation limits damage

### Negative
- **Operational Complexity**: Certificate management required
- **Performance Overhead**: TLS and encryption costs
- **Initial Setup**: CA and certificate generation needed

### Neutral
- Industry-standard security practices
- Similar to Kubernetes/Consul security models
- Requires security-conscious deployment

## Alternatives Considered

### Plain TCP with Application-Level Security
- **Pros**: Simpler, potentially faster
- **Cons**: Vulnerable to network attacks
- **Decision**: mTLS provides stronger guarantees

### Shared Secret Authentication
- **Pros**: Simple to implement
- **Cons**: Secret distribution problem
- **Decision**: PKI provides better security

### External Auth Service (OAuth2/OIDC)
- **Pros**: Centralized auth, enterprise integration
- **Cons**: External dependency
- **Decision**: Violates minimal core philosophy

## Security Checklist

- [ ] Generate CA certificate
- [ ] Generate node certificates for each instance
- [ ] Configure mTLS between nodes
- [ ] Enable gossip encryption
- [ ] Set up API authentication
- [ ] Configure RBAC policies
- [ ] Enable audit logging
- [ ] Test certificate rotation
- [ ] Verify firewall rules
- [ ] Document security procedures

## References
- [mTLS Best Practices](https://www.cncf.io/blog/2021/07/13/mtls-in-practice/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [OWASP Security Guidelines](https://owasp.org/www-project-application-security-verification-standard/)
- [ADR-0004: Minimal Core Philosophy](0004-minimal-core-philosophy.md)
- [ADR-0015: Distributed Protocol Architecture](0015-distributed-protocol-architecture.md)
