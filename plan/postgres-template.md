# PostgreSQL Container Management - Template Repo

Work required in `systemprompt-template` repository.

## Summary

Update justfile to support per-tenant PostgreSQL containers.

## Files to Modify

### `justfile`

Update database commands to accept tenant name parameter:

```just
# ══════════════════════════════════════════════════════════════════════════════
# DATABASE — Local PostgreSQL (per tenant)
# ══════════════════════════════════════════════════════════════════════════════

# Start PostgreSQL for a specific tenant (default: local)
db-up TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yml up -d

# Stop PostgreSQL for a specific tenant
db-down TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yml down

# Show logs for a specific tenant's PostgreSQL
db-logs TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yml logs -f

# Reset database (stop, remove volume, start fresh)
db-reset TENANT="local":
    docker compose -f .systemprompt/docker/{{TENANT}}.yml down -v
    docker compose -f .systemprompt/docker/{{TENANT}}.yml up -d

# List all tenant databases
db-list:
    @ls -1 .systemprompt/docker/*.yml 2>/dev/null | xargs -I {} basename {} .yml || echo "No tenant databases found"
```

## Updated Directory Structure

```
.systemprompt/
├── credentials.json            ← Cloud auth
├── tenants.json                ← Tenant registry
├── Dockerfile                  ← App deployment image
├── docker/                     ← PostgreSQL containers
│   ├── local.yml               ← "local" tenant
│   └── dev.yml                 ← "dev" tenant (if created)
└── profiles/
    └── local/
        ├── profile.yml
        └── secrets.json
```

## Usage After Changes

```bash
# Default (local tenant)
just db-up
just db-down
just db-logs

# Specific tenant
just db-up dev
just db-down dev

# List all tenant databases
just db-list
```

## Checklist

- [x] Update `db-up` with TENANT parameter
- [x] Update `db-down` with TENANT parameter
- [x] Update `db-logs` with TENANT parameter
- [x] Update `db-reset` with TENANT parameter
- [x] Add `db-list` command
- [x] Update quickstart recipes if needed
