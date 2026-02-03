---
title: "Sessions"
description: "CLI authentication state and profile switching. Sessions track which profile is active and authenticate your requests."
author: "SystemPrompt Team"
slug: "config/sessions"
keywords: "sessions, authentication, profiles, CLI, login, switch"
image: "/files/images/docs/config-sessions.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# Sessions

Sessions track your active CLI state. When you run SystemPrompt commands, the session determines which profile is active and authenticates your requests to the database and cloud services.

## What Sessions Do

Every CLI command needs to know two things: which environment to target and who is making the request. Sessions answer both questions.

Sessions are tenant-keyed, meaning each tenant has its own session. When you switch profiles, you're also switching which session is active. This keeps your work in different environments completely separate.

A session contains:
- The active profile path
- Your authenticated user identity
- A JWT token for API requests
- Expiration timestamp (sessions expire after 24 hours)

## Session Storage

Sessions are stored in `.systemprompt/sessions/index.json`. This file tracks all sessions across all tenants and identifies which one is currently active.

```json
{
  "version": 1,
  "sessions": {
    "local": {
      "version": 4,
      "tenant_key": "local",
      "profile_name": "local",
      "profile_path": "/path/to/.systemprompt/profiles/local/profile.yaml",
      "session_token": "eyJ...",
      "session_id": "sess_abc123",
      "user_id": "usr_def456",
      "user_email": "admin@local.dev",
      "user_type": "admin",
      "created_at": "2026-01-30T10:00:00Z",
      "expires_at": "2026-01-31T10:00:00Z",
      "last_used": "2026-01-30T15:30:00Z"
    },
    "production": {
      "tenant_key": "production",
      "profile_name": "production",
      "profile_path": "/path/to/.systemprompt/profiles/production/profile.yaml",
      "session_token": "eyJ...",
      "user_email": "user@company.com",
      "user_type": "admin"
    }
  },
  "active_key": "local",
  "active_profile_name": "local"
}
```

## Profile Priority

When you run a command, SystemPrompt determines which profile to use in this order:

1. **`--profile` flag** - Explicit override on the command
2. **`SYSTEMPROMPT_PROFILE` environment variable** - Set in your shell
3. **Active session** - From `sessions/index.json`
4. **Default profile** - Fallback if nothing else is set

This hierarchy lets you temporarily use a different profile without switching sessions.

```bash
# Uses active session (local)
systemprompt admin agents list

# Overrides with production profile for this command only
systemprompt admin agents list --profile production

# Sets production as default for this shell session
export SYSTEMPROMPT_PROFILE=~/.systemprompt/profiles/production/profile.yaml
systemprompt admin agents list
```

## Check Current Session

View your active session to verify you're authenticated and using the correct profile.

```bash
systemprompt admin session show
```

Output shows your user identity, active profile, and session expiration:

```
Session Status
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Profile:     local
User:        admin@local.dev
Type:        admin
Expires:     2026-01-31 10:00:00 UTC
Last Used:   5 minutes ago
```

## List Available Profiles

See all profiles you can switch between.

```bash
systemprompt admin session list
```

Output:

```
Available Profiles
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  local        (active) - Local Development
  staging               - Staging Environment
  production            - Production
```

## Switch Profiles

Change your active session to a different profile.

```bash
systemprompt admin session switch staging
```

This loads the staging profile's session. If no session exists for that profile, you'll be prompted to authenticate.

## Login and Logout

Authentication happens at the terminal level, not through MCP commands.

```bash
# Authenticate with SystemPrompt
just login

# Authenticate for a specific profile
just login production

# Clear authentication
just logout
```

Login opens your browser for GitHub or Google OAuth. On success, it creates or updates `credentials.json` and establishes a session for the active profile.

## Session Lifecycle

**Creation**: Sessions are created when you first run a command after setting up a profile. The CLI authenticates against the profile's database and creates a session token.

**Usage**: Each command refreshes the `last_used` timestamp. The session token is included in API requests for authentication.

**Expiration**: Sessions expire after 24 hours by default. Expired sessions require re-authentication with `just login`.

**Switching**: Switching profiles changes which session is active. Both sessions remain valid until they expire.

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| "Not authenticated" | No valid session | Run `just login` |
| "Session expired" | Token older than 24 hours | Run `just login` |
| "Wrong profile active" | Unexpected active session | Run `admin session list` then `admin session switch <name>` |
| "Permission denied" | User lacks required permissions | Check `admin session show` for user type |
| "Connection failed" | Cannot reach database | Verify profile's `DATABASE_URL` is correct |

## Sessions and Tenants

Sessions are tenant-keyed. Each tenant (local or cloud) maintains its own session state. This means:

- Switching from a local profile to a cloud profile switches sessions entirely
- Session tokens are specific to their tenant's database
- You can have active sessions in multiple tenants simultaneously
- Each tenant tracks its own session expiration

See [Tenants](/documentation/config/tenants) for more on how tenants work.