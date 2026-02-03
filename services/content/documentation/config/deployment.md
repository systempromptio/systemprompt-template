---
title: "Deployment"
description: "Deploy AI agents to SystemPrompt Cloud with a single command. CI/CD integration, zero-downtime deployments, and rollback strategies."
author: "SystemPrompt Team"
slug: "config/deployment"
keywords: "deployment, deploy, ci/cd, production, rollback"
image: "/files/images/docs/config-deployment.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Deployment

Deploy your SystemPrompt application to the cloud with a single command. The deployment process builds a Docker image, pushes it to the registry, and starts your services.

## Deployment Pipeline

```
┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐    ┌──────────┐
│ Validate │───▶│  Build   │───▶│   Push   │───▶│  Deploy  │───▶│   Sync   │
│  Config  │    │  Image   │    │ Registry │    │ Services │    │ Secrets  │
└──────────┘    └──────────┘    └──────────┘    └──────────┘    └──────────┘
```

## Basic Deployment

### Deploy to Production

```bash
systemprompt cloud deploy --profile production
```

### Deploy with Options

```bash
# Skip building (use existing image)
systemprompt cloud deploy --profile production --skip-build

# Force rebuild
systemprompt cloud deploy --profile production --force

# Dry run (show what would happen)
systemprompt cloud deploy --profile production --dry-run
```

## What Happens During Deploy

1. **Validate** - Check local configuration for errors
2. **Build** - Create Docker image with compiled Rust code
3. **Push** - Upload image to SystemPrompt registry
4. **Deploy** - Start new containers, drain old ones
5. **Sync** - Push secrets and configuration to cloud database

## Deployment Options

| Flag | Description |
|------|-------------|
| `--profile <name>` | Profile to deploy (required) |
| `--skip-build` | Skip Docker build, use existing image |
| `--force` | Force rebuild even if no changes |
| `--dry-run` | Show what would happen without executing |
| `--tag <tag>` | Custom image tag |

## Checking Status

```bash
# Current deployment status
systemprompt cloud status

# Deployment logs
systemprompt cloud logs

# Follow logs
systemprompt cloud logs -f
```

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy to Production
on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install SystemPrompt CLI
        run: cargo install systemprompt

      - name: Login to Cloud
        run: systemprompt cloud auth login --token ${{ secrets.SP_TOKEN }}

      - name: Deploy
        run: systemprompt cloud deploy --profile production
```

### GitLab CI Example

```yaml
deploy:
  stage: deploy
  script:
    - cargo install systemprompt
    - systemprompt cloud auth login --token $SP_TOKEN
    - systemprompt cloud deploy --profile production
  only:
    - main
```

## Rollback Strategies

### Immediate Rollback

```bash
# Rollback to previous deployment
systemprompt cloud rollback

# Rollback to specific version
systemprompt cloud rollback --version v1.2.3
```

### View Deployment History

```bash
systemprompt cloud history
```

## Cloud Infrastructure Architecture

Understanding the SystemPrompt Cloud architecture helps diagnose deployment and connectivity issues.

### Multi-Tenant Architecture

```
                        ┌─────────────────────────────────┐
                        │       DNS (Cloudflare)          │
                        │   *.systemprompt.io → Proxy     │
                        └───────────────┬─────────────────┘
                                        │
                        ┌───────────────▼─────────────────┐
                        │     Management API (Proxy)      │
                        │  (Wildcard SSL Termination)     │
                        └───────────────┬─────────────────┘
                                        │ internal routing
              ┌─────────────────────────┼─────────────────────────┐
              │                         │                         │
    ┌─────────▼─────────┐     ┌─────────▼─────────┐     ┌─────────▼─────────┐
    │  Tenant A App     │     │  Tenant B App     │     │  Tenant C App     │
    │  sp-{tenant-id}   │     │  sp-{tenant-id}   │     │  sp-{tenant-id}   │
    │  Own IP address   │     │  Own IP address   │     │  Own IP address   │
    └───────────────────┘     └───────────────────┘     └───────────────────┘
```

### Request Flow

1. **DNS Resolution**: `{tenant-id}.systemprompt.io` resolves via wildcard DNS to the Management API
2. **SSL Termination**: Management API terminates SSL using the wildcard certificate (`*.systemprompt.io`)
3. **Proxy Routing**: The proxy extracts the subdomain, looks up the tenant, and routes to `sp-{tenant-id}`
4. **Tenant Response**: The tenant app processes the request and returns the response

### Key Components

| Component | Purpose | Hostname |
|-----------|---------|----------|
| Management API | SSL termination, tenant routing | `*.systemprompt.io` |
| Tenant App | Individual tenant application | `{tenant-id}.systemprompt.io` |
| Cloud Database | Shared PostgreSQL database | Internal |

### SSL Certificates

**All wildcard certificates must be on the Management API**, not on individual tenant apps.

```bash
# Correct: Certificate on Management API (proxy)
systemprompt cloud certs add {subdomain}.systemprompt.io --app management-api

# Wrong: Certificate on tenant app (causes routing conflicts)
systemprompt cloud certs add {subdomain}.systemprompt.io --app sp-{tenant-id}
```

### Verifying Connectivity

```bash
# Check tenant status
systemprompt cloud status

# Check via custom domain
curl -sI https://{tenant-id}.systemprompt.io/

# Check certificate status
systemprompt cloud certs list
systemprompt cloud certs show {subdomain}.systemprompt.io
```

### Common Issues

| Symptom | Cause | Solution |
|---------|-------|----------|
| SSL handshake fails | Certificate on wrong app | Move cert to Management API |
| 502 Bad Gateway | Tenant app not found or down | Check `systemprompt cloud status` |
| DNS mismatch error | Cert expects different IP | Remove cert from tenant, add to proxy |
| "Awaiting configuration" | DNS not pointing to correct IP | Verify wildcard DNS points to proxy |

---

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Build fails | Rust compilation error | Check `cargo build` locally |
| Push fails | Authentication expired | Run `systemprompt cloud auth login` |
| Deploy fails | Resource limits | Check cloud dashboard for quotas |
| Sync fails | Database schema mismatch | Run migrations first |
| Site unreachable after deploy | SSL/DNS routing issue | See Cloud Infrastructure section above |

## Quick Reference

| Task | Command |
|------|---------|
| Deploy | `systemprompt cloud deploy --profile <name>` |
| Status | `systemprompt cloud status` |
| Logs | `systemprompt cloud logs -f` |
| Rollback | `systemprompt cloud rollback` |
| History | `systemprompt cloud history` |