---
title: "0024. OAuth2 Authentication for Web Admin Console"
date: 2025-09-04
status: accepted
layout: adr
categories: [Security, Authentication, Enterprise]
---


Date: 2025-09-04

## Status

Accepted

## Context

The Caxton web admin console requires secure authentication for
production
deployments. Enterprise environments demand Single Sign-On (SSO)
integration
with existing identity providers, making traditional
username/password
authentication insufficient.

**Enterprise Requirements**:

- SSO integration with corporate identity providers (Okta, Auth0,
  Azure AD)
- Centralized user management without duplicating credentials
- Compliance with organizational security policies
- Support for multi-factor authentication (MFA) through identity
  providers
- Audit trails tied to corporate user accounts

**Security Concerns**:

- Password management introduces security complexity (hashing,
  storage,
  rotation)
- Password-based attacks (brute force, credential stuffing,
  phishing)
- Compliance requirements often prohibit local password storage
- User experience degraded by separate credential management

**Operational Reality**:

- Modern applications delegate authentication to specialized providers
- OAuth2/OIDC is the industry standard for web application
  authentication
- Identity providers offer security expertise that exceeds
  custom
  implementations

## Decision

Caxton will use **OAuth2/OIDC exclusively** for web admin
console
authentication, with **no local password management**.

### Authentication Architecture

**OAuth2/OIDC Flow**:

```rust
pub struct OAuth2Config {
    // OIDC provider configuration
    issuer_url: String,              // e.g., "https://auth.example.com"
    client_id: String,               // Application identifier
    client_secret: String,           // Application secret
    redirect_uri: String,            // Callback URL after authentication

    // Scopes and claims
    scopes: Vec<String>,             // Default: ["openid", "profile", "email"]
    required_claims: Vec<String>,    // Claims required for authorization

    // Session configuration
    session_duration: Duration,      // JWT token lifetime
    refresh_enabled: bool,           // Enable token refresh
}

impl OAuth2AuthProvider {
    pub async fn initiate_login(&self, redirect_url: &str) -> Result<String> {
        let state = generate_secure_state();
        let nonce = generate_secure_nonce();

        let auth_url = self.discovery_document
            .authorization_endpoint
            .clone()
            .query_pairs_mut()
            .append_pair("client_id", &self.config.client_id)
            .append_pair("response_type", "code")
            .append_pair("scope", &self.config.scopes.join(" "))
            .append_pair("redirect_uri", &self.config.redirect_uri)
            .append_pair("state", &state)
            .append_pair("nonce", &nonce)
            .finish()
            .to_string();

        self.store_auth_state(&state, &nonce, redirect_url).await?;
        Ok(auth_url)
    }

    pub async fn handle_callback(
        &self,
        code: &str,
        state: &str,
    ) -> Result<UserSession> {
        self.validate_auth_state(state).await?;

        let token_response = self.exchange_code_for_token(code).await?;
        let user_claims = self
            .validate_and_decode_token(&token_response.id_token)
            .await?;

        self.authorize_user(&user_claims).await?;

        Ok(UserSession {
            user_id: user_claims.sub,
            email: user_claims.email,
            name: user_claims.name,
            expires_at: user_claims.exp,
            access_token: token_response.access_token,
        })
    }
}
```

### Bootstrap Process

**Initial Setup with ROOT_PASSWORD**:

