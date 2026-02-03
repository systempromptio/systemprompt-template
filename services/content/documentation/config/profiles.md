---
title: "Profiles"
description: "Environment configurations for SystemPrompt. Each profile contains all settings needed to run in a specific environment."
author: "SystemPrompt Team"
slug: "config/profiles"
keywords: "profiles, configuration, environments, local, production, settings"
image: "/files/images/docs/config-profiles.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Profiles

Profiles are environment configurations stored in `.systemprompt/profiles/<name>/`. Each profile contains everything needed to run SystemPrompt in a specific environment: database connection, server settings, security configuration, and API keys.

## Profiles and Tenants

Every profile belongs to exactly one tenant. The tenant provides the database and isolation boundary, while the profile configures how you interact with that tenant.

```yaml
# .systemprompt/profiles/local/profile.yaml
cloud:
  tenant_id: local_19bff27604c    # This profile belongs to this tenant
```

You can have multiple profiles per tenant. For example, a local tenant might have:
- `local` - Standard development settings
- `local-test` - Configured for running tests
- `local-verbose` - Extra logging for debugging

Each profile points to the same tenant but with different configuration.

## Profile Directory

Each profile has its own directory containing configuration and secrets:

```
.systemprompt/profiles/
├── local/
│   ├── profile.yaml       # Configuration (can be committed)
│   ├── secrets.json       # API keys, DATABASE_URL (gitignored)
│   └── docker/            # Docker compose files (local tenants)
│       ├── shared.yaml
│       └── systemprompt.yaml
└── production/
    ├── profile.yaml
    └── secrets.json
```

## Profile Structure

A complete profile configuration:

```yaml
# .systemprompt/profiles/local/profile.yaml
name: local
display_name: "Local Development"
target: local                    # "local" or "cloud"
environment: development         # development, staging, production

# Site identity
site:
  name: "My Project"
  github_link: "https://github.com/org/repo"

# Database configuration
database:
  type: postgres
  external_db_access: true       # Allow external connections

# Server settings
server:
  host: "0.0.0.0"
  port: 8080
  api_server_url: "http://localhost:8080"
  https:
    enabled: false
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"

# File paths
paths:
  system: "/path/to/project"
  services: "/path/to/project/services"
  bin: "/path/to/project/target/release"
  web_path: "/path/to/project/web"
  storage: "/path/to/project/storage"

# Security settings
security:
  jwt:
    issuer: "systemprompt-local"
    access_token_expiration: 2592000    # 30 days
    refresh_token_expiration: 15552000  # 180 days
    audiences: ["web", "api", "a2a", "mcp"]
  validation_level: "warn"               # "warn" or "strict"

# Rate limiting
rate_limits:
  disabled: true                         # Disable for development

# Runtime configuration
runtime:
  log_level: "verbose"                   # trace, debug, info, warn, error
  output_format: "pretty"                # "pretty" or "json"
  colors: true

# Tenant linkage
cloud:
  credentials_path: "../../credentials.json"
  tenants_path: "../../tenants.json"
  tenant_id: local_19bff27604c

# Secrets reference
secrets:
  path: "./secrets.json"
  validation_mode: "warn"
```

## Configuration Sections

### Database

Database connection is configured in the profile but the actual `DATABASE_URL` is stored in secrets.

```yaml
database:
  type: postgres
  external_db_access: true    # Allow connections from outside Docker
```

See [Database](/documentation/config/database) for connection string formats.

### Server

Controls how the HTTP server runs. See [Server Configuration](/documentation/config/server) for details.

```yaml
server:
  host: "0.0.0.0"                              # Bind address
  port: 8080                                   # Listen port
  api_server_url: "http://localhost:8080"      # Primary API URL
  api_internal_url: "http://localhost:8080"    # Internal service URL
  api_external_url: "http://localhost:8080"    # Public/external URL
  use_https: false                             # Enable for production
  cors_allowed_origins:
    - "http://localhost:8080"
    - "http://localhost:5173"
```

