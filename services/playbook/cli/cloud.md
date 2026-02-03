---
title: "Cloud Management Playbook"
description: "Authentication, tenants, profiles, secrets, and complete setup flow for cloud operations."
author: "SystemPrompt"
slug: "cli-cloud"
keywords: "cloud, authentication, tenants, profiles, secrets, setup, bootstrap"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Cloud Management Playbook

Authentication, tenants, profiles, secrets, and complete setup flow for cloud operations.

---

## Authentication

```json
{ "command": "cloud auth whoami" }
```

Login and logout require the terminal:

```bash
just login
just logout
```

---

## Tenant Management

```json
{ "command": "cloud tenant list" }
{ "command": "cloud tenant show" }
{ "command": "cloud tenant show <tenant-id>" }
{ "command": "cloud tenant create --region iad" }
{ "command": "cloud tenant create --name \"My Project\" --region lhr" }
{ "command": "cloud tenant select <tenant-id>" }
{ "command": "cloud tenant rotate-credentials <tenant-id> -y" }
```

---

## Profile Management

```json
{ "command": "cloud profile list" }
{ "command": "cloud profile show" }
{ "command": "cloud profile show <profile-name>" }
{ "command": "cloud profile create production" }
{ "command": "cloud profile create staging --environment staging" }
{ "command": "cloud profile edit <profile-name>" }
{ "command": "cloud profile delete staging -y" }
```

---

## Secrets Management

```json
{ "command": "cloud secrets list" }
{ "command": "cloud secrets list --profile <profile-name>" }
{ "command": "cloud secrets set ANTHROPIC_API_KEY sk-ant-xxxxx" }
{ "command": "cloud secrets set DATABASE_URL postgres://..." }
{ "command": "cloud secrets delete OLD_KEY -y" }
```

---

## Cloud Database

```json
{ "command": "cloud db status" }
{ "command": "cloud db status --profile <profile-name>" }
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM users\"" }
{ "command": "cloud db query --profile <profile-name> \"SELECT * FROM content LIMIT 5\"" }
{ "command": "cloud db tables --profile <profile-name>" }
```

---

## Cloud Status & Operations

```json
{ "command": "cloud status" }
{ "command": "cloud restart --yes" }
{ "command": "cloud init" }
{ "command": "cloud dockerfile" }
{ "command": "cloud dockerfile --output Dockerfile" }
```

---

## Complete Setup Flow

### Step 1: Cloud Login

Authenticate with SystemPrompt Cloud using OAuth.

```json
{ "command": "cloud auth login" }
```

**What it does:**
- Opens browser for GitHub/Google OAuth
- Saves credentials to `.systemprompt/credentials.json`
- Displays user info and available subscriptions

### Step 2: Tenant Setup

Link this project to a cloud tenant.

```json
{ "command": "cloud tenant create --name my-project --region iad" }
{ "command": "cloud tenant list" }
{ "command": "cloud tenant" }
```

**What it does:**
- Lists existing tenants or creates new one
- Configures tenant in credentials file
- Polls for tenant provisioning (1-2 minutes)

### Step 3: Profile Configuration

Interactive wizard for complete environment setup.

```json
{ "command": "cloud profile create local" }
{ "command": "cloud profile create production" }
```

**Generates:**
```
.systemprompt/profiles/<env>/
├── profile.yml         # Runtime configuration
├── secrets.json        # API keys + DATABASE_URL
└── docker-compose.yml  # Local services (local only)
```

### Step 4: Database Setup

Start the database and run migrations.

```bash
just db-up
```

```json
{ "command": "infra db migrate" }
```

### Step 5: Verify

```json
{ "command": "cloud status" }
{ "command": "cloud auth whoami" }
```

---

## The `.systemprompt` Directory

### Structure

```
project-root/
└── .systemprompt/                      # Project-level (gitignored)
    ├── credentials.json                # Auth token + user info
    ├── tenants.json                    # Tenant cache
    ├── Dockerfile                      # Application image
    └── profiles/                       # Environment-specific configs
        ├── local/
        │   ├── profile.yaml            # Configuration (note: .yaml not .yml)
        │   ├── secrets.json
        │   └── docker/
        │       └── docker-compose.yaml
        └── production/
            ├── profile.yaml
            └── secrets.json
```

### File Specifications

#### credentials.json

See [Cloud Credentials Playbook](/playbooks/config-credentials) for details.

