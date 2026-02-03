---
title: "Secrets Management Playbook"
description: "Manage API keys, credentials, and sensitive configuration."
keywords:
  - secrets
  - credentials
  - api-keys
  - security
category: cli
---

# Secrets Management Playbook

Manage API keys, credentials, and sensitive configuration.

> **Help**: `{ "command": "cloud secrets --help" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Understanding Secrets

Secrets are stored in profile-specific `secrets.json` files:

```
.systemprompt/profiles/
├── local/
│   └── secrets.json       # Local development secrets
└── systemprompt-prod/
    └── secrets.json       # Production secrets
```

**All secrets files are gitignored by default.**

---

## Secrets Flow

```
┌─────────────────────────────────────────────────────────────────┐
│  1. ADD TO secrets.json                                         │
│     Edit .systemprompt/profiles/<profile>/secrets.json          │
│                           │                                      │
│                           ▼                                      │
│  2. SYNC TO CLOUD                                               │
│     systemprompt cloud secrets sync                             │
│                           │                                      │
│                           ▼                                      │
│  3. VERIFY                                                       │
│     Check cloud dashboard or use secrets in services            │
└─────────────────────────────────────────────────────────────────┘
```

---

## secrets.json Format

```json
{
  "jwt_secret": "minimum-32-character-secret-key-here",
  "database_url": "postgres://user:pass@host:5432/db",
  "anthropic": "sk-ant-...",
  "openai": "sk-...",
  "gemini": "AIza...",
  "moltbook_builder": "moltbook_sk_...",
  "custom_api_key": "your-key-here"
}
```

**Key naming conventions:**
- Use snake_case for all keys
- Agent-specific keys: `{agent_name}` (e.g., `moltbook_builder`)
- Service keys: `{service_name}` (e.g., `anthropic`, `openai`)

---

## Add New Secrets

### Step 1: Edit secrets.json

```bash
# Open the secrets file for your profile
nano .systemprompt/profiles/local/secrets.json
```

Add your new secret:
```json
{
  "existing_key": "existing_value",
  "new_api_key": "your-new-key-here"
}
```

### Step 2: Sync to Cloud

```bash
systemprompt cloud secrets sync
```

---

## Sync Secrets

Sync local secrets.json to cloud:

```json
{ "command": "cloud secrets sync" }
```

This reads from `.systemprompt/profiles/<active-profile>/secrets.json` and uploads to cloud.

---

## Set Individual Secrets

Set secrets directly without editing files:

```json
{ "command": "cloud secrets set ANTHROPIC_API_KEY=sk-ant-..." }
{ "command": "cloud secrets set MOLTBOOK_BUILDER=moltbook_sk_..." }
{ "command": "cloud secrets set KEY1=value1 KEY2=value2" }
```

**Note:** This sets secrets in cloud but does NOT update local secrets.json. For consistency, prefer editing secrets.json and using `sync`.

---

## Remove Secrets

```json
{ "command": "cloud secrets unset GITHUB_TOKEN" }
{ "command": "cloud secrets unset OLD_KEY UNUSED_KEY" }
```

---

## Cleanup System Variables

Remove incorrectly synced system-managed variables:

```json
{ "command": "cloud secrets cleanup" }
```

---

## Required Secrets

| Secret | Required For | Format |
|--------|--------------|--------|
| `jwt_secret` | Authentication | Min 32 characters |
| `database_url` | Database | `postgres://user:pass@host:port/db` |
| `anthropic` | Claude AI | `sk-ant-...` |
| `openai` | OpenAI | `sk-...` |
| `gemini` | Google AI | `AIza...` |

### Generate JWT Secret

```bash
openssl rand -base64 48
```

---

## Agent-Specific Secrets

For agents that need their own API keys (like Moltbook agents):

```json
{
  "moltbook_builder": "moltbook_sk_...",
  "moltbook_community": "moltbook_sk_...",
  "moltbook_devrel": "moltbook_sk_..."
}
```

Agents access their secrets via the secrets service, keyed by agent name.

---

## Environment Variables

Secrets can also be set via environment variables for local development:

```bash
# In .env file
ANTHROPIC_API_KEY=sk-ant-...
MOLTBOOK_API_KEY=moltbook_sk_...

# Or export directly
export ANTHROPIC_API_KEY=sk-ant-...
```

**Priority:** Environment variables override secrets.json values.

---

## Profile-Specific Secrets

Each profile has its own secrets:

```bash
# Sync secrets for specific profile
systemprompt cloud secrets sync --profile production

# Set secret for specific profile
systemprompt cloud secrets set API_KEY=value --profile staging
```

---

## Troubleshooting

**Secret not found in service:**
1. Check secrets.json has the key
2. Run `cloud secrets sync`
3. Restart the service

**Sync failed:**
1. Check network connection
2. Verify active session: `admin session show`
3. Check cloud authentication

**Permission denied:**
1. Verify you have admin role
2. Check tenant permissions

---

## Security Best Practices

1. **Never commit secrets** - All secret files are gitignored
2. **Use separate secrets per profile** - Different keys for local vs production
3. **Rotate regularly** - Update keys periodically
4. **Least privilege** - Only add secrets that are actually needed
5. **Use secrets.json + sync** - Don't scatter secrets across .env files

---

## Quick Reference

| Task | Command |
|------|---------|
| Sync secrets to cloud | `cloud secrets sync` |
| Set secret directly | `cloud secrets set KEY=VALUE` |
| Set multiple secrets | `cloud secrets set K1=V1 K2=V2` |
| Remove secret | `cloud secrets unset KEY` |
| Cleanup system vars | `cloud secrets cleanup` |
| Sync for profile | `cloud secrets sync --profile NAME` |

-> See [Session Playbook](session.md) for authentication.
-> See [Cloud Playbook](cloud.md) for cloud setup.
