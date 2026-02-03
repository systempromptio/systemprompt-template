---
title: "Services Management Playbook"
description: "Manage API server, agents, and MCP servers lifecycle."
author: "SystemPrompt"
slug: "cli-services"
keywords: "services, restart, start, stop, infrastructure"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Services Management Playbook

Manage API server, agents, and MCP servers lifecycle.

---

## Understanding Services

SystemPrompt runs three types of services:

| Service Type | Flag | Description |
|--------------|------|-------------|
| API | `--api` | HTTP API server for CLI and web requests |
| Agents | `--agents` | AI agent processes (blog, linkedin, twitter, etc.) |
| MCP | `--mcp` | Model Context Protocol servers (systemprompt, content-manager) |

Use `infra services status` to see running services.

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
{ "command": "infra services status" }
{ "command": "infra services status --health" }
{ "command": "infra services status --detailed" }
```

---

## Stop Services

```json
{ "command": "infra services stop --all" }
{ "command": "infra services stop --agents" }
{ "command": "infra services stop --mcp" }
{ "command": "infra services stop --api" }
{ "command": "infra services stop --all --force" }
```

---

## Restart Services

```json
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
{ "command": "infra services cleanup --dry-run" }
{ "command": "infra services cleanup -y" }
```

---

## Troubleshooting

**Services won't start** -- run `just start` from terminal. If still failing, clean up port conflicts:
```json
{ "command": "infra services cleanup --dry-run" }
{ "command": "infra services cleanup -y" }
```
Then run `just start` again.

**Agent health check timeout:**
```json
{ "command": "admin agents status blog" }
{ "command": "admin agents logs blog" }
{ "command": "infra services restart agent blog" }
```

**MCP server not responding:**
```json
{ "command": "plugins mcp status" }
{ "command": "infra services restart mcp content-manager --build" }
```

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