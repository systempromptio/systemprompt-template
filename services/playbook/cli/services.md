---
title: "Services Management Playbook"
description: "Manage API server, agents, and MCP servers lifecycle."
keywords:
  - services
  - restart
  - start
  - stop
  - infrastructure
---

# Services Management Playbook

Manage API server, agents, and MCP servers lifecycle.

> **Help**: `{ "command": "infra services" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Start Services

Starting services requires the terminal:

```bash
just start
```

This runs `systemprompt infra services start --skip-web`.

---

## Check Service Status

```json
// MCP: systemprompt
{ "command": "infra services status" }
{ "command": "infra services status --health" }
{ "command": "infra services status --detailed" }
```

---

## Stop Services

```json
// MCP: systemprompt
{ "command": "infra services stop --all" }
{ "command": "infra services stop --agents" }
{ "command": "infra services stop --mcp" }
{ "command": "infra services stop --api" }
{ "command": "infra services stop --all --force" }
```

---

## Restart Services

```json
// MCP: systemprompt
{ "command": "infra services restart api" }
{ "command": "infra services restart agent blog" }
{ "command": "infra services restart agent linkedin" }
{ "command": "infra services restart agent twitter" }
{ "command": "infra services restart mcp content-manager" }
{ "command": "infra services restart mcp content-manager --build" }
{ "command": "infra services restart --agents" }
{ "command": "infra services restart --failed" }
```

---

## Cleanup Orphaned Processes

```json
// MCP: systemprompt
{ "command": "infra services cleanup --dry-run" }
{ "command": "infra services cleanup -y" }
```

---

## Troubleshooting

**Services won't start** -- run `just start` from terminal. If still failing, clean up port conflicts:
```json
// MCP: systemprompt
{ "command": "infra services cleanup --dry-run" }
{ "command": "infra services cleanup -y" }
```
Then run `just start` again.

**Agent health check timeout:**
```json
// MCP: systemprompt
{ "command": "admin agents status blog" }
{ "command": "admin agents logs blog" }
{ "command": "infra services restart agent blog" }
```

**MCP server not responding:**
```json
// MCP: systemprompt
{ "command": "plugins mcp status" }
{ "command": "infra services restart mcp content-manager --build" }
```

-> See [Agents Playbook](agents.md) | [Plugins Playbook](plugins.md) | [Build Playbook](build.md)

---

## Quick Reference

| Task | Command |
|------|---------|
| Start services | `just start` (terminal) |
| Check status | `infra services status` |
| Health check | `infra services status --health` |
| Stop all | `infra services stop --all` |
| Restart API | `infra services restart api` |
| Restart agent | `infra services restart agent <name>` |
| Restart MCP | `infra services restart mcp <name>` |
| Restart MCP + rebuild | `infra services restart mcp <name> --build` |
| Restart failed | `infra services restart --failed` |
| Cleanup | `infra services cleanup -y` |
