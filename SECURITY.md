# Security Policy

## Table of Contents

- [Overview](#overview)
- [Security Architecture](#security-architecture)
  - [Hybrid Trust Model](#hybrid-trust-model)
  - [MCP Tool Sandboxing](#mcp-tool-sandboxing)
  - [Configuration Security](#configuration-security)
  - [Embedded Database Security](#embedded-database-security)
  - [REST API Security](#rest-api-security)
- [Security Guarantees](#security-guarantees)
- [Threat Model](#threat-model)
- [Security Controls](#security-controls)
  - [Development Security](#development-security)
  - [Build Security](#build-security)
  - [Runtime Security](#runtime-security)
- [Vulnerability Management](#vulnerability-management)
  - [Reporting Security Vulnerabilities](#reporting-security-vulnerabilities)
  - [Response Timeline](#response-timeline)
  - [Severity Classification](#severity-classification)
- [Security Testing](#security-testing)
- [RFC 9116 Security.txt Best Practices](#rfc-9116-securitytxt-best-practices)
- [Compliance](#compliance)
- [Security Resources](#security-resources)
- [Security Contacts](#security-contacts)

## Overview

Caxton is a configuration-driven multi-agent platform that prioritizes security through
a hybrid trust model: trusted configuration agents run in the host runtime for performance,
while untrusted MCP tools are isolated in WebAssembly sandboxes. This document outlines
our security practices, vulnerability reporting procedures, and security
guarantees for our single-binary, zero-dependency deployment architecture.

## Security Architecture

### Hybrid Trust Model

Caxton employs a hybrid security architecture:

- **Configuration Agents**: TOML-based agents run in the host runtime for
  performance, representing the primary trusted boundary
- **MCP Tool Isolation**: Model Context Protocol tools execute in WebAssembly
  sandboxes with strict resource and capability limits
- **Single Binary Deployment**: Zero external dependencies reduce attack surface
  and eliminate supply chain risks from runtime dependencies
- **Embedded Database**: Local SQLite storage eliminates network database
  security concerns

### MCP Tool Sandboxing

MCP tools represent the untrusted boundary in our security model:

- Tools run in isolated WebAssembly environments with no direct system access
- Resource consumption is limited through configurable memory and CPU constraints
- All system interactions are mediated through capability-based APIs
- Network access is restricted and must be explicitly granted per tool

### Configuration Security

TOML configuration files are validated against strict schemas with hot-reload
safety checks. Built-in agent templates undergo cryptographic verification to
prevent tampering.

### API and Database Security

The REST management API requires authentication and implements role-based
authorization with rate limiting. The embedded SQLite database uses prepared
statements and restrictive file permissions.

## Security Guarantees

### MCP Tool Isolation

1. **Memory Safety**: MCP tools cannot access host system memory or other tool
   instances
2. **Resource Limits**: Tools cannot consume more resources than allocated
   through WebAssembly constraints
3. **System Boundaries**: Tools cannot make unauthorized system calls outside
   capability grants
4. **Network Isolation**: Tools have no direct network access unless
   explicitly granted

### Configuration Agent Security

1. **Schema Validation**: All TOML configurations are validated against strict schemas
2. **Hot-Reload Safety**: Configuration changes undergo security validation
   before application
3. **Template Integrity**: Built-in agent templates are cryptographically verified
4. **File System Security**: Configuration files use restrictive permissions

### Platform Security

1. **Least Privilege**: Components run with minimal required privileges
2. **Defense in Depth**: Multiple security layers prevent single points of failure
3. **Fail-Safe Defaults**: Secure defaults are used throughout the system
4. **Security Monitoring**: Comprehensive logging and monitoring detect security
   anomalies

## Threat Model

### In Scope (Our Responsibility)

- Vulnerabilities in Caxton's source code and design
- Insecure default configurations or behaviors
- Dependency vulnerabilities in our build artifacts
- Supply chain security of our development and release process
- Design flaws that could enable privilege escalation or data exposure
- Security issues in our architectural decisions and implementation

### Out of Scope (Deployer Responsibility)

- Operational security of systems running Caxton
- Network infrastructure security where Caxton is deployed
- Physical security of host systems
- Operating system configuration and hardening
- Third-party LLM service security and access controls
- User authentication and authorization policies
- Data protection and encryption at rest
- Compliance with specific regulatory frameworks

**Note**: As per our licensing terms, we assume no responsibility for security
issues arising from deployment, configuration, or operational use of Caxton.
Users are responsible for implementing appropriate security controls for their
specific use cases and environments.

## Security Controls

### Development Security

- **Secure Coding**: Rust memory safety prevents common vulnerability
  classes
- **Dependency Management**: All dependencies are audited for vulnerabilities
  using cargo-deny
- **Static Analysis**: Code undergoes security-focused static analysis
- **Security Testing**: Comprehensive security testing including fuzzing and
  property-based tests

### Build Security

- **Supply Chain Security**: All dependencies are verified and audited before inclusion
- **Reproducible Builds**: Build process ensures consistent, verifiable artifacts
- **Vulnerability Scanning**: Automated scanning for known vulnerabilities in dependencies
- **License Compliance**: All dependencies comply with approved license requirements

### Application Security

- **Memory Safety**: Rust language guarantees prevent entire classes of vulnerabilities
- **Input Validation**: All external inputs undergo strict validation
- **Secure Defaults**: Default configurations prioritize security over convenience
- **Capability-Based Architecture**: Components operate with minimal necessary privileges

## Vulnerability Management

### Supported Versions

We provide security updates for the following versions:

| Version | Supported | | ------- | ------------------ | | 0.1.x |
:white_check_mark: | | < 0.1 | :x: |

### Reporting Security Vulnerabilities

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them by emailing
john+caxton-security-report@johnwilger.com or through GitHub's private
vulnerability reporting feature (preferred).

For automated vulnerability reporting, see our
[security.txt file](.well-known/security.txt) which follows
[RFC 9116](https://tools.ietf.org/rfc/rfc9116.txt) standards.

#### Security.txt Implementation

Our security.txt provides machine-readable security contact information:

- **Primary Contact**: `john+caxton-security-report@johnwilger.com` for
  vulnerability reports
- **Policy Location**: Comprehensive disclosure guidelines at
  `https://jwilger.github.io/caxton/security/policy`
- **Acknowledgments**: Public recognition for responsible disclosure at
  `https://jwilger.github.io/caxton/security/acknowledgments`
- **Expiration**: Annual renewal required (expires 2025-12-31)
- **Language**: English for fastest response times

**Best Practices Implemented:**

- Located at both `/.well-known/security.txt` and `/security.txt` for
  discoverability
- Signed with PGP key for authenticity (planned)
- Regular expiration updates to ensure current contact information
- Clear scope definition in linked policy document

When reporting a vulnerability, please include:

- Description of the vulnerability
- Steps to reproduce (if applicable)
- Potential impact assessment
- Suggested remediation (if any)

### Response Timeline

- **Acknowledgment**: Within 24 hours
- **Initial Assessment**: Within 72 hours
- **Detailed Response**: Within 1 week
- **Fix Timeline**: Varies by severity (see below)

### Severity Classification

#### Critical (Fix within 24 hours)

- Remote code execution
- Authentication bypass
- Data exfiltration
- Complete system compromise

#### High (Fix within 1 week)

- Privilege escalation
- Denial of service
- Information disclosure
- Security control bypass

#### Medium (Fix within 1 month)

- Less severe information disclosure
- Limited privilege escalation
- Security feature weakness

#### Low (Fix in next release)

- Security hardening opportunities
- Defense-in-depth improvements
- Minor information leaks

### Security Update Process

1. **Validation**: Vulnerability is validated and assessed
2. **Fix Development**: Security fix is developed and tested
3. **Security Review**: Fix undergoes security review
4. **Release Preparation**: Release notes and advisories are prepared
5. **Coordinated Disclosure**: Fix is released with security advisory
6. **Post-Release**: Monitoring for successful deployment

## Security Testing

### Automated Testing

Our CI/CD pipeline includes automated security validation on every pull request:

- **Dependency Auditing**: Comprehensive dependency vulnerability scanning
- **License Compliance**: Automated license compatibility verification
- **Supply Chain Security**: Validation of dependency sources and integrity
- **Static Analysis**: Security-focused code analysis
- **Fuzzing**: Automated fuzzing of critical parsing and validation components

### Manual Testing

Regular manual security assessments include:

- **Security-Focused Code Reviews**: All changes undergo security review
- **Threat Model Updates**: Regular review and updates of our threat model
- **Security Architecture Review**: Periodic evaluation of security design decisions

## RFC 9116 Security.txt Best Practices

### Implementation Details

Our security.txt implementation follows RFC 9116 specifications for
machine-readable security policies:

#### Required Fields

- **Contact**: Primary security email
  (`john+caxton-security-report@johnwilger.com`) with 24-hour response
  commitment
- **Expires**: Annual renewal with clear expiration date to ensure current
  information

#### Optional Fields (Enhanced Disclosure)

- **Acknowledgments**: Public recognition page for responsible disclosure
  participants
- **Policy**: Comprehensive vulnerability disclosure policy with scope and safe
  harbor provisions
- **Preferred-Languages**: English prioritized for fastest response times
- **Canonical**: Authoritative location preventing stale security contact
  information

#### Implementation Standards

Our security.txt implementation follows RFC 9116 best practices:

- **Dual Location**: Available at both `/.well-known/security.txt` and `/security.txt`
- **HTTPS Only**: Served exclusively over encrypted connections
- **Regular Updates**: Annual expiration with automated renewal monitoring
- **PGP Signing**: Cryptographic signature verification (planned)
- **Contact Verification**: Regular validation of contact information accessibility

### Integration with Vulnerability Management

Security.txt coordinates with our broader vulnerability management:

1. **Automated Reporting**: Enables security scanners to report findings
   directly
2. **Researcher Onboarding**: Provides clear entry point for security
   researchers
3. **Process Documentation**: Links to detailed disclosure policies and
   procedures
4. **Recognition Program**: Acknowledges responsible disclosure through public
   acknowledgments

## Compliance

Caxton development follows established security frameworks:

- **OWASP Secure Coding Practices**: Applied throughout development
  process
- **CWE/SANS Top 25**: Proactive prevention of common weaknesses
- **NIST Secure Software Development Framework**: Integrated security
  practices

## Security Resources

### Security Documentation

#### Core Security Documentation

- [Architecture Decision Records](/docs/adr/): Security-related architectural decisions
- [deny.toml](deny.toml): Cargo-deny security policy configuration
- [.well-known/security.txt](.well-known/security.txt): RFC 9116 security
  contact information

#### External Security Standards

- [RFC 9116 Security.txt](https://tools.ietf.org/rfc/rfc9116.txt): Security
  contact standard
- [RustSec Advisory Database](https://rustsec.org/): Vulnerability tracking
- [OWASP Application Security](https://owasp.org/): Web application security
  guidance

## Run security checks

## Security Contacts

- **Vulnerability Reports**: john+caxton-security-report@johnwilger.com
- **Security Inquiries**: john+caxton-security-report@johnwilger.com
- **Incident Response**: john+caxton-security-report@johnwilger.com

---

For the latest security information, please visit our [security page](https://jwilger.github.io/caxton/security).
