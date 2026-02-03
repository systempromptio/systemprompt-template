---
title: "Deploy Playbook"
description: "Deploy changes to cloud tenants."
author: "SystemPrompt"
slug: "cli-deploy"
keywords: "deploy, cloud, production, release"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Deploy Playbook

Deploy changes to cloud tenants.

---

## Pre-Deploy Checklist

```json
{ "command": "cloud auth whoami" }
{ "command": "admin session show" }
{ "command": "cloud tenant list" }
{ "command": "cloud status" }
```

If not authenticated, run `just login` in the terminal.

Switch profile if needed:
```json
{ "command": "admin session switch <profile-name>" }
```

---

## Deploy Commands

Full deploy (build + push + deploy) is a long-running operation -- use the terminal:

```bash
just deploy
```

### Non-Interactive Mode (CI/CD or Agents)

In non-interactive mode (scripts, CI/CD, AI agents), you must provide explicit flags:

```bash
# Required flags for non-interactive deploy
systemprompt cloud deploy --profile <profile-name> --yes

# Skip pre-deploy sync (use when only deploying code changes, not runtime files)
systemprompt cloud deploy --profile <profile-name> --yes --no-sync
```

| Flag | Required | Description |
|------|----------|-------------|
| `--profile <name>` | Yes | Target profile (e.g., `systemprompt-prod`) |
| `--yes` | Yes | Confirm destructive operation |
| `--no-sync` | No | Skip syncing runtime files from cloud before deploy |
| `--skip-push` | No | Skip Docker build/push (redeploy existing image) |

### Via MCP

```json
{ "command": "cloud deploy" }
{ "command": "cloud deploy --profile <profile-name> --yes" }
{ "command": "cloud deploy --profile <profile-name> --yes --no-sync" }
```

---

## Post-Deploy Verification

```json
{ "command": "cloud status" }
{ "command": "cloud db status --profile <profile-name>" }
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM users\"" }
```

---

## Restart Cloud Tenant

```json
{ "command": "cloud restart --yes" }
```

---

## Troubleshooting

**Not authenticated** (`Error: Cloud authentication required`) -- run `just login` in the terminal.

**No tenant** (`Error: No tenant configured`):

```json
{ "command": "cloud tenant list" }
{ "command": "cloud profile show" }
```

**Cloud not responding after deploy** -- check `cloud status`, then `cloud restart --yes`.

**Site unreachable after deploy (SSL error)**:

This usually indicates a certificate routing conflict. See the [Cloud Infrastructure Playbook](/playbooks/build-cloud) for details.

Quick diagnosis:
```bash
# Test direct tenant access (bypasses proxy)
curl -sI https://sp-{tenant-id}.fly.dev/

# Test via custom domain
curl -sI https://{tenant-id}.systemprompt.io/

# Check certificate status
systemprompt cloud certs list
```

If direct access works but custom domain fails, there's likely a certificate on the tenant app that conflicts with the Management API's wildcard certificate. See the Cloud Infrastructure Playbook for resolution steps.

---

## Quick Reference

| Task | Command |
|------|---------|
| Login | `just login` (terminal) |
| Full deploy (interactive) | `just deploy` (terminal) |
| Full deploy (non-interactive) | `cloud deploy --profile <name> --yes` |
| Deploy without sync | `cloud deploy --profile <name> --yes --no-sync` |
| Skip rebuild | `cloud deploy --profile <name> --yes --skip-push` |
| Cloud status | `cloud status` |
| Restart cloud | `cloud restart --yes` |
| Cloud DB query | `cloud db query --profile <name> "SQL"` |