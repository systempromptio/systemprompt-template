---
title: "Docker & Container Management"
description: "Manage PostgreSQL containers for local development and multi-project setups."
author: "SystemPrompt"
slug: "build-cloud-docker"
keywords: "docker, postgresql, containers, local development, tenant, shared"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-03"
updated_at: "2026-02-03"
---

# Docker & Container Management

Manage PostgreSQL containers for local development and multi-project setups.

> **Help**: `{ "command": "cloud tenant --help" }`
> **Requires**: Docker running -> `docker info`

---

## Container Architecture

SystemPrompt uses a shared PostgreSQL container across multiple projects:

| Component | Location | Purpose |
|-----------|----------|---------|
| Container | `systemprompt-postgres-shared` | Single PostgreSQL instance |
| Volume | `systemprompt-postgres-shared-data` | Persisted database data |
| Config | `.systemprompt/docker/shared_config.json` | Per-project password storage |
| Compose | `.systemprompt/docker/shared.yaml` | Docker compose definition |

---

## Create Local Tenant

```bash
systemprompt cloud tenant
```

Interactive flow:
1. Select "Local (creates PostgreSQL container automatically)"
2. Enter tenant name
3. Enter profile name
4. Provide at least one AI provider API key

---

## Multi-Project Scenarios

### Scenario 1: First Project

Container doesn't exist → creates new container with generated password.

```
Project A:
├── .systemprompt/docker/shared.yaml          (compose file)
├── .systemprompt/docker/shared_config.json   (password stored here)
└── .systemprompt/profiles/local/             (profile config)
```

### Scenario 2: Second Project (Container Running)

Container exists but no local config → prompts for action:

```
⚠ Shared PostgreSQL container is running but no local configuration found.
ℹ This container may be managed by another systemprompt project.

? How would you like to proceed?
> Add tenant to existing container (requires password)
  Create new isolated container for this project
  Cancel
```

**To add tenant to existing container:**
1. Get password from first project: `cat /path/to/project-a/.systemprompt/docker/shared_config.json`
2. Copy `admin_password` value
3. Enter password when prompted
4. New tenant database created in shared container

### Scenario 3: Orphaned Volume

Container stopped, config deleted, but volume exists → prompts for reset:

```
⚠ PostgreSQL data volume exists but no container or configuration found.
? Reset volume? (This will delete existing database data) (y/N)
```

---

## Container States

| Config Exists | Container Running | Action |
|---------------|-------------------|--------|
| Yes | Yes | Use existing (no restart) |
| Yes | No | Restart container |
| No | Yes | Prompt: join existing or cancel |
| No | No | Check volume, create new |

---

## Password Management

Password is generated once during initial container creation and stored in `shared_config.json`:

```json
{
  "admin_password": "1a2b3c4d5e6f7890abcdef1234567890",
  "port": 5432,
  "created_at": "2026-02-03T10:00:00Z",
  "tenant_databases": [
    { "tenant_id": "local_abc123", "database_name": "myproject" }
  ]
}
```

**PostgreSQL volume behavior**: Password is only read during initial volume creation. Changing `POSTGRES_PASSWORD` in compose file does NOT update existing credentials.

---

## Manual Operations

### Start Container

```bash
docker compose -f .systemprompt/docker/shared.yaml up -d
```

### Stop Container

```bash
docker compose -f .systemprompt/docker/shared.yaml down
```

### View Logs

```bash
docker logs systemprompt-postgres-shared
```

### Connect to PostgreSQL

```bash
docker exec -it systemprompt-postgres-shared psql -U systemprompt_admin -d postgres
```

### List Databases

```bash
docker exec systemprompt-postgres-shared psql -U systemprompt_admin -d postgres -c "\l"
```

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Password authentication failed | Volume has different password than config | Reset volume or use correct password |
| Container recreated unexpectedly | Another project created new compose | Get password from original project |
| Connection refused | Container not running | `docker compose -f .systemprompt/docker/shared.yaml up -d` |
| Volume orphaned | Container and config deleted | Reset volume or remove manually |

### Reset Volume (Destructive)

```bash
docker compose -f .systemprompt/docker/shared.yaml down -v
docker volume rm systemprompt-postgres-shared-data
rm .systemprompt/docker/shared_config.json
```

### Check Container Status

```bash
docker ps -f name=systemprompt-postgres-shared
```

### Check Volume Exists

```bash
docker volume ls -f name=systemprompt-postgres-shared-data
```

### Verify Password

```bash
cat .systemprompt/docker/shared_config.json | jq -r '.admin_password'
```

---

## Core Code Reference

| File | Purpose |
|------|---------|
| `crates/entry/cli/src/commands/cloud/tenant/create.rs` | Tenant creation flow |
| `crates/entry/cli/src/commands/cloud/tenant/docker.rs` | Container management |
| `crates/entry/cli/src/commands/cloud/profile/create_setup.rs` | Profile setup |

---

## Quick Reference

| Task | Command |
|------|---------|
| Create tenant | `systemprompt cloud tenant` |
| Start PostgreSQL | `docker compose -f .systemprompt/docker/shared.yaml up -d` |
| Stop PostgreSQL | `docker compose -f .systemprompt/docker/shared.yaml down` |
| View container logs | `docker logs systemprompt-postgres-shared` |
| List databases | `docker exec systemprompt-postgres-shared psql -U systemprompt_admin -d postgres -c "\l"` |
| Check password | `cat .systemprompt/docker/shared_config.json \| jq -r '.admin_password'` |
| Reset everything | `docker compose -f .systemprompt/docker/shared.yaml down -v` |

---

## Related

-> See [Docker Configuration](/docs/config/docker) for full Docker documentation
-> See [Tenant Management](tenant.md) for tenant lifecycle