```rust
pub enum AuthenticationMethod {
    // Bootstrap mode - environment variable only
    RootPassword {
        enabled: bool,
        configured: bool,  // Disabled once OAuth2 is configured
    },

    // Production authentication
    OAuth2 {
        provider: OAuth2Config,
        enabled: bool,
    },
}

impl AuthenticationService {
    pub async fn authenticate(
        &self,
        request: &AuthRequest,
    ) -> Result<AuthResult> {
        match &self.method {
            AuthenticationMethod::RootPassword {
                enabled: true,
                configured: false,
            } => {
                // Only during initial setup
                self.validate_root_password(&request.password).await
            }

            AuthenticationMethod::OAuth2 { provider, enabled: true } => {
                // Production authentication
                self.validate_oauth2_token(&request.token, provider).await
            }

            _ => Err(AuthError::NoValidAuthMethod)
        }
    }

    pub async fn configure_oauth2(
        &mut self,
        config: OAuth2Config,
    ) -> Result<()> {
        // Validate configuration by testing OIDC discovery
        self.validate_oidc_provider(&config).await?;

        // Switch to OAuth2 authentication
        self.method = AuthenticationMethod::OAuth2 {
            provider: config,
            enabled: true,
        };

        // Disable root password
        if let AuthenticationMethod::RootPassword { configured, .. } =
            &mut self.method
        {
            *configured = true;
        }

        // Persist configuration
        self.save_config().await?;

        Ok(())
    }
}
```

**Configuration Flow**:

1. **Initial Access**: Admin accesses console with `ROOT_PASSWORD`
   environment variable
2. **OAuth2 Setup**: Admin configures OAuth2 provider through web interface
3. **Validation**: System validates OIDC discovery and test authentication
4. **Activation**: OAuth2 enabled, root password authentication disabled
5. **Production**: All future authentication via OAuth2/OIDC

### Supported Identity Providers

**Enterprise Providers**:

- **Okta**: `https://company.okta.com/.well-known/openid_configuration`
- **Auth0**: `https://company.us.auth0.com/.well-known/openid_configuration`
- **Azure AD**:
  `https://login.microsoftonline.com/tenant-id/v2.0/
  .well-known/openid_configuration`
- **Google Workspace**:
  `https://accounts.google.com/.well-known/openid_configuration`

**Development Providers**:

- **GitHub**: OAuth2 (not OIDC, but supported)
- **GitLab**: `https://gitlab.com/.well-known/openid_configuration`
- **Keycloak**: Self-hosted OIDC provider

**Provider Detection**:

```rust
pub enum IdentityProvider {
    Okta { domain: String },
    Auth0 { domain: String },
    AzureAD { tenant_id: String },
    Google,
    GitHub,
    GitLab,
    Generic { discovery_url: String },
}

impl IdentityProvider {
    pub fn discovery_url(&self) -> String {
        match self {
            Self::Okta { domain } => {
                format!("https://{domain}/.well-known/openid_configuration")
            }
            Self::Auth0 { domain } =>
                format!("https://{domain}/.well-known/openid_configuration"),
            Self::AzureAD { tenant_id } => format!(
                "https://login.microsoftonline.com/{tenant_id}/v2.0/
                .well-known/openid_configuration"),
            Self::Google =>
                "https://accounts.google.com/.well-known/openid_configuration"
                .to_string(),
            Self::Generic { discovery_url } => discovery_url.clone(),
            _ => panic!("Provider does not support OIDC discovery"),
        }
    }
}
```

### Security Implementation

**JWT Token Validation**:

```rust
pub struct TokenValidator {
    // JWKS (JSON Web Key Set) from provider
    jwks: JsonWebKeySet,

    // Required claims for authorization
    required_issuer: String,
    required_audience: String,

    // Validation settings
    clock_skew_tolerance: Duration,
    max_token_age: Duration,
}

impl TokenValidator {
    pub async fn validate_id_token(&self, token: &str) -> Result<UserClaims> {
        let header = self.decode_header(token)?;
        let key = self.jwks.find_key(&header.kid)?;

        let claims: UserClaims = jsonwebtoken::decode(
            token,
            &key.decode_key()?,
            &Validation {
                iss: Some(self.required_issuer.clone()),
                aud: Some(self.required_audience.clone()),
                leeway: self.clock_skew_tolerance.as_secs(),
                ..Default::default()
            }
        )?.claims;

        // Additional validation
        self.validate_claims(&claims)?;

        Ok(claims)
    }

    pub async fn refresh_jwks(&mut self) -> Result<()> {
        let response = reqwest::get(&self.jwks_uri).await?;
        self.jwks = response.json().await?;
        Ok(())
    }
}
```

