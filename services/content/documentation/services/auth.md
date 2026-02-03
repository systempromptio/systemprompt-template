---
title: "Authentication"
description: "OAuth2/OIDC and WebAuthn authentication built in. Ship AI products with enterprise-grade security from day one."
author: "SystemPrompt Team"
slug: "services/auth"
keywords: "authentication, oauth2, oidc, webauthn, security, tokens"
image: "/files/images/docs/services-auth.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Authentication

SystemPrompt includes a complete authentication system that handles both API access and user login. OAuth2 with OpenID Connect secures API calls and MCP tool execution. WebAuthn provides passwordless authentication for web interfaces. Every authentication event is logged for audit compliance.

The authentication system is not optional or bolted on. It's woven into every layer of the platform. When you connect Claude Desktop to a SystemPrompt MCP server, OAuth2 handles the authorization. When a user logs into your AI product, WebAuthn eliminates password vulnerabilities. When agents communicate via A2A, scoped tokens ensure proper authorization.

## Authentication Architecture

The authentication architecture has three layers, each handling a different concern.

**Identity layer**: Determines who is making a request. This layer handles user registration, credential verification, and session management. WebAuthn provides the primary authentication mechanism for interactive users.

**Authorization layer**: Determines what a requester can do. OAuth2 scopes define permissions. Every API endpoint, MCP tool, and agent capability is protected by scope requirements. The `systemprompt-security` crate implements token generation, validation, and introspection.

**Audit layer**: Records what actually happened. Every authentication event, token issuance, and authorization decision is logged with trace IDs. This creates a complete audit trail for compliance and debugging.

The `systemprompt-oauth` crate implements the OAuth2 authorization server. It supports authorization code flow with PKCE for web applications, client credentials flow for service-to-service communication, and token introspection for resource servers.

## OAuth2 Authorization Flows

SystemPrompt implements OAuth2 as the authorization framework for all API access. The authorization server exposes standard endpoints that work with any OAuth2 client.

**Authorization endpoints:**

| Endpoint | Purpose |
|----------|---------|
| `/api/v1/core/oauth/authorize` | Authorization request |
| `/api/v1/core/oauth/token` | Token exchange |
| `/api/v1/core/oauth/introspect` | Token introspection |
| `/.well-known/openid-configuration` | OIDC discovery |

**Authorization code flow with PKCE:**

The recommended flow for web applications and native apps. PKCE (Proof Key for Code Exchange) prevents authorization code interception attacks.

1. Client generates a code verifier and challenge
2. User is redirected to authorization endpoint with challenge
3. User authenticates and grants consent
4. Authorization server redirects back with authorization code
5. Client exchanges code + verifier for access token

```yaml
# Agent OAuth2 configuration
securitySchemes:
  oauth2:
    type: oauth2
    flows:
      authorizationCode:
        authorizationUrl: "/api/v1/core/oauth/authorize"
        tokenUrl: "/api/v1/core/oauth/token"
        scopes:
          anonymous: "Public access without authentication"
          user: "Authenticated user access"
          admin: "Administrative operations"
```

**Client credentials flow:**

For service-to-service authentication where no user is involved. The MCP server uses this flow when executing commands on behalf of the system.

```bash
# Obtain service token
curl -X POST /api/v1/core/oauth/token \
  -d "grant_type=client_credentials" \
  -d "client_id=mcp-server" \
  -d "client_secret=${CLIENT_SECRET}" \
  -d "scope=admin"
```

**Token structure:**

Access tokens are JWTs containing claims about the authenticated principal and granted scopes. The token includes:

- `sub`: Subject identifier (user ID or client ID)
- `scope`: Space-separated list of granted scopes
- `aud`: Intended audience (e.g., "mcp", "api")
- `exp`: Expiration timestamp
- `tenant_id`: Multi-tenant isolation identifier

## WebAuthn Passwordless Authentication

WebAuthn eliminates passwords entirely. Users authenticate with biometrics (Face ID, Touch ID, Windows Hello) or hardware security keys (YubiKey). This provides phishing-resistant authentication that cannot be stolen or guessed.