**Note**: All three API URLs are required (`api_server_url`, `api_internal_url`, `api_external_url`).

### Security

JWT tokens and validation settings. See [Security Configuration](/documentation/config/security) for details.

```yaml
security:
  jwt_issuer: "systemprompt-local"
  jwt_access_token_expiration: 2592000       # 30 days in seconds
  jwt_refresh_token_expiration: 15552000     # 180 days
  jwt_audiences:
    - web
    - api
    - a2a
    - mcp
```

**Note**: Access token expiration maximum is 31,536,000 seconds (1 year).

### Rate Limits

Protect your API from abuse. See [Rate Limits Configuration](/documentation/config/rate-limits) for details.

```yaml
rate_limits:
  disabled: false                  # Enable in production
  oauth_public_per_second: 10
  contexts_per_second: 100
  agents_per_second: 20
  mcp_per_second: 200
  burst_multiplier: 3
  tier_multipliers:
    admin: 10.0
    user: 1.0
    a2a: 5.0
    mcp: 5.0
    service: 5.0
    anon: 0.5
```

Disable rate limits in development to avoid interference during testing.

### Runtime

Logging and output configuration. See [Runtime Configuration](/documentation/config/runtime) for details.

```yaml
runtime:
  environment: development        # development, test, staging, production
  log_level: verbose              # quiet, normal, verbose, debug
  output_format: text             # text, json, yaml
  no_color: false                 # Disable colors
  non_interactive: false          # Disable prompts
```

Use `json` output format in production for structured logging.

## Production Profile

A typical production profile:

```yaml
name: production
display_name: "Production"
target: cloud
environment: production

server:
  api_server_url: "https://your-domain.com"
  https:
    enabled: true

security:
  validation_level: "strict"

rate_limits:
  disabled: false

runtime:
  log_level: "info"
  output_format: "json"
  colors: false

cloud:
  tenant_id: 999bc654-9a64-49bc-98be-db976fc84e76
```

## Create a Profile

Use the CLI wizard to create a new profile.

```bash
systemprompt cloud profile create staging
```

The wizard prompts for:
- Environment type (development, staging, production)
- Server URL and port
- Database connection
- API keys

For non-interactive creation:

```bash
systemprompt cloud profile create staging --environment staging
```

## List Profiles

View all profiles in your project.

```bash
systemprompt cloud profile list
```

Output:

```
Profiles
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  local          Local Development
  staging        Staging Environment
  production     Production
```

## Show Profile Details

View a profile's configuration.

```bash
systemprompt cloud profile show local
```

## Edit a Profile

Modify an existing profile.

```bash
systemprompt cloud profile edit local
```

Opens the profile in your editor, or use the interactive wizard:

```bash
systemprompt cloud profile edit local --interactive
```

## Switch Profiles

Change the active profile for your CLI session.

```bash
# Via session command
systemprompt admin session switch staging

# Via environment variable
export SYSTEMPROMPT_PROFILE=~/.systemprompt/profiles/staging/profile.yaml

# Per-command override
systemprompt admin agents list --profile production
```

See [Sessions](/documentation/config/sessions) for the full profile priority order.

## Delete a Profile

Remove a profile and its configuration.

```bash
systemprompt cloud profile delete staging -y
```

This removes the profile directory but does not affect the tenant or database.

## Secrets

Each profile has a `secrets.json` file for sensitive values.

```json
{
  "database_url": "postgres://user:pass@localhost:5432/systemprompt",
  "anthropic_api_key": "sk-ant-...",
  "openai_api_key": "sk-...",
  "gemini_api_key": "AIza...",
  "github_token": "ghp_..."
}
```

Secrets are referenced in `profile.yaml` via the `secrets.path` setting and are always gitignored.

See [Secrets](/documentation/config/secrets) for managing API keys and credentials.