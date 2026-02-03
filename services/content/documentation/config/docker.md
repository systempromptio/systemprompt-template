---
title: "Docker Configuration"
description: "Docker serves two purposes in SystemPrompt: running PostgreSQL locally and deploying your application to production."
author: "SystemPrompt Team"
slug: "config/docker"
keywords: "docker, containers, deployment, production, dockerfile, postgresql, local development"
image: "/files/images/docs/config-docker.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Docker Configuration

Docker serves two distinct purposes in SystemPrompt: running PostgreSQL for local development and deploying your application to production. Understanding which Docker files to use and when is essential for a smooth development workflow.

## PostgreSQL Containers

SystemPrompt uses Docker to run PostgreSQL locally. You have two options depending on your setup: a shared container for multiple projects, or tenant-specific containers for isolated development.

### Shared PostgreSQL

The shared PostgreSQL container allows multiple SystemPrompt projects to share a single database server. This is efficient when you're working on several projects simultaneously and want to minimize resource usage.

The configuration lives in `.systemprompt/docker/shared.yaml`. When you run `just db-up`, this container starts and exposes PostgreSQL on port 5432. A companion file, `shared_config.json`, tracks which tenant databases exist within this shared server.

```bash
# Start shared PostgreSQL
just db-up

# Stop shared PostgreSQL
just db-down

# Reset shared PostgreSQL (removes all data)
just db-reset

# View PostgreSQL logs
just db-logs

# List all tenant databases in shared server
just db-list
```

Each tenant gets its own database within the shared server. When you create a new local tenant with `systemprompt cloud tenant create`, it automatically registers a database in the shared configuration. Your profile's `DATABASE_URL` in `secrets.json` points to this specific database.

### Tenant-Specific PostgreSQL

For isolated development, each tenant can have its own dedicated PostgreSQL container. This is useful when you need different PostgreSQL versions or configurations per project, or when you want complete isolation between environments.

Tenant-specific configurations are generated automatically in `.systemprompt/docker/{TENANT_NAME}.yaml` when you create a tenant. Each container uses a named volume to persist data across restarts.

```bash
# Start a specific tenant's PostgreSQL
just db-up TENANT="my-project"

# Stop a specific tenant's PostgreSQL
just db-down TENANT="my-project"
```

The tenant-specific approach uses more resources but provides complete isolation. Changes to one project's database cannot affect another.

## Application Deployment

Beyond database containers, Docker is used to package your SystemPrompt application for deployment. Profile-specific Dockerfiles define how your application is built and run in production.

### Profile Dockerfiles

Each profile can include its own Dockerfile in `.systemprompt/profiles/{profile-name}/docker/`. This allows different deployment configurations for development versus production.

A local profile Dockerfile might include debugging tools and skip certain optimizations. A production profile Dockerfile focuses on minimal image size, security hardening, and proper health checks.

The Dockerfile structure follows a consistent pattern:

```dockerfile
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY target/release/systemprompt /app/bin/
COPY services/ /app/services/
COPY web/ /app/web/

ENV HOST=0.0.0.0 PORT=8080
EXPOSE 8080
CMD ["/app/bin/systemprompt", "infra", "services", "start", "--all"]
```

### Entrypoint Scripts

Each profile includes an `entrypoint.sh` script that runs before your application starts. This typically handles database migrations and any pre-flight checks:

```bash
#!/bin/sh
set -e
echo "Running database migrations..."
/app/bin/systemprompt infra db migrate
echo "Starting services..."
exec /app/bin/systemprompt infra services serve --foreground
```

The entrypoint ensures your database schema is up-to-date before the application accepts traffic.

### Building Images Locally

Test your Docker build locally before deploying to production:

```bash
# Build the release binary first
cargo build --release -p systemprompt-cli

# Build MCP servers if using them
just build-mcp

# Build Docker image
just docker-build local

# Run the image locally
just docker-run local
```

This workflow validates that your Dockerfile works correctly and your application starts as expected.

### Cloud Deployment

When you run `just deploy`, the CLI orchestrates the full deployment workflow:

1. Builds the Rust binary with release optimizations
2. Builds any MCP servers your application uses
3. Uses the active profile's Dockerfile to build an image
4. Pushes the image to the container registry
5. Deploys to your cloud tenant