```json
{
  "api_token": "sp_live_abc123...",
  "api_url": "https://api.systemprompt.io",
  "authenticated_at": "2026-02-01T10:00:00Z",
  "user_email": "developer@example.com"
}
```

#### tenants.json

See [Tenant Management Playbook](/playbooks/config-tenants) for details.

```json
{
  "tenants": [
    {
      "id": "local_abc123",
      "name": "my-project",
      "tenant_type": "local",
      "database_url": "postgres://localhost:5432/systemprompt"
    },
    {
      "id": "ten_abc123",
      "name": "my-project-prod",
      "tenant_type": "cloud",
      "app_id": "sp-my-project-abc",
      "hostname": "my-project.systemprompt.io",
      "region": "iad",
      "sync_token": "sp_sync_..."
    }
  ],
  "synced_at": "2026-02-01T10:00:00Z"
}
```

#### secrets.json

See [Secrets Management Playbook](/playbooks/config-secrets) for details.

```json
{
  "jwt_secret": "your-secret-key-minimum-32-characters-long",
  "database_url": "postgres://user:pass@localhost:5432/db",
  "anthropic": "sk-ant-...",
  "openai": "sk-...",
  "gemini": "AIza..."
}
```

**Note**: `jwt_secret` must be at least 32 characters.

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SYSTEMPROMPT_PROFILE` | Path to active profile |
| `DATABASE_URL` | PostgreSQL connection |
| `ANTHROPIC_API_KEY` | Claude API key |
| `OPENAI_API_KEY` | OpenAI API key |

---

## Bootstrap from Scratch

For a completely fresh project:

```bash
SQLX_OFFLINE=true cargo build --release --manifest-path core/crates/entry/cli/Cargo.toml
./core/target/release/systemprompt cloud auth login
./core/target/release/systemprompt cloud tenant
./core/target/release/systemprompt cloud profile
./core/target/release/systemprompt infra db migrate
./core/target/release/systemprompt infra services start --all
```

Server available at `http://127.0.0.1:8080`.

---

## Setup Flow Checklist

| Phase | Command | Verify |
|-------|---------|--------|
| 1 | `just login` | credentials.json created |
| 2 | `just tenant` | tenants.json created |
| 3 | `just init` | services/ created |
| 4 | `just configure` | profiles/ created |
| 5 | `just db-up` | Container running |
| 6 | `just migrate` | Tables created |
| 7 | `just sync` | Data synced |
| 8 | `just start` | Server running |
| 9 | `just deploy` | Deployed (optional) |

---

## Security Notes

1. **Never commit secrets** -- `.systemprompt/` is gitignored
2. **Project-specific credentials** -- Each project has its own tenant
3. **Environment isolation** -- Each profile has its own secrets
4. **Token refresh** -- Re-run `login` if API calls fail with 401
5. **File permissions** -- Secrets files created with `0o600`
6. **JWT secret length** -- Minimum 32 characters enforced

---

## Troubleshooting

**"Not logged in"** -- `just login`

**"No tenant configured"** -- `cloud tenant` or `just tenant`

**"Profile required"** -- `export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml`

**"Database connection failed"** -- `just db-up` then verify with `docker ps | grep postgres`

**"Cloud token expired"** -- `just login`

**Cloud DB connection failed** -- verify profile config with `cloud profile show <profile-name>` and check `cloud status`.

**JWT Secret Too Short** -- Generate a longer secret: `openssl rand -base64 48`

---

## Quick Reference

| Task | Command |
|------|---------|
| Login | `just login` (terminal) |
| Check auth | `cloud auth whoami` |
| Logout | `just logout` (terminal) |
| List tenants | `cloud tenant list` |
| Show tenant | `cloud tenant show` |
| Create tenant | `cloud tenant create --region iad` |
| Select tenant | `cloud tenant select <id>` |
| List profiles | `cloud profile list` |
| Show profile | `cloud profile show <name>` |
| Create profile | `cloud profile create <name>` |
| Edit profile | `cloud profile edit <name>` |
| Delete profile | `cloud profile delete <name> -y` |
| List secrets | `cloud secrets list` |
| Set secret | `cloud secrets set KEY value` |
| Delete secret | `cloud secrets delete KEY -y` |
| DB status | `cloud db status --profile <name>` |
| DB query | `cloud db query --profile <name> "SQL"` |
| DB tables | `cloud db tables --profile <name>` |
| Cloud status | `cloud status` |
| Restart cloud | `cloud restart --yes` |
| Init project | `cloud init` |
| Full setup | `just login && just tenant && just configure && just db-up && just migrate` |