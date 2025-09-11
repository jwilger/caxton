---
title: "ADR-0016: Security Architecture"
date: 2025-08-09
status: accepted
layout: adr
categories: [Architecture]
---


## Status

Accepted

## Context

Caxton's distributed architecture requires comprehensive security measures to
protect agent communications, prevent unauthorized access, and ensure data
confidentiality. With the coordination-first architecture (ADR-0014) and
distributed protocols (ADR-0015), we need clear security boundaries and
authentication mechanisms.

The security model must balance strong protection with operational simplicity,
avoiding complexity that would violate our minimal core philosophy (ADR-0004).

## Decision

### Security Layers

#### 1. Inter-Node Communication (mTLS)

All communication between Caxton instances uses mutual TLS authentication with:

- Unique certificates per node for identity verification
- Certificate authority validation of all peers
- Automatic certificate rotation to maintain security
- Peer identity verification against expected node identifiers

#### 2. Agent Authentication

Agents are authenticated using capability-based security with:

- Unique agent identifiers for individual authentication
- Cryptographically signed capabilities defining permitted operations
- Deployment signatures linking agents to authorized deployers
- Optional tenant isolation for multi-tenancy scenarios
- Time-limited capability tokens with expiration enforcement

#### 3. Agent-to-Agent Authorization

Fine-grained authorization using capability tokens with:

- Allow-lists controlling which agents can communicate
- Operation-specific permissions based on capability tokens
- Per-sender rate limiting to prevent abuse
- Optional message signing requirements for high-security scenarios
- Real-time authorization checks during message routing

### Secret Management

#### 1. Node Secrets

Node-level secrets including TLS certificates, private keys, and CA certificates
are managed through:

- File-based certificate storage with configurable paths
- Automatic certificate rotation with configurable thresholds
- Secure key generation and renewal processes

#### 2. Agent Secrets

Agent secrets are accessed indirectly through MCP (Model Context Protocol) tools
to maintain isolation:

- Agents cannot directly access the host secret store
- All secret access is mediated through MCP tool interfaces
- Automatic secret rotation capabilities
- Audit logging of all secret access attempts

### Network Security

#### 1. SWIM Gossip Encryption

SWIM protocol gossip messages are protected through:

- Symmetric encryption of all gossip payloads
- Periodic key rotation to limit exposure
- Message authentication codes (MAC) to verify integrity
- Nonce-based encryption to prevent replay attacks
- Key versioning to support rotation without service interruption

#### 2. Transport Security

- All external APIs use TLS 1.3 minimum
- HTTP/2 with ALPN negotiation
- Optional QUIC transport for better performance
- Configurable cipher suites (secure defaults)

### Access Control

#### 1. Management API Authentication

The management API supports multiple authentication methods:

- **API Keys**: Long-lived tokens for service-to-service authentication
- **JWT Tokens**: Short-lived tokens with cryptographic verification
- **Client Certificates**: mTLS-based authentication for high-security
  environments
- **OAuth2 Integration**: External identity provider support for enterprise
  scenarios

Authentication method selection is configurable per deployment environment.

#### 2. Role-Based Access Control (RBAC)

Access control uses a role-based system with:

- **Agent Management**: Deploy, remove, and update agent permissions
- **System Management**: Metrics viewing, configuration changes, cluster
  operations
- **Message Operations**: Send, read, and trace message permissions
- **Context-Based Checks**: Resource-specific access validation
- **Principle of Least Privilege**: Minimal permissions granted by default

### Audit Logging

Security audit logs capture comprehensive security events with:

- **Authentication Events**: Login attempts, token operations, session
  management
- **Authorization Events**: Access grants/denials, privilege changes
- **Agent Events**: Deployment, removal, and compromise detection
- **System Events**: Configuration changes, policy updates, certificate
  operations
- **Complete Context**: Identity, timestamp, source location, and outcome
  tracking
- **Structured Format**: Machine-readable logs for SIEM integration

## Implementation Details

### Certificate Management

Certificate lifecycle management includes:

- **CA Initialization**: One-time certificate authority setup
- **Node Certificate Generation**: Per-instance certificate creation
- **Automatic Renewal**: Proactive certificate rotation before expiry
- **Integration Tools**: CLI commands for certificate operations

### Security Configuration

Security features are configurable through YAML configuration covering:

- **Inter-node Security**: mTLS configuration, peer verification, gossip
  encryption
- **API Security**: TLS settings, authentication methods, rate limiting
- **Agent Security**: Deployment signing, capability enforcement, sandboxing
- **Audit Logging**: Event capture, retention policies, SIEM integration

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

## Security Implementation Requirements

Deployment security requires:

- Certificate authority establishment and node certificate distribution
- Inter-node mTLS configuration and gossip encryption enablement
- API authentication setup and RBAC policy configuration
- Audit logging activation and certificate rotation testing
- Network security verification and operational procedure documentation

## References

- [mTLS Best Practices](https://www.cncf.io/blog/2021/07/13/mtls-in-practice/)
- [NIST Cybersecurity Framework](https://www.nist.gov/cyberframework)
- [OWASP Security Guidelines](https://owasp.org/www-project-application-security-verification-standard/)
- [ADR-0004: Minimal Core Philosophy](0004-minimal-core-philosophy.md)
- [ADR-0015: Distributed Protocol Architecture](0015-distributed-protocol-architecture.md)