**Authorization Integration**:

```rust
pub struct OAuth2Authorization {
    // Map OAuth2 claims to Caxton permissions
    claim_mappings: HashMap<String, Vec<Permission>>,

    // Default permissions for authenticated users
    default_permissions: Vec<Permission>,

    // Required claims/groups for access
    required_groups: Option<Vec<String>>,
}

impl OAuth2Authorization {
    pub fn authorize_user(&self, claims: &UserClaims)
        -> Result<Vec<Permission>> {
        // Check required groups if configured
        if let Some(required) = &self.required_groups {
            let user_groups: Vec<String> = claims.groups
                .as_ref()
                .unwrap_or(&vec![])
                .clone();

            if !required.iter().any(|g| user_groups.contains(g)) {
                return Err(AuthError::InsufficientPermissions);
            }
        }

        // Map claims to permissions
        let mut permissions = self.default_permissions.clone();

        for (claim, perms) in &self.claim_mappings {
            if claims.has_claim(claim) {
                permissions.extend(perms.clone());
            }
        }

        Ok(permissions)
    }
}
```

## Implementation Details

### Configuration Interface

**Web Console Setup**:

```html
<!-- OAuth2 Configuration Form -->
<form hx-post="/admin/auth/oauth2/configure">
    <div class="form-group">
        <label>Identity Provider</label>
        <select name="provider_type">
            <option value="okta">Okta</option>
            <option value="auth0">Auth0</option>
            <option value="azure">Azure AD</option>
            <option value="google">Google</option>
            <option value="generic">Custom OIDC</option>
        </select>
    </div>

    <div class="form-group">
        <label>Discovery URL</label>
        <input type="url" name="discovery_url" required>
        <small>OIDC discovery endpoint
        (.well-known/openid_configuration)</small>
    </div>

    <div class="form-group">
        <label>Client ID</label>
        <input type="text" name="client_id" required>
    </div>

    <div class="form-group">
        <label>Client Secret</label>
        <input type="password" name="client_secret" required>
    </div>

    <button type="submit">Test & Configure</button>
</form>
```

**Environment Variables**:

```bash
# Bootstrap authentication (initial setup only)
CAXTON_ROOT_PASSWORD="secure-bootstrap-password"

# OAuth2 configuration (production)
CAXTON_OAUTH2_ISSUER="https://company.okta.com"
CAXTON_OAUTH2_CLIENT_ID="caxton-admin-console"
CAXTON_OAUTH2_CLIENT_SECRET="oauth2-client-secret"
CAXTON_OAUTH2_REDIRECT_URI="https://caxton.example.com/auth/callback"
```

### Session Management

**Secure Session Handling**:

```rust
pub struct UserSession {
    // User identity from OAuth2
    user_id: String,
    email: String,
    name: String,

    // Token information
    access_token: String,
    refresh_token: Option<String>,
    expires_at: DateTime<Utc>,

    // Authorization
    permissions: Vec<Permission>,

    // Security
    csrf_token: String,
    session_id: String,
}

impl SessionStore {
    pub async fn create_session(&self, user_claims: UserClaims)
        -> Result<UserSession> {
        let session = UserSession {
            user_id: user_claims.sub,
            email: user_claims.email,
            name: user_claims.name,
            access_token: user_claims.access_token,
            refresh_token: user_claims.refresh_token,
            expires_at: DateTime::from_timestamp(user_claims.exp, 0).unwrap(),
            permissions: self.authorization.authorize_user(&user_claims)?,
            csrf_token: generate_csrf_token(),
            session_id: generate_session_id(),
        };

        // Store session (encrypted)
        self.store.set(&session.session_id, &session,
            session.expires_at).await?;

        Ok(session)
    }
}
```

## Benefits

### Security Advantages

- **Delegated Security**: Authentication complexity handled by
  specialized providers
