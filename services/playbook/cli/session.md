---
title: "Session Management Playbook"
description: "Manage CLI sessions, profiles, and environment switching."
keywords:
  - session
  - profiles
  - authentication
  - login
---

# Session Management Playbook

Manage CLI sessions, profiles, and environment switching.

> **Help**: `{ "command": "admin session" }` via `systemprompt_help`

---

## Understanding Sessions

Sessions are tenant-keyed -- each profile has its own session. Configuration priority (highest to lowest):

1. `--profile` CLI flag
2. `SYSTEMPROMPT_PROFILE` environment variable
3. Active session from `.systemprompt/session.json`
4. Default profile

---

## Check Current Session

```json
// MCP: systemprompt
{ "command": "admin session show" }
```

---

## List Available Profiles

```json
// MCP: systemprompt
{ "command": "admin session list" }
```

---

## Switch Profile

Switch to a different profile (automatically loads corresponding session):

```json
// MCP: systemprompt
{ "command": "admin session switch <profile-name>" }
```

---

## Login and Logout

Login and logout are terminal operations (not MCP):

```bash
just login
just login production
just logout
```

Login creates cloud credentials at `.systemprompt/credentials.json`. Sessions expire after 24 hours by default.

-> See [Contexts Playbook](contexts.md) for managing conversation contexts within a session.

---

## Troubleshooting

**No valid session** -- Re-login via terminal with `just login`.

**Wrong profile active** -- List profiles with `admin session list`, then switch with `admin session switch <profile-name>`.

**Session expired** -- Sessions expire after 24 hours. Re-authenticate with `just login`.

---

## Quick Reference

| Task | Method | Command |
|------|--------|---------|
| Check session | MCP | `admin session show` |
| List profiles | MCP | `admin session list` |
| Switch profile | MCP | `admin session switch <name>` |
| Login | Terminal | `just login` |
| Logout | Terminal | `just logout` |
