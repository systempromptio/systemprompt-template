---
title: "Authentication & Cloud Setup Playbook"
description: "Guide for authenticating, provisioning tenants, and configuring SystemPrompt projects."
keywords:
  - auth
  - cloud
  - setup
  - credentials
---

# Authentication & Cloud Setup

Complete guide to authenticating, provisioning tenants, and configuring your SystemPrompt project.

> **Help**: `{ "command": "playbook cloud" }` via `systemprompt_help`

---

## Command Overview

| Command | Domain | Purpose |
|---------|--------|---------|
| `login` | Auth | Authenticate with SP Cloud |
| `logout` | Auth | Clear credentials |
| `whoami` | Auth | Show current user |
| `tenant` | Tenant | Create or select cloud tenant |
| `init` | Project | Initialize new project |
| `configure` | Profile | Generate profiles (local/production) |
| `secrets` | Secrets | Manage API keys |
| `migrate` | Database | Run database migrations |
| `sync` | Data | Sync content/skills to database |
| `deploy` | Cloud | Push to cloud |
| `status` | Cloud | Check deployment status |

---

## Setup Flow

### Step 1: Cloud Login

Authenticate with SystemPrompt Cloud using OAuth.

```json
// MCP: systemprompt
{ "command": "cloud auth login" }
```

**What it does:**
- Opens browser for GitHub/Google OAuth
- Saves credentials to `.systemprompt/credentials.json`
- Displays user info and available subscriptions

---

### Step 2: Tenant Setup

Link this project to a cloud tenant.

```json
// MCP: systemprompt
{ "command": "cloud tenant create --name my-project --region iad" }
{ "command": "cloud tenant list" }
{ "command": "cloud tenant" }
```

**What it does:**
- Lists existing tenants or creates new one
- Configures tenant in credentials file
- Polls for tenant provisioning (1-2 minutes)

---

### Step 3: Profile Configuration

Interactive wizard for complete environment setup.

```json
// MCP: systemprompt
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

---

### Step 4: Database Setup

Start the database and run migrations.

```bash
# Terminal: Start PostgreSQL (if using Docker)
just db-up

# MCP: Run migrations
{ "command": "infra db migrate" }
```

---

### Step 5: Verify

Check configuration status.

```json
// MCP: systemprompt
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
        │   ├── profile.yml
        │   ├── secrets.json
        │   └── docker-compose.yml
        └── production/
            ├── profile.yml
            └── secrets.json
```

### File Specifications

#### `credentials.json`

```json
{
  "api_token": "sp_live_abc123...",
  "api_url": "https://api.systemprompt.io",
  "user_email": "developer@example.com"
}
```

#### `tenants.json`

```json
{
  "selected": "ten_abc123",
  "tenants": [
    {
      "id": "ten_abc123",
      "name": "my-project",
      "app_id": "sp-my-project-abc",
      "hostname": "my-project.systemprompt.io",
      "region": "iad"
    }
  ]
}
```

#### `secrets.json`

```json
{
  "ANTHROPIC_API_KEY": "sk-ant-...",
  "OPENAI_API_KEY": "sk-...",
  "DATABASE_URL": "postgresql://..."
}
```

---

## Quickstart Recipes

### Local Development Only

```json
// MCP: systemprompt (run sequentially)
{ "command": "cloud init" }
```

```bash
# Terminal
just db-up
just migrate
just sync
just start
```

### Cloud Deployment

```json
// MCP: systemprompt (run sequentially)
{ "command": "cloud auth login" }
{ "command": "cloud tenant create --name my-project" }
{ "command": "cloud profile create local" }
{ "command": "cloud profile create production" }
```

```bash
# Terminal
just db-up
just migrate
just sync
just deploy
```

---

## Troubleshooting

**"Not logged in"**
```json
{ "command": "cloud auth login" }
```

**"No tenant configured"**
```json
{ "command": "cloud tenant" }
```

**"Profile required"**
```bash
export SYSTEMPROMPT_PROFILE=.systemprompt/profiles/local/profile.yml
```

**"Database connection failed"**
```bash
just db-up
just db-status
```

---

## Environment Variables

| Variable | Description |
|----------|-------------|
| `SYSTEMPROMPT_PROFILE` | Path to active profile |
| `DATABASE_URL` | PostgreSQL connection |
| `ANTHROPIC_API_KEY` | Claude API key |
| `OPENAI_API_KEY` | OpenAI API key |

---

## Security Notes

1. **Never commit secrets** -- `.systemprompt/` is gitignored
2. **Project-specific credentials** -- Each project has its own tenant
3. **Environment isolation** -- Each profile has its own secrets
4. **Token refresh** -- Re-run `login` if API calls fail with 401

---

## Quick Reference

| Task | Command |
|------|---------|
| Login | `cloud auth login` |
| Logout | `cloud auth logout` |
| Who am I | `cloud auth whoami` |
| List tenants | `cloud tenant list` |
| Create tenant | `cloud tenant create --name <name>` |
| Create profile | `cloud profile create <env>` |
| Check status | `cloud status` |
| Deploy | `cloud deploy` |
