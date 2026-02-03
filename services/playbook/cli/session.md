---
title: "Session Management Playbook"
description: "Manage CLI sessions, profiles, and environment switching."
author: "SystemPrompt"
slug: "cli-session"
keywords: "session, profiles, authentication, login"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-01"
updated_at: "2026-02-02"
---

# Session Management Playbook

Manage CLI sessions, profiles, and environment switching.

---

## Understanding Sessions

Sessions are tenant-keyed -- each profile has its own session. Configuration priority (highest to lowest):

1. `--profile` CLI flag
2. `SYSTEMPROMPT_PROFILE` environment variable
3. Active session from `.systemprompt/sessions/index.json`
4. Default profile

### Session Storage

Sessions are stored in `.systemprompt/sessions/index.json`:

```json
{
  "version": 1,
  "sessions": {
    "local": {
      "tenant_key": "local",
      "profile_name": "local",
      "profile_path": "/path/to/.systemprompt/profiles/local/profile.yaml",
      "session_token": "eyJ...",
      "user_email": "admin@local.dev",
      "user_type": "admin",
      "expires_at": "2026-02-02T10:00:00Z"
    }
  },
  "active_key": "local"
}
```

### Session Expiration

Sessions expire **24 hours** after creation. When expired, re-authenticate with `just login`.

---

## Check Current Session

```json
{ "command": "admin session show" }
```

---

## List Available Profiles

```json
{ "command": "admin session list" }
```

---

## Switch Profile

Switch to a different profile (automatically loads corresponding session):

```json
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

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| No valid session | Re-login via terminal: `just login` |
| Not authenticated | Re-login via terminal: `just login` |
| Wrong profile active | List with `admin session list`, switch with `admin session switch <name>` |
| Session expired | Sessions expire after 24 hours. Re-authenticate: `just login` |
| Permission denied | Check profile has required permissions: `admin session show` |
| Connection failed | Check credentials exist in `.systemprompt/credentials.json` |

**Note**: All other playbooks should cross-reference this section for authentication issues.

---

## Quick Reference

| Task | Method | Command |
|------|--------|---------|
| Check session | MCP | `admin session show` |
| List profiles | MCP | `admin session list` |
| Switch profile | MCP | `admin session switch <name>` |
| Login | Terminal | `just login` |
| Logout | Terminal | `just logout` |