---
title: "Configuration & Credentials"
description: "How credentials management works in .systemprompt/ - sessions, tenants, profiles, secrets, and cloud deployment."
author: "SystemPrompt Team"
slug: "config"
keywords: "config, credentials, sessions, tenants, profiles, authentication, deployment, cloud"
image: "/files/images/docs/config.svg"
kind: "guide"
public: true
tags: ["config", "credentials", "authentication", "deployment"]
published_at: "2026-01-30"
updated_at: "2026-02-01"
after_reading_this:
  - "Understand how credentials management works in .systemprompt/"
  - "Know the relationship between sessions, tenants, and profiles"
  - "Authenticate with SystemPrompt Cloud via OAuth"
  - "Deploy and sync configuration to cloud"
related_playbooks:
  - title: "Session Management"
    url: "/playbooks/cli-session"
  - title: "Cloud Operations"
    url: "/playbooks/cli-cloud"
  - title: "Deployment Guide"
    url: "/playbooks/cli-deploy"
related_code:
  - title: "Configuration Manager"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/infra/config/src/services/manager.rs#L1-L50"
  - title: "Cloud CLI Commands"
    url: "https://github.com/systempromptio/systemprompt-core/blob/main/crates/entry/cli/src/commands/cloud/mod.rs#L1-L50"
---

# Configuration & Credentials

SystemPrompt uses a layered credential system stored in the `.systemprompt/` directory. This system manages authentication, multi-tenancy, and environment-specific configuration so you can run the same project in development, staging, and production with complete isolation.

## How It Works

The credential system follows a clear hierarchy. Understanding this flow is essential for managing SystemPrompt effectively.

**Cloud Login** creates your authentication token. This token identifies you across all SystemPrompt operations and enables access to cloud features.

**Tenants** are isolated environments that own databases and configuration. You might have a local tenant for development and a cloud tenant for production. Each tenant is completely separate.

**Profiles** are environment configurations within a tenant. A profile contains all the settings needed to run SystemPrompt: database connection, API keys, server configuration, and runtime options.

**Sessions** track your active CLI state. When you run commands, the session determines which profile is active and authenticates your requests.

## The Credential Flow

```
Cloud Login (credentials.json)
    │
    └── Authenticates you with SystemPrompt Cloud
            │
            ▼
Tenants (tenants.json)
    │
    └── Isolated environments (local or cloud)
            │
            ▼
Profiles (profiles/<name>/)
    │
    └── Environment config + secrets for each tenant
            │
            ▼
Sessions (sessions/index.json)
    │
    └── Active CLI state, determines which profile is used
```

## Directory Structure

The `.systemprompt/` directory contains all credential and configuration files. These files are gitignored by default to protect sensitive data.

```
.systemprompt/
├── credentials.json              # Cloud authentication (OAuth token)
├── tenants.json                  # Registry of all tenants
├── sessions/
│   └── index.json               # Active session state per tenant
├── profiles/
│   ├── local/
│   │   ├── profile.yaml         # Environment configuration
│   │   ├── secrets.json         # API keys, DATABASE_URL (gitignored)
│   │   └── docker/              # Local Docker compose files
│   └── production/
│       ├── profile.yaml
│       └── secrets.json
└── docker/
    └── shared.yaml              # Shared PostgreSQL container
```

## Quick Start

Get running with SystemPrompt in four steps.

**1. Authenticate with Cloud**

```bash
just login
```

Opens your browser for GitHub or Google OAuth. Creates `credentials.json` with your API token.

**2. Create or Select a Tenant**

```bash
systemprompt cloud tenant create --type local
# or
systemprompt cloud tenant list
systemprompt cloud tenant select <tenant-id>
```

Local tenants run PostgreSQL in Docker. Cloud tenants use managed infrastructure.

**3. Create a Profile**

```bash
systemprompt cloud profile create local
```

