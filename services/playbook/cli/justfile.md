---
title: "Justfile Playbook"
description: "Human-only interactive commands for build, auth, tenants, profiles, and deployment."
keywords:
  - justfile
  - build
  - login
  - deploy
  - interactive
category: cli
---

# Justfile

Human-only interactive commands. For all other operations, use playbooks.

> **Note**: These commands require terminal interaction (browser auth, prompts).
> Agents should use playbooks instead: `systemprompt core playbooks show <id>`

---

## When to Use Justfile vs Playbooks

| Use Justfile | Use Playbooks |
|--------------|---------------|
| Interactive commands (login, tenant setup) | Agent-executable operations |
| Building the project | Querying data |
| Deploying to cloud | Managing content |
| Database container management | Service operations |

---

## Build

```bash
just build              # Build (auto-detects offline mode)
just build --release    # Release build
```

---

## Authentication

```bash
just login              # Login (opens browser)
just logout             # Logout
just whoami             # Check current session
```

---

## Tenant Management

```bash
just tenant                                                  # Interactive tenant menu
just tenant create --region iad                              # Cloud tenant
just tenant create --database-url postgres://...             # Local tenant
just tenant list
just tenant show
```

---

## Profile Management

```bash
just profile             # Interactive profile menu
just profile create local
just profile create production
just profiles            # List all profiles
```

---

## Database (Docker)

```bash
just db-up               # Start PostgreSQL container
just db-down             # Stop container
just db-reset            # Reset (delete data + restart)
just db-logs             # View container logs
just db-list             # List tenant databases
```

---

## Migrations & Services

```bash
just migrate             # Run database migrations
just start               # Start all services
```

---

## Deployment

```bash
just deploy                        # Build + deploy to cloud
just deploy --profile production   # Deploy specific profile
```

---

## Docker

```bash
just docker-build        # Build Docker image
just docker-run          # Run Docker container
just docker-test         # Build and test Docker image
```

---

## Passthrough

Any command not defined passes through to the CLI:

```bash
just infra services status    # Runs: systemprompt infra services status
just admin agents list        # Runs: systemprompt admin agents list
```

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `just build` |
| Build release | `just build --release` |
| Login | `just login` |
| Logout | `just logout` |
| Create tenant | `just tenant create --database-url postgres://...` |
| Create profile | `just profile create <name>` |
| Start database | `just db-up` |
| Run migrations | `just migrate` |
| Start services | `just start` |
| Deploy | `just deploy` |

-> For agent-executable operations, see playbooks: `systemprompt core playbooks list`
