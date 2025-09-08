---
title: "0023. HTTPS/SSL Reverse Proxy Pattern"
date: 2025-09-04
status: accepted
layout: adr
categories: [Security, Deployment, Infrastructure]
---


Date: 2025-09-04

## Status

Accepted

## Context

The Caxton web admin console requires secure HTTPS access in production
deployments. During Story 006 development, we initially planned to implement
HTTPS/SSL termination directly within the Caxton application server.

**Original Approach**: Implement TLS certificate management, HTTPS
enforcement, and SSL termination within the Caxton binary using Rust TLS
libraries.

**Industry Reality**: Modern web applications universally delegate SSL
termination to reverse proxy infrastructure, focusing application logic on
business functionality rather than transport security.

**Operational Precedent**: Django, Rails, Node.js, Spring Boot, and
virtually all production web applications serve HTTP behind SSL-terminating
reverse proxies.

## Decision Drivers

- **Separation of Concerns**: Application logic should focus on domain
  functionality, not transport security
- **Operational Simplicity**: Certificate management is better handled by
  specialized infrastructure tools
- **Industry Standards**: Following established production deployment
  patterns reduces operational complexity
- **Infrastructure Flexibility**: Reverse proxies provide load balancing,
  caching, and security features beyond SSL
- **Certificate Management**: Automated certificate renewal (Let's Encrypt)
  is standardized in reverse proxy solutions
- **Deployment Compatibility**: Containerized deployments, cloud
  platforms, and CDNs all expect HTTP applications

## Decision

Caxton will **serve HTTP only** and rely on reverse proxy infrastructure
for HTTPS/SSL termination in production deployments.

### Application Architecture

**HTTP-Only Application Server**:

```rust
// Caxton serves plain HTTP - SSL handled by reverse proxy
let app = Router::new()
    .route("/api/v1/*path", api_routes)
    .route_service("/", serve_static_files)
    .layer(
        ServiceBuilder::new()
            .layer(SessionLayer::new(session_store, session_key))
            .layer(CsrfLayer::new())
            .layer(TrustProxyHeadersLayer::new()) // Trust X-Forwarded-* headers
    );

axum::Server::bind(
        &format!("0.0.0.0:{}", config.web_port).parse()?
    )
    .serve(app.into_make_service())
    .await?;
```

**Configuration Simplification**:

```rust
#[derive(Debug, Clone)]
pub struct WebConsoleConfig {
    pub web_port: u16,              // HTTP port (e.g., 8080)
    pub trust_reverse_proxy: bool,  // Trust X-Forwarded-* headers
    pub session_secret: String,     // Secure session key
    // TLS configuration removed - handled by reverse proxy
}
```

### Deployment Pattern

**Standard Production Setup**:

```text
Internet → Reverse Proxy (nginx/Caddy) → Caxton HTTP Server
           ├─ SSL Termination
           ├─ Certificate Management
           ├─ HTTPS → HTTP forwarding
           └─ Security Headers
```

## Implementation Details

### Reverse Proxy Integration

**Trust Proxy Headers**:

```rust
// Middleware to trust X-Forwarded-* headers from reverse proxy
pub struct TrustProxyHeadersLayer {
    trusted_proxies: Vec<IpAddr>,
}

impl TrustProxyHeadersLayer {
    pub fn new() -> Self {
        // Trust localhost and private network ranges
        Self {
            trusted_proxies: vec![
                "127.0.0.1".parse().unwrap(),
                "::1".parse().unwrap(),
                // Add Docker/Kubernetes network ranges as needed
            ],
        }
    }
}
```

**Secure Session Cookies**:

```rust
// Sessions work correctly behind HTTPS reverse proxy
SessionLayer::new(session_store, session_key)
    .with_secure(true)     // Secure flag when behind HTTPS proxy
    .with_http_only(true)  // Prevent XSS access
    .with_same_site(SameSite::Strict)
```

### Example Reverse Proxy Configurations

**nginx Configuration**:

```nginx
server {
    listen 443 ssl http2;
    server_name caxton.example.com;

    # SSL certificate management
    ssl_certificate \
        /etc/letsencrypt/live/caxton.example.com/fullchain.pem;
    ssl_certificate_key \
        /etc/letsencrypt/live/caxton.example.com/privkey.pem;

    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;

    # Proxy to Caxton HTTP server
    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For \
            $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket upgrade for SSE
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}

# HTTP to HTTPS redirect
server {
    listen 80;
    server_name caxton.example.com;
    return 301 https://$server_name$request_uri;
}
```

**Caddy Configuration** (automatic HTTPS):

```caddyfile
caxton.example.com {
    reverse_proxy localhost:8080

    # Caddy handles SSL certificates automatically
    # Security headers added automatically

    header {
        # Additional security headers if needed
        X-Frame-Options DENY
        X-Content-Type-Options nosniff
    }
}
```

**Docker Compose Example**:

```yaml
version: '3.8'
services:
  caddy:
    image: caddy:2-alpine
    ports:
      - "80:80"
      - "443:443"
    volumes:
      - ./Caddyfile:/etc/caddy/Caddyfile
      - caddy_data:/data
      - caddy_config:/config
    depends_on:
      - caxton

  caxton:
    image: caxton:latest
    command: ["server", "--admin-console", "--web-port", "8080"]
    expose:
      - "8080"
    volumes:
      - ./caxton-config.toml:/etc/caxton/config.toml
    environment:
      - CAXTON_WEB_TRUST_REVERSE_PROXY=true

volumes:
  caddy_data:
  caddy_config:
```

### Security Considerations

**Proxy Trust Validation**:

- Only trust X-Forwarded-* headers from known proxy IP addresses
- Validate proxy configuration to prevent header injection
- Monitor for direct access bypassing reverse proxy

**Certificate Management**:

- Reverse proxy handles certificate renewal automatically
- No application downtime for certificate updates
- Centralized SSL configuration across services

**Header Security**:

- Security headers (HSTS, CSP, etc.) set by reverse proxy
- Consistent security policy across applications
- Framework-agnostic security implementation

## Benefits

### Operational Advantages

- **Simplified Application**: Caxton focuses on domain logic, not
  transport security
- **Certificate Automation**: Let's Encrypt integration handled by proxy
  (nginx/Caddy)
- **Zero Downtime Updates**: Certificate renewal without application restart
- **Infrastructure Flexibility**: Standard deployment pattern for containers,
  cloud, CDNs
- **Proven Security**: Battle-tested SSL implementations in nginx/Caddy vs.
  custom Rust TLS

### Development Benefits

- **Reduced Complexity**: Eliminates TLS certificate handling from
  application code
- **Faster Development**: No need to implement certificate validation,
  renewal, or rotation
- **Better Testing**: Local development uses HTTP, production patterns
  tested via proxy
- **Standard Patterns**: Follows established practices from Django, Rails,
  Spring Boot

### Security Benefits

- **Defense in Depth**: Proxy provides additional security layer beyond
  application
- **Specialized Tools**: nginx/Caddy optimized for TLS performance and security
- **Centralized Policy**: SSL configuration managed independently of application
- **Attack Surface**: Reduces application attack surface by delegating
  transport security

## Consequences

### Positive

- **Industry Standard**: Follows universal production deployment pattern
- **Operational Simplicity**: Certificate management handled by
  specialized tools
- **Development Focus**: Application development concentrates on business logic
- **Infrastructure Agnostic**: Works with any reverse proxy, load balancer,
  or CDN
- **Security Excellence**: Proven TLS implementations vs. custom
  application code
- **Deployment Flexibility**: Standard pattern for containers, Kubernetes,
  cloud platforms

### Negative

- **Infrastructure Dependency**: Requires reverse proxy for production HTTPS
- **Configuration Complexity**: Must configure both application and
  proxy correctly
- **Trust Relationship**: Application must trust proxy headers for
  client IP detection
- **Development-Production Gap**: Local HTTP vs. production HTTPS behavior
  differences

### Mitigation Strategies

**Infrastructure Documentation**:

- Provide example configurations for nginx, Caddy, Apache
- Document container deployment patterns
- Include cloud platform deployment guides

**Configuration Validation**:

- Validate proxy header trust configuration
- Health checks verify proxy-to-application communication
- Monitoring detects direct access bypassing proxy

**Development-Production Parity**:

- Local development proxy setup for testing HTTPS behavior
- Environment-specific configuration for proxy trust settings
- Integration tests verify proxy header handling

## Related Decisions

- ADR-0022: Web-based Admin Console - Established HTTP application server
  architecture
- ADR-0016: Security Architecture - Defines overall security approach for
  Caxton
- ADR-0006: Application Server Architecture - Established Caxton as
  standalone server
- ADR-0004: Minimal Core Philosophy - Avoiding unnecessary complexity in
  application logic

## References

- [nginx SSL Configuration](
  https://nginx.org/en/docs/http/configuring_https_servers.html)
- [Caddy Automatic HTTPS](https://caddyserver.com/docs/automatic-https)
- [Let's Encrypt Certificate Authority](https://letsencrypt.org/)
- [OWASP Transport Layer Security](
  https://owasp.org/www-community/controls/Transport_Layer_Security_Cheat_Sheet)
- [12-Factor App: Port Binding](https://12factor.net/port-binding) -
  Applications export HTTP as a service
