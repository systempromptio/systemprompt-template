---
title: "Configuration & Profiles"
description: "Profile-based configuration for the Foodles AI governance platform. Manage environment-specific settings for local development, staging, and production deployments."
author: "systemprompt.io"
slug: "configuration"
keywords: "configuration, profiles, environment variables, config.yaml, secrets, settings"
kind: "guide"
public: true
tags: ["devops", "configuration", "profiles"]
published_at: "2026-03-25"
updated_at: "2026-03-25"
after_reading_this:
  - "Understand the profile system and how to switch between environments"
  - "Know the configuration file hierarchy and override order"
  - "Configure database connections, logging, and security settings per environment"
related_docs:
  - title: "Installation & Setup"
    url: "/documentation/installation"
  - title: "Deployment Models"
    url: "/documentation/deployment-models"
  - title: "Scaling Architecture"
    url: "/documentation/scaling"
---

# Configuration & Profiles

The platform uses a profile-based configuration system that separates environment-specific settings from application logic. Switch between local development, staging, and production with a single environment variable.

## Profile System

A profile is a directory under `services/profiles/` containing environment-specific configuration:

```
services/profiles/
  local/
    config.yaml          # Local development settings
    docker/
      Dockerfile         # Development container
      docker-compose.yml # Local service stack
  production/
    config.yaml          # Production settings
    docker/
      Dockerfile         # Production container
```

Set the active profile with the `PROFILE` environment variable:

```bash
PROFILE=local systemprompt infra services start
PROFILE=production systemprompt infra services start
```

## Configuration Hierarchy

Settings are resolved in order, with later sources overriding earlier ones:

1. **Default values** — Built into the binary
2. **Profile config.yaml** — Environment-specific settings from the active profile
3. **Site config.yaml** — Site-level configuration at `services/web/config.yaml`
4. **Environment variables** — Override any setting at runtime

## Core Configuration

### Database

```yaml
database:
  url: "postgresql://user:pass@localhost:5432/foodles"
  max_connections: 20
  min_connections: 5
  connect_timeout_secs: 30
```

The database URL can be overridden with the `DATABASE_URL` environment variable, which takes precedence over config files.

### Server

```yaml
server:
  host: "0.0.0.0"
  port: 3000
  workers: 4
```

### Logging

```yaml
logging:
  level: "info"        # trace, debug, info, warn, error
  format: "json"       # json or pretty
```

Local profiles typically use `level: "debug"` with `format: "pretty"`. Production profiles use `level: "info"` with `format: "json"` for structured log aggregation.

## Secrets Management

Secrets are stored encrypted in the database using ChaCha20-Poly1305. The encryption key is provided via the `ENCRYPTION_KEY` environment variable. Never commit encryption keys to configuration files.

Plugin-level secrets (API keys, OAuth credentials) are managed through the admin interface or CLI:

```bash
systemprompt cloud secrets list
systemprompt cloud secrets set MY_API_KEY --secret
```

See [Secrets & Encryption](/documentation/secrets) for details on key rotation and access auditing.

## Custom Profiles

Create a custom profile by adding a new directory under `services/profiles/`:

```bash
mkdir -p services/profiles/staging/docker
```

Add a `config.yaml` with environment-specific overrides. Only the settings you want to change need to be specified — all other settings inherit from defaults.

## Environment Variables

Any configuration value can be overridden with environment variables. Common overrides:

| Variable | Purpose |
|----------|---------|
| `PROFILE` | Active profile name |
| `DATABASE_URL` | PostgreSQL connection string |
| `ENCRYPTION_KEY` | Secret encryption key |
| `PORT` | Server port |
| `RUST_LOG` | Logging level override |
| `BASE_URL` | Public-facing URL |
