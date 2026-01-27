---
title: "Configuration Management Playbook"
description: "View and manage system configuration."
keywords:
  - config
  - settings
  - rate-limits
  - admin
---

# Configuration Management Playbook

View and manage system configuration.

> **Help**: `{ "command": "admin config" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Configuration Overview

```json
// MCP: systemprompt
{ "command": "admin config show" }
```

---

## Rate Limits

```json
// MCP: systemprompt
{ "command": "admin config rate-limits show" }
{ "command": "admin config rate-limits list" }
```

---

## Server Configuration

```json
// MCP: systemprompt
{ "command": "admin config server show" }
```

Shows host, port, API endpoints, and timeout settings.

---

## Runtime Configuration

```json
// MCP: systemprompt
{ "command": "admin config runtime show" }
```

Shows worker threads, memory limits, and feature flags.

---

## Security Configuration

```json
// MCP: systemprompt
{ "command": "admin config security show" }
```

Shows authentication settings, CORS configuration, and session settings.

---

## Paths Configuration

```json
// MCP: systemprompt
{ "command": "admin config paths show" }
```

Shows storage paths, log paths, and config file locations.

---

## Troubleshooting

**Check current settings** -- Run `admin config show` for a full overview.

**Verify paths** -- Run `admin config paths show` to confirm file locations.

---

## Quick Reference

| Task | Command |
|------|---------|
| Config overview | `admin config show` |
| Rate limits | `admin config rate-limits show` |
| Server config | `admin config server show` |
| Runtime config | `admin config runtime show` |
| Security config | `admin config security show` |
| Paths config | `admin config paths show` |
