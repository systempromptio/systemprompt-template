---
title: "Security Configuration"
description: "Configure JWT token settings including issuer, expiration, and audiences."
author: "SystemPrompt"
slug: "config-security"
keywords: "security, jwt, token, expiration, issuer, audience, authentication"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Security Configuration

Configure JWT token settings including issuer, expiration, and audiences.

> **Help**: `{ "command": "admin config security show" }` via `systemprompt_help`
> **Requires**: Profile configured -> See [Profiles Playbook](../profiles/index.md)

SecurityConfig defines JWT token settings: issuer, expiration times, and allowed audiences.

---

## SecurityConfig Struct

**Source**: `crates/shared/models/src/profile/security.rs:4-22`

```rust
pub struct SecurityConfig {
    #[serde(rename = "jwt_issuer")]
    pub issuer: String,                        // Required: JWT "iss" claim
    #[serde(rename = "jwt_access_token_expiration")]
    pub access_token_expiration: i64,          // Required: Seconds (max 1 year)
    #[serde(rename = "jwt_refresh_token_expiration")]
    pub refresh_token_expiration: i64,         // Required: Seconds
    #[serde(rename = "jwt_audiences")]
    pub audiences: Vec<JwtAudience>,           // Required: Allowed audiences
}
```

### Field Details

| Field | YAML Name | Type | Range | Default |
|-------|-----------|------|-------|---------|
| `issuer` | `jwt_issuer` | String | Non-empty | - |
| `access_token_expiration` | `jwt_access_token_expiration` | i64 | 1 - 31,536,000 | 2,592,000 (30 days) |
| `refresh_token_expiration` | `jwt_refresh_token_expiration` | i64 | > 0 | 15,552,000 (180 days) |
| `audiences` | `jwt_audiences` | Vec | Non-empty | - |

---

## JWT Issuer

The issuer identifies who created the token. It appears in the `iss` claim.

### Configuration

```yaml
security:
  jwt_issuer: "systemprompt-local"
```

### Recommendations

| Environment | Issuer |
|-------------|--------|
| Local | `systemprompt-local` |
| Staging | `systemprompt-staging` |
| Production | `systemprompt-production` or your domain |

---

## Token Expiration

### Access Token

Short-lived token for API requests.

```yaml
security:
  jwt_access_token_expiration: 2592000  # 30 days in seconds
```

**Maximum**: 31,536,000 seconds (1 year)

### Refresh Token

Long-lived token to obtain new access tokens.

```yaml
security:
  jwt_refresh_token_expiration: 15552000  # 180 days in seconds
```

### Common Values

| Duration | Seconds | Use Case |
|----------|---------|----------|
| 1 hour | 3600 | High security |
| 24 hours | 86400 | Session-based |
| 7 days | 604800 | Web apps |
| 30 days | 2592000 | Default |
| 90 days | 7776000 | Mobile apps |
| 180 days | 15552000 | Refresh default |
| 1 year | 31536000 | Maximum |

---

## JWT Audiences

Audiences define what clients can use the token.

### JwtAudience Enum

```rust
pub enum JwtAudience {
    Web,      // Browser-based applications
    Api,      // Direct API access
    A2a,      // Agent-to-agent communication
    Mcp,      // Model Context Protocol
}
```

### Configuration

```yaml
security:
  jwt_audiences:
    - web
    - api
    - a2a
    - mcp
```

### Audience Use Cases

| Audience | Use Case | Examples |
|----------|----------|----------|
| `web` | Browser clients | React app, Vue app |
| `api` | Direct API calls | REST clients, Postman |
| `a2a` | Agent communication | A2A protocol messages |
| `mcp` | MCP servers | Tool/resource access |

---

## Validation Rules

**Source**: `crates/shared/models/src/profile/validation.rs:141-149`

### Expiration Validation

```rust
fn validate_security_settings(&self) -> Result<()> {
    if self.security.access_token_expiration <= 0 {
        return Err(ProfileError::InvalidTokenExpiration("access_token"));
    }
    if self.security.refresh_token_expiration <= 0 {
        return Err(ProfileError::InvalidTokenExpiration("refresh_token"));
    }
    Ok(())
}
```

### Rules

| Setting | Rule |
|---------|------|
| `access_token_expiration` | > 0, <= 31,536,000 |
| `refresh_token_expiration` | > 0 |
| `issuer` | Non-empty string |
| `audiences` | At least one value |

---

## Complete Configuration Examples

### Local Development