Generates `profiles/local/` with `profile.yaml` and `secrets.json`. The wizard prompts for database URL and API keys.

**4. Start Services**

```bash
just db-up      # Start PostgreSQL
just migrate    # Run database migrations
just start      # Start the server
```

Your session is now active. All CLI commands use the local profile by default.

## Bootstrap Sequence

SystemPrompt follows a strict 5-stage initialization sequence:

```
1. ProfileBootstrap   →  Load profile.yaml, validate
2. SecretsBootstrap   →  Load secrets (JWT, DATABASE_URL, API keys)
3. CredentialsBootstrap → Load cloud credentials (optional)
4. Config             →  Aggregate into runtime config
5. AppContext         →  Initialize database, services
```

See the [Bootstrap Sequence Playbook](/playbooks/config-bootstrap) for technical details.

## Configuration Sections

### Core Configuration

| Section | Purpose |
|---------|---------|
| [Profiles](/documentation/config/profiles) | Environment-specific settings |
| [Secrets](/documentation/config/secrets) | API keys and sensitive credentials |
| [Credentials](/documentation/config/credentials) | Cloud API authentication |
| [Database](/documentation/config/database) | PostgreSQL connection setup |

### Profile Sub-Configuration

| Section | Purpose |
|---------|---------|
| [Server](/documentation/config/server) | Host, port, API URLs, CORS |
| [Security](/documentation/config/security) | JWT issuer, token expiration |
| [Paths](/documentation/config/paths) | Directory layout |
| [Runtime](/documentation/config/runtime) | Environment, logging, output format |
| [Rate Limits](/documentation/config/rate-limits) | API throttling |

### Multi-Tenancy & Cloud

| Section | Purpose |
|---------|---------|
| [Tenants](/documentation/config/tenants) | Isolated environments (local and cloud) |
| [Sessions](/documentation/config/sessions) | CLI authentication state and profile switching |
| [Sync](/documentation/config/sync) | Push/pull configuration between environments |
| [Deployment](/documentation/config/deployment) | Deploy to SystemPrompt Cloud |
| [Docker](/documentation/config/docker) | Container configuration for local and cloud |
| [Domains](/documentation/config/domains) | Custom domain setup with TLS |

## Cloud Features

SystemPrompt Cloud is managed infrastructure for AI agents. Deploy with a single command, sync configuration between environments, and use custom domains with automatic TLS.

| Feature | Description |
|---------|-------------|
| **One-Command Deploy** | `systemprompt cloud deploy` pushes configuration to production |
| **Managed Database** | PostgreSQL with automatic backups |
| **Auto-Scaling** | Handle traffic spikes without configuration |
| **Custom Domains** | Use your domain with automatic Let's Encrypt certificates |
| **Code Sync** | Push and pull configuration between local and cloud |

### Free vs Paid

| Feature | Local (Free) | Cloud (Paid) |
|---------|--------------|--------------|
| AI agents | Unlimited | Unlimited |
| Database | Docker PostgreSQL | Managed PostgreSQL |
| Hosting | Your machine | SystemPrompt Cloud |
| Custom domain | localhost only | Any domain |
| TLS/HTTPS | Self-signed | Automatic certificates |
| Scaling | Manual | Automatic |

**Local development is free forever.** Cloud hosting is a paid service.

## Profile Priority

When you run a CLI command, SystemPrompt determines which profile to use in this order:

1. `--profile` flag on the command
2. `SYSTEMPROMPT_PROFILE` environment variable
3. Active session from `sessions/index.json`
4. Default profile

This lets you override the active profile for individual commands without switching sessions.

## Git Ignore

All sensitive files are automatically gitignored:

```
.systemprompt/credentials.json
.systemprompt/tenants.json
.systemprompt/sessions/
.systemprompt/profiles/*/secrets.json
```

Profile configuration (`profile.yaml`) can be committed since it contains no secrets. This makes it easy to share environment settings across a team while keeping credentials private.
