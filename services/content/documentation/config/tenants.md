---
title: "Tenants"
description: "Isolated environments for SystemPrompt. Local tenants run on your machine, cloud tenants run on managed infrastructure."
author: "SystemPrompt Team"
slug: "config/tenants"
keywords: "tenants, multi-tenancy, isolation, local, cloud, environments"
image: "/files/images/docs/config-tenants.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Tenants

Tenants are isolated environments that own databases and configuration. Each tenant is completely separate from others, with its own data, users, and settings. You can run multiple tenants simultaneously for development, staging, and production.

## Why Tenants

Multi-tenancy solves a fundamental problem: keeping environments truly isolated. Without tenants, it's too easy to accidentally run production commands in development or mix data between environments.

Tenants provide:
- **Database isolation** - Each tenant has its own PostgreSQL database
- **Configuration isolation** - Settings don't leak between environments
- **Credential isolation** - API keys and secrets are tenant-specific
- **Session isolation** - CLI state is tracked per-tenant

## Tenant Types

SystemPrompt supports two types of tenants.

### Local Tenants

Local tenants run on your machine using Docker. They're free and ideal for development.

- PostgreSQL runs in a Docker container
- Data is stored locally
- No network dependencies
- Perfect for offline development

### Cloud Tenants

Cloud tenants run on SystemPrompt's managed infrastructure. They're designed for production.

- Managed PostgreSQL with automatic backups
- TLS certificates included
- Accessible via `<tenant-id>.systemprompt.io`
- Requires a paid subscription

## Tenant Registry

All tenants are tracked in `.systemprompt/tenants.json`. This file is synced with SystemPrompt Cloud and includes both local and cloud tenants.

```json
{
  "tenants": [
    {
      "id": "local_19bff27604c",
      "name": "my-project",
      "tenant_type": "local",
      "database_url": "postgres://systemprompt:localdev@localhost:5432/systemprompt",
      "external_db_access": false,
      "shared_container_db": "systemprompt-shared-db"
    },
    {
      "id": "999bc654-9a64-49bc-98be-db976fc84e76",
      "name": "my-project-prod",
      "tenant_type": "cloud",
      "app_id": "sp-999bc6549a64",
      "hostname": "999bc6549a64.systemprompt.io",
      "region": "iad",
      "internal_database_url": "postgres://user:pass@internal-db:5432/tenant",
      "sync_token": "sp_sync_abc123..."
    }
  ],
  "synced_at": "2026-02-01T10:00:00Z"
}
```

### StoredTenant Fields

| Field | Local | Cloud | Description |
|-------|-------|-------|-------------|
| `id` | Yes | Yes | Unique tenant identifier |
| `name` | Yes | Yes | Display name |
| `tenant_type` | `local` | `cloud` | Tenant type |
| `database_url` | Yes | No | Local PostgreSQL URL |
| `internal_database_url` | No | Yes | Cloud internal database URL |
| `external_db_access` | Yes | No | Allow external connections |
| `shared_container_db` | Optional | No | Shared Docker container name |
| `app_id` | No | Yes | Fly.io app ID |
| `hostname` | No | Yes | Cloud hostname |
| `region` | No | Yes | Deployment region |
| `sync_token` | Optional | Yes | Sync authentication token |

## Create a Tenant

### Local Tenant

Create a local tenant for development. This sets up PostgreSQL in Docker.

```bash
systemprompt cloud tenant create --type local
```

The CLI prompts for:
- **Name**: Human-readable identifier (e.g., "my-project")
- **Database URL**: Connection string for PostgreSQL

For a quick start with defaults:

```bash
systemprompt cloud tenant create --type local --name my-project
```

### Cloud Tenant

Create a cloud tenant for production. Requires authentication with SystemPrompt Cloud.

```bash
just login
systemprompt cloud tenant create --region iad
```

Available regions:
- `iad` - US East (Virginia)
- `lhr` - Europe (London)
- `syd` - Asia Pacific (Sydney)

Cloud tenant creation takes 1-2 minutes while infrastructure provisions.

## List Tenants

View all tenants registered in your project.

```bash
systemprompt cloud tenant list
```

Output:

```
Tenants
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  local_19bff27604c    (selected) local   my-project
  999bc654-9a64...     cloud   my-project-prod   iad
```

## Select a Tenant

Switch the active tenant. This affects which profiles are available.

```bash
systemprompt cloud tenant select local_19bff27604c
```

After selecting a tenant, create or switch to a profile within that tenant.

## Show Tenant Details

View detailed information about a tenant.

```bash
# Show selected tenant
systemprompt cloud tenant show

# Show specific tenant
systemprompt cloud tenant show 999bc654-9a64-49bc-98be-db976fc84e76
```

Output for cloud tenants includes hostname, region, and status.

## Rotate Credentials

Regenerate API keys and sync tokens for a tenant. Use this if credentials are compromised.

```bash
# Rotate API credentials
systemprompt cloud tenant rotate-credentials <tenant-id> -y

# Rotate sync token
systemprompt cloud tenant rotate-sync-token <tenant-id> -y
```

After rotation, update any external systems that use these credentials.

## Delete a Tenant

Remove a tenant and all its data. This action is irreversible.

```bash
systemprompt cloud tenant delete <tenant-id> -y
```

For cloud tenants, this also cancels the subscription:

```bash
systemprompt cloud tenant cancel <tenant-id> -y
```

## Tenants and Profiles

Tenants and profiles work together. A profile belongs to exactly one tenant, specified in `profile.yaml`:

```yaml
# profiles/local/profile.yaml
name: local
target: local

cloud:
  tenant_id: local_19bff27604c
  credentials_path: ../../credentials.json
  tenants_path: ../../tenants.json
```

When you create a profile, it's linked to the currently selected tenant. You can have multiple profiles per tenant (e.g., `local-dev`, `local-test`) but each profile points to exactly one tenant.

```
Tenant: local_19bff27604c
├── Profile: local
├── Profile: local-test
└── Profile: local-integration

Tenant: 999bc654-... (cloud)
├── Profile: staging
└── Profile: production
```

## Tenants and Sessions

Sessions are tenant-keyed. When you switch tenants, you switch to that tenant's session:

1. Select tenant A → Profile A's session is active
2. Select tenant B → Profile B's session is active (completely separate)
3. Switch back to tenant A → Profile A's session is restored

This means you can work in multiple tenants without cross-contamination. Each tenant tracks its own authentication, permissions, and state.

## Local Tenant Setup

Local tenants require PostgreSQL. The easiest approach is Docker:

```bash
# Start shared PostgreSQL container
just db-up

# Verify it's running
docker ps | grep postgres
```

The database connection is stored in the tenant registry and profile secrets. See [Database](/documentation/config/database) for connection options.

## Cloud Tenant Setup

Cloud tenants are fully managed. After creation:

1. **Wait for provisioning** - Takes 1-2 minutes
2. **Create a profile** - `systemprompt cloud profile create production`
3. **Add secrets** - API keys are required for AI features
4. **Run migrations** - `systemprompt infra db migrate`

Your cloud tenant is now ready at `https://<tenant-id>.systemprompt.io`.

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "No tenant configured" | No tenant selected | Run `cloud tenant list` then `cloud tenant select <id>` |
| "Tenant not found" | Invalid tenant ID | Check `tenants.json` for valid IDs |
| "Database connection failed" | PostgreSQL not running | Run `just db-up` for local tenants |
| "Provisioning failed" | Cloud infrastructure issue | Check `cloud tenant show` for status |
| "Access denied" | Credential issue | Run `cloud tenant rotate-credentials` |