```yaml
security:
  jwt_issuer: "systemprompt-local"
  jwt_access_token_expiration: 2592000      # 30 days
  jwt_refresh_token_expiration: 15552000    # 180 days
  jwt_audiences:
    - web
    - api
    - a2a
    - mcp
```

### Production (Standard)

```yaml
security:
  jwt_issuer: "systemprompt-production"
  jwt_access_token_expiration: 86400        # 24 hours
  jwt_refresh_token_expiration: 7776000     # 90 days
  jwt_audiences:
    - web
    - api
    - a2a
    - mcp
```

### Production (High Security)

```yaml
security:
  jwt_issuer: "api.example.com"
  jwt_access_token_expiration: 3600         # 1 hour
  jwt_refresh_token_expiration: 604800      # 7 days
  jwt_audiences:
    - web
    - api
```

---

## JWT Token Structure

Tokens issued by SystemPrompt include these claims:

```json
{
  "iss": "systemprompt-local",
  "aud": ["web", "api"],
  "sub": "user_abc123",
  "exp": 1706832000,
  "iat": 1704240000,
  "jti": "token_xyz789"
}
```

| Claim | Description |
|-------|-------------|
| `iss` | Issuer (from config) |
| `aud` | Audience (from config) |
| `sub` | Subject (user ID) |
| `exp` | Expiration timestamp |
| `iat` | Issued at timestamp |
| `jti` | Unique token ID |

---

## Environment Variables

When using `Profile::from_env()`:

| Env Variable | Maps To |
|--------------|---------|
| `JWT_ISSUER` | `security.issuer` |
| `JWT_ACCESS_TOKEN_EXPIRATION` | `security.access_token_expiration` |
| `JWT_REFRESH_TOKEN_EXPIRATION` | `security.refresh_token_expiration` |
| `JWT_AUDIENCES` | `security.audiences` (comma-separated) |

### Example

```bash
export JWT_ISSUER="systemprompt-local"
export JWT_ACCESS_TOKEN_EXPIRATION=2592000
export JWT_REFRESH_TOKEN_EXPIRATION=15552000
export JWT_AUDIENCES="web,api,a2a,mcp"
```

---

## Config Access

After bootstrap, security config is available via Config struct:

```rust
let config = Config::get()?;
println!("Issuer: {}", config.jwt_issuer);
println!("Access expiration: {} seconds", config.jwt_access_token_expiration);
println!("Refresh expiration: {} seconds", config.jwt_refresh_token_expiration);
println!("Audiences: {:?}", config.jwt_audiences);
```

---

## Security Best Practices

### Token Expiration

1. **Shorter is safer**: Reduce exposure window
2. **Balance UX**: Very short tokens require frequent refresh
3. **Refresh tokens**: Longer-lived but more carefully protected
4. **Production**: Consider 1-24 hour access tokens

### Issuer

1. **Unique per environment**: Prevents token misuse across environments
2. **Include domain**: Makes token origin clear
3. **Consistent**: Same issuer for all tokens in an environment

### Audiences

1. **Least privilege**: Only grant needed audiences
2. **Separate tokens**: Different tokens for different uses
3. **Validate always**: Check audience on every request

---

## Troubleshooting

**"Token expired"**
- Access token lived past `access_token_expiration`
- Use refresh token to get new access token
- Check server clock synchronization

**"Invalid audience"**
- Token's audience doesn't match required audience
- Ensure token request includes correct audience
- Check `jwt_audiences` in profile

**"Invalid issuer"**
- Token issuer doesn't match expected
- Verify `jwt_issuer` in profile
- May be using token from wrong environment

**"Token expiration invalid"**
- `access_token_expiration` must be > 0
- Maximum is 31,536,000 (1 year)
- Value is in seconds, not milliseconds

---

## Quick Reference

| Setting | Development | Production |
|---------|-------------|------------|
| `jwt_issuer` | `systemprompt-local` | `systemprompt-production` |
| `jwt_access_token_expiration` | 2,592,000 (30d) | 86,400 (24h) |
| `jwt_refresh_token_expiration` | 15,552,000 (180d) | 7,776,000 (90d) |
| `jwt_audiences` | `[web, api, a2a, mcp]` | `[web, api, a2a, mcp]` |

### Time Conversions

| Human | Seconds |
|-------|---------|
| 1 hour | 3,600 |
| 1 day | 86,400 |
| 1 week | 604,800 |
| 30 days | 2,592,000 |
| 90 days | 7,776,000 |
| 180 days | 15,552,000 |
| 1 year | 31,536,000 |