**Registration flow:**

1. User initiates registration
2. Server generates a challenge
3. User's authenticator creates a credential
4. Public key is stored server-side
5. Private key never leaves the authenticator

**Authentication flow:**

1. User initiates login
2. Server sends challenge with allowed credentials
3. Authenticator signs challenge with private key
4. Server verifies signature with stored public key
5. Session is established

WebAuthn credentials are bound to the origin (domain), preventing phishing attacks. Even if a user is tricked into visiting a fake site, the authenticator will refuse to respond because the origin doesn't match.

The CLI uses device-bound credentials for authentication:

```bash
# Initiate WebAuthn login
systemprompt cloud auth login

# This opens a browser for biometric verification
# No password is ever transmitted
```

## Scopes for Fine-Grained Access Control

Scopes are the unit of permission in SystemPrompt. Every protected operation requires specific scopes. Users and applications receive tokens with only the scopes they need.

**Common scopes:**

| Scope | Permission |
|-------|------------|
| `anonymous` | Public access, no authentication required |
| `user` | Basic authenticated user operations |
| `admin` | Administrative operations |
| `tools:read` | Read MCP tool definitions |
| `tools:execute` | Execute MCP tools |
| `agents:read` | View agent configurations |
| `agents:write` | Modify agent configurations |

**Scope enforcement:**

Scopes are checked at multiple levels. API routes declare required scopes in their handlers. MCP tools specify scope requirements in their manifests. Agents define which scopes are needed to interact with them.

```yaml
# MCP server OAuth configuration
oauth:
  required: true
  scopes: ["admin"]
  audience: "mcp"
```

When a request arrives, the authorization layer extracts scopes from the access token and compares them against the operation's requirements. If the token lacks required scopes, the request is rejected with a 403 Forbidden response.

**Scope hierarchies:**

Some scopes imply others. The `admin` scope typically includes all lesser scopes. This is configured in the authorization server and simplifies token management for privileged users.

## Configuration

Authentication settings are distributed across profile configuration and service-specific YAML files.

**Profile security settings:**

```yaml
# .systemprompt/profiles/local/profile.yaml
security:
  auth_secret: ${AUTH_SECRET}
  encryption_key: ${ENCRYPTION_KEY}
  token_expiry_seconds: 3600
```

**Agent security schemes:**

```yaml
# services/agents/welcome.yaml
securitySchemes:
  oauth2:
    type: oauth2
    flows:
      authorizationCode:
        authorizationUrl: "/api/v1/core/oauth/authorize"
        tokenUrl: "/api/v1/core/oauth/token"
        scopes:
          anonymous: "Public access"
          user: "Authenticated access"
          admin: "Administrative access"

security:
  - oauth2: ["anonymous"]  # Minimum required scopes
```

## CLI Authentication Commands

```bash
# Login to SystemPrompt Cloud
systemprompt cloud auth login

# Check current session
systemprompt admin session show

# Refresh access token
systemprompt admin session refresh

# Logout and clear credentials
systemprompt cloud auth logout
```

## CLI Reference

Authentication is managed through cloud auth and session commands.

| Command | Description |
|---------|-------------|
| `systemprompt cloud auth login` | Authenticate with SystemPrompt Cloud via OAuth |
| `systemprompt cloud auth logout` | Clear saved cloud credentials |
| `systemprompt cloud auth whoami` | Show current authenticated user and token status |
| `systemprompt admin session show` | Show current session and routing info |
| `systemprompt admin session switch` | Switch to a different profile |
| `systemprompt admin session list` | List available profiles |
| `systemprompt admin session login` | Create an admin session for CLI access |
| `systemprompt admin session logout` | Remove a session |

See `systemprompt cloud auth --help` and `systemprompt admin session --help` for detailed options.

## Security Considerations

Several security properties are enforced by default:

- **TLS required**: All production traffic must use HTTPS
- **Token rotation**: Access tokens have short lifetimes
- **Credential isolation**: Credentials are never logged or exposed in errors
- **Tenant boundaries**: Tokens are scoped to specific tenants
- **Audit logging**: All authentication events are recorded