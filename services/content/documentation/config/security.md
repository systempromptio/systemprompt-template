---
title: "Security Configuration"
description: "JWT token settings including issuer, expiration times, and audience configuration."
author: "SystemPrompt Team"
slug: "config/security"
keywords: "security, jwt, token, expiration, issuer, audience"
image: "/files/images/docs/config-security.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Security Configuration

Security settings control JWT token issuance, including issuer identity, token lifetimes, and allowed audiences.

## Configuration

```yaml
# .systemprompt/profiles/local/profile.yaml
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

## Fields

| Field | Type | Description |
|-------|------|-------------|
| `jwt_issuer` | String | Token issuer identity |
| `jwt_access_token_expiration` | i64 | Access token lifetime (seconds) |
| `jwt_refresh_token_expiration` | i64 | Refresh token lifetime (seconds) |
| `jwt_audiences` | List | Allowed token audiences |

## Token Expiration

| Duration | Seconds | Use Case |
|----------|---------|----------|
| 1 hour | 3,600 | High security |
| 24 hours | 86,400 | Session-based |
| 30 days | 2,592,000 | Default access |
| 180 days | 15,552,000 | Default refresh |
| 1 year | 31,536,000 | Maximum |

**Maximum**: Access token expiration cannot exceed 31,536,000 seconds (1 year).

## JWT Audiences

| Audience | Use Case |
|----------|----------|
| `web` | Browser applications |
| `api` | Direct API access |
| `a2a` | Agent-to-agent communication |
| `mcp` | Model Context Protocol |

## Development vs Production

| Setting | Development | Production |
|---------|-------------|------------|
| `jwt_issuer` | `systemprompt-local` | `systemprompt-production` |
| Access expiration | 30 days | 24 hours |
| Refresh expiration | 180 days | 90 days |

## Production Example

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

See the [Security Configuration Playbook](/playbooks/config-security) for detailed technical information.