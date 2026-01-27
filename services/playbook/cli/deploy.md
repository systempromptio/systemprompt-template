---
title: "Deploy Playbook"
description: "Deploy changes to cloud tenants."
keywords:
  - deploy
  - cloud
  - production
  - release
---

# Deploy Playbook

Deploy changes to cloud tenants.

> **Help**: `{ "command": "cloud deploy" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Pre-Deploy Checklist

```json
// MCP: systemprompt
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

Or via MCP:
```json
// MCP: systemprompt
{ "command": "cloud deploy" }
{ "command": "cloud deploy --profile <profile-name>" }
{ "command": "cloud deploy --skip-push" }
```

---

## Post-Deploy Verification

```json
// MCP: systemprompt
{ "command": "cloud status" }
{ "command": "cloud db status --profile <profile-name>" }
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM users\"" }
```

---

## Restart Cloud Tenant

```json
// MCP: systemprompt
{ "command": "cloud restart --yes" }
```

---

## Troubleshooting

**Not authenticated** (`Error: Cloud authentication required`) -- run `just login` in the terminal.

**No tenant** (`Error: No tenant configured`):
```json
// MCP: systemprompt
{ "command": "cloud tenant list" }
{ "command": "cloud profile show" }
```

**Cloud not responding after deploy** -- check `cloud status`, then `cloud restart --yes`. Check platform logs if still failing.

**Wrong profile active** -- check with `admin session show`, switch with `admin session switch <profile-name>`.

-> See [Cloud Playbook](cloud.md) for tenant and profile management | [Build Playbook](build.md) | [Session Playbook](session.md)

---

## Quick Reference

| Task | Command |
|------|---------|
| Login | `just login` (terminal) |
| Full deploy | `just deploy` (terminal) |
| Deploy profile | `cloud deploy --profile <name>` |
| Skip rebuild | `cloud deploy --skip-push` |
| Cloud status | `cloud status` |
| Restart cloud | `cloud restart --yes` |
| Cloud DB query | `cloud db query --profile <name> "SQL"` |
