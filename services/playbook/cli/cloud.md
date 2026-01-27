---
title: "Cloud Management Playbook"
description: "Authentication, tenants, profiles, and secrets for cloud operations."
keywords:
  - cloud
  - authentication
  - tenants
  - profiles
  - secrets
---

# Cloud Management Playbook

Authentication, tenants, profiles, and secrets for cloud operations.

> **Help**: `{ "command": "cloud" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Authentication

```json
// MCP: systemprompt
{ "command": "cloud auth whoami" }
```

Login and logout require the terminal:

```bash
just login
just logout
```

---

## Tenant Management

```json
// MCP: systemprompt
{ "command": "cloud tenant list" }
{ "command": "cloud tenant show" }
{ "command": "cloud tenant show <tenant-id>" }
{ "command": "cloud tenant create --region iad" }
{ "command": "cloud tenant create --name \"My Project\" --region lhr" }
{ "command": "cloud tenant select <tenant-id>" }
{ "command": "cloud tenant rotate-credentials <tenant-id> -y" }
```

---

## Profile Management

```json
// MCP: systemprompt
{ "command": "cloud profile list" }
{ "command": "cloud profile show" }
{ "command": "cloud profile show <profile-name>" }
{ "command": "cloud profile create production" }
{ "command": "cloud profile create staging --environment staging" }
{ "command": "cloud profile edit <profile-name>" }
{ "command": "cloud profile delete staging -y" }
```

---

## Secrets Management

```json
// MCP: systemprompt
{ "command": "cloud secrets list" }
{ "command": "cloud secrets list --profile <profile-name>" }
{ "command": "cloud secrets set ANTHROPIC_API_KEY sk-ant-xxxxx" }
{ "command": "cloud secrets set DATABASE_URL postgres://..." }
{ "command": "cloud secrets delete OLD_KEY -y" }
```

---

## Cloud Database

```json
// MCP: systemprompt
{ "command": "cloud db status" }
{ "command": "cloud db status --profile <profile-name>" }
{ "command": "cloud db query --profile <profile-name> \"SELECT COUNT(*) FROM users\"" }
{ "command": "cloud db query --profile <profile-name> \"SELECT * FROM content LIMIT 5\"" }
{ "command": "cloud db tables --profile <profile-name>" }
```

---

## Cloud Status & Operations

```json
// MCP: systemprompt
{ "command": "cloud status" }
{ "command": "cloud restart --yes" }
{ "command": "cloud init" }
{ "command": "cloud dockerfile" }
{ "command": "cloud dockerfile --output Dockerfile" }
```

---

## Troubleshooting

**Not authenticated** -- run `just login` in the terminal.

**Wrong profile active:**
```json
// MCP: systemprompt
{ "command": "admin session show" }
{ "command": "admin session switch <profile-name>" }
```

**Cloud DB connection failed** -- verify profile config with `cloud profile show <profile-name>` and check `cloud status`.

-> See [Deploy Playbook](deploy.md) | [Sync Playbook](sync.md) | [Session Playbook](session.md)

---

## Quick Reference

| Task | Command |
|------|---------|
| Login | `just login` (terminal) |
| Check auth | `cloud auth whoami` |
| Logout | `just logout` (terminal) |
| List tenants | `cloud tenant list` |
| Show tenant | `cloud tenant show` |
| Create tenant | `cloud tenant create --region iad` |
| Select tenant | `cloud tenant select <id>` |
| List profiles | `cloud profile list` |
| Show profile | `cloud profile show <name>` |
| Create profile | `cloud profile create <name>` |
| Edit profile | `cloud profile edit <name>` |
| Delete profile | `cloud profile delete <name> -y` |
| List secrets | `cloud secrets list` |
| Set secret | `cloud secrets set KEY value` |
| Delete secret | `cloud secrets delete KEY -y` |
| DB status | `cloud db status --profile <name>` |
| DB query | `cloud db query --profile <name> "SQL"` |
| DB tables | `cloud db tables --profile <name>` |
| Cloud status | `cloud status` |
| Restart cloud | `cloud restart --yes` |