```bash
# Full deployment workflow
just deploy

# Deploy a specific profile
systemprompt cloud deploy --profile production --yes

# Redeploy without rebuilding (uses existing image)
systemprompt cloud deploy --profile production --yes --skip-push
```

The deployment process respects your profile's Dockerfile, ensuring environment-specific configurations are correctly applied.

## Environment Variables

Docker containers receive configuration through environment variables. These override defaults and connect your application to external services.

| Variable | Description | Required |
|----------|-------------|----------|
| `DATABASE_URL` | PostgreSQL connection string | Yes |
| `HOST` | Bind address (default: 0.0.0.0) | No |
| `PORT` | Server port (default: 8080) | No |
| `RUST_LOG` | Log level (info, debug, trace) | No |
| `SYSTEMPROMPT_PROFILE` | Path to active profile | No |
| `SYSTEMPROMPT_SERVICES_PATH` | Path to services directory | No |

Pass environment variables when running containers:

```bash
docker run -p 8080:8080 \
  -e DATABASE_URL="postgres://user:pass@host:5432/db" \
  -e RUST_LOG="info" \
  myproject:latest
```

## Docker Compose for Development

For local development with both the application and PostgreSQL, use Docker Compose:

```yaml
# docker-compose.yaml
services:
  app:
    build: .
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: postgres://systemprompt:localdev@postgres:5432/systemprompt
    depends_on:
      - postgres

  postgres:
    image: postgres:17-alpine
    environment:
      POSTGRES_USER: systemprompt
      POSTGRES_PASSWORD: localdev
      POSTGRES_DB: systemprompt
    volumes:
      - pgdata:/var/lib/postgresql/data

volumes:
  pgdata:
```

```bash
docker compose up -d
```

This approach bundles everything needed to run your application in a single command.

## Production Considerations

### Multi-stage Builds

Reduce image size with multi-stage builds that separate compilation from runtime:

```dockerfile
# Build stage
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release -p systemprompt-cli

# Runtime stage
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/systemprompt /app/bin/
COPY services/ /app/services/
COPY web/ /app/web/
ENV HOST=0.0.0.0 PORT=8080
EXPOSE 8080
CMD ["/app/bin/systemprompt", "infra", "services", "start", "--all"]
```

### Health Checks

Add health checks so orchestrators can detect unhealthy containers:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/api/v1/health || exit 1
```

### Secrets Management

Never bake secrets into Docker images. Pass them at runtime through environment variables or a secrets manager:

```bash
# Environment variable (simple)
docker run -e DATABASE_URL="$DATABASE_URL" myproject:latest

# Docker secrets (Swarm mode)
docker secret create db_url ./db_url.txt
docker service create --secret db_url myproject:latest
```

## Directory Structure

Docker-related files are organized within `.systemprompt/`:

```
.systemprompt/
├── Dockerfile                    # Template for local testing
├── .dockerignore                 # Files excluded from build context
├── entrypoint.sh                 # Root entrypoint script
├── docker/
│   ├── shared.yaml              # Shared PostgreSQL container
│   ├── shared_config.json       # Tracks databases in shared server
│   └── {TENANT}.yaml            # Tenant-specific PostgreSQL
└── profiles/
    ├── local/
    │   └── docker/
    │       ├── Dockerfile       # Local profile build
    │       ├── Dockerfile.dockerignore
    │       └── entrypoint.sh
    └── production/
        └── docker/
            ├── Dockerfile       # Production profile build
            ├── Dockerfile.dockerignore
            └── entrypoint.sh
```

## Quick Reference

| Task | Command |
|------|---------|
| Start shared PostgreSQL | `just db-up` |
| Stop shared PostgreSQL | `just db-down` |
| Reset shared PostgreSQL | `just db-reset` |
| View PostgreSQL logs | `just db-logs` |
| Start tenant PostgreSQL | `just db-up TENANT="name"` |
| Build Docker image | `just docker-build local` |
| Run Docker image | `just docker-run local` |
| Deploy to cloud | `just deploy` |
| View profile Dockerfile | `systemprompt cloud dockerfile` |