---
title: "Cloud Credentials"
description: "Cloud API authentication credentials, token management, and login workflow."
author: "SystemPrompt Team"
slug: "config/credentials"
keywords: "credentials, cloud, authentication, token, oauth, login"
image: "/files/images/docs/config-credentials.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Cloud Credentials

Cloud credentials authenticate your CLI and API requests to SystemPrompt Cloud. They're stored in `.systemprompt/credentials.json` and expire 24 hours after login.

## How It Works

When you run `just login`, SystemPrompt opens your browser for OAuth authentication. After successful login, credentials are saved locally.

```
just login
    │
    └── Opens browser for GitHub/Google OAuth
            │
            └── Saves credentials.json
                    │
                    └── Valid for 24 hours
```

## credentials.json Structure

```json
{
  "api_token": "sp_live_eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "api_url": "https://api.systemprompt.io",
  "authenticated_at": "2026-02-01T10:00:00Z",
  "user_email": "user@example.com"
}
```

| Field | Description |
|-------|-------------|
| `api_token` | JWT token for API authentication |
| `api_url` | Cloud API endpoint |
| `authenticated_at` | Login timestamp |
| `user_email` | User's email address |

## Token Expiration

Tokens expire 24 hours after `authenticated_at`. When expired:

1. CLI commands return authentication errors
2. Run `just login` to re-authenticate
3. New credentials.json is created

## Profile Configuration

Reference credentials in your profile:

```yaml
# .systemprompt/profiles/local/profile.yaml
cloud:
  credentials_path: "../../credentials.json"
  tenants_path: "../../tenants.json"
```

## Container Deployment

In Fly.io containers, credentials load from environment variables:

| Environment Variable | Description |
|---------------------|-------------|
| `SYSTEMPROMPT_API_TOKEN` | API token (required) |
| `SYSTEMPROMPT_USER_EMAIL` | User email (required) |
| `SYSTEMPROMPT_API_URL` | API endpoint (optional) |

## Commands

| Task | Command |
|------|---------|
| Login | `just login` |
| Check auth | `systemprompt cloud auth whoami` |
| Logout | `just logout` |

## Security

- Never commit credentials.json to git
- File should have `0600` permissions (owner read/write only)
- Tokens can be revoked by contacting support

See the [Cloud Credentials Playbook](/playbooks/config-credentials) for detailed technical information.