- **No Password Storage**: Eliminates password-related vulnerabilities
- **MFA Integration**: Multi-factor authentication through identity provider
- **Centralized Audit**: Authentication events logged by identity provider
- **Token-Based**: Stateless authentication with JWT tokens
- **Automatic Logout**: Token expiration enforces session timeouts

### Enterprise Integration

- **SSO Compliance**: Meets enterprise SSO requirements
- **Existing Workflows**: Users authenticate with familiar corporate credentials
- **Centralized Management**: User provisioning/deprovisioning via
  identity provider
- **Group-Based Authorization**: Leverage existing organizational groups
- **Compliance Reporting**: Audit trails through corporate identity systems

### Operational Benefits

- **Simplified Deployment**: No password management complexity
- **Standard Protocol**: OAuth2/OIDC is universally supported
- **Provider Flexibility**: Support for multiple identity providers
- **Development Simplicity**: Well-established libraries and patterns
- **Maintenance Reduction**: Identity provider handles security updates

## Consequences

### Positive

- **Enterprise Ready**: Meets corporate authentication requirements
- **Security Excellence**: Delegates authentication to security specialists
- **User Experience**: Familiar SSO login experience
- **Compliance**: Supports audit and regulatory requirements
- **Scalability**: Identity provider handles user management at scale
- **Standards Based**: OAuth2/OIDC are industry standards

### Negative

- **External Dependency**: Requires identity provider availability
- **Initial Setup**: Bootstrap process needed for initial configuration
- **Network Connectivity**: Requires network access to identity provider
- **Token Management**: Must handle token refresh and validation
- **Configuration Complexity**: OAuth2 setup requires technical knowledge

### Neutral

- **Industry Standard**: OAuth2/OIDC is the expected authentication method
- **Provider Agnostic**: Works with any compliant OIDC provider
- **Bootstrap Only**: ROOT_PASSWORD only needed during initial setup

## Mitigation Strategies

### Availability Concerns

**Token Caching**:

```rust
pub struct TokenCache {
    // Cache validated tokens to reduce provider dependency
    validated_tokens: LruCache<String, (UserClaims, Instant)>,
    cache_duration: Duration,

    // Graceful degradation during provider outages
    offline_mode: bool,
    offline_cache_duration: Duration,
}
```

**Health Monitoring**:

```rust
pub async fn check_provider_health(&self) -> Result<ProviderHealth> {
    // Test OIDC discovery endpoint
    let discovery = reqwest::get(&self.config.discovery_url).await?;

    // Test token endpoint
    let endpoints = discovery.json::<OidcDiscovery>().await?;
    let token_endpoint = reqwest::get(&endpoints.token_endpoint).await?;

    Ok(ProviderHealth {
        discovery_available: discovery.status().is_success(),
        token_endpoint_available: token_endpoint.status().is_success(),
        last_check: Utc::now(),
    })
}
```

### Configuration Documentation

**Provider-Specific Guides**:

- Okta application setup with Caxton redirect URIs
- Auth0 application configuration with required scopes
- Azure AD app registration with API permissions
- Google Workspace OAuth2 client creation
- Keycloak client configuration for OIDC

## Related Decisions

- ADR-0022: Web-based Admin Console - Established need for web authentication
- ADR-0023: HTTPS Reverse Proxy Pattern - Provides secure transport
  for OAuth2 flows
- ADR-0016: Security Architecture - Defines overall security approach for Caxton
- ADR-0004: Minimal Core Philosophy - OAuth2 delegation aligns with
  avoiding security complexity

## References

- [OAuth 2.0 Authorization Framework (RFC 6749)](
  https://tools.ietf.org/html/rfc6749)
- [OpenID Connect Core 1.0](
  https://openid.net/specs/openid-connect-core-1_0.html)
- [OAuth 2.0 Security Best Current Practice](
  https://tools.ietf.org/html/draft-ietf-oauth-security-topics)
- [OWASP Authentication Cheat Sheet](
  https://cheatsheetseries.owasp.org/cheatsheets/Authentication_Cheat_Sheet.html)
- [Enterprise Identity Providers](https://openid.net/certification/)
