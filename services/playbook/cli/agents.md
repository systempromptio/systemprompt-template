---
title: "Agents Management Playbook"
description: "Create, configure, and communicate with AI agents via A2A protocol."
author: "SystemPrompt"
slug: "cli-agents"
keywords: "agents, a2a, messaging, admin"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Agents Management Playbook

Create, configure, and communicate with AI agents via A2A protocol.

---

## List Agents

```json
{ "command": "admin agents list" }
{ "command": "admin agents list --enabled" }
```

---

## Show Agent Configuration

```json
{ "command": "admin agents show <name>" }
```

---

## Check Agent Status

```json
{ "command": "admin agents status <name>" }
```

---

## View Agent Logs

```json
{ "command": "admin agents logs <name>" }
{ "command": "admin agents logs <name> -n 50" }
```

---

## Registry (Running Agents)

```json
{ "command": "admin agents registry" }
```

---

## Send Message to Agent (A2A)

```json
{ "command": "admin agents message <name> -m \"What tools do you have?\"" }
{ "command": "admin agents message <name> -m \"Create content\" --blocking" }
{ "command": "admin agents message <name> -m \"Long task\" --blocking --timeout 120" }
{ "command": "admin agents message <name> -m \"Generate content\" --stream" }
{ "command": "admin agents message <name> -m \"Continue\" --context-id <context-id> --blocking" }
{ "command": "admin agents message <name> -m \"Continue\" --task-id <task-id> --blocking" }
```

---

## Get Task Response

```json
{ "command": "admin agents task <name> --task-id <task-id>" }
{ "command": "admin agents task <name> --task-id <task-id> --history-length 10" }
```

---

## List Agent MCP Tools

```json
{ "command": "admin agents tools <name>" }
```

---

## Validate Agent Configurations

```json
{ "command": "admin agents validate" }
{ "command": "admin agents validate <name>" }
```

---

## Create New Agent

Agent names must be lowercase alphanumeric with underscores only (no hyphens). Using `--enabled` will automatically start the agent process.

```json
{ "command": "admin agents create --name <name> --port <port>" }
{ "command": "admin agents create --name <name> --port <port> --display-name \"My Agent\" --description \"Agent description\"" }
{ "command": "admin agents create --name <name> --port <port> --provider anthropic --model claude-3-5-sonnet-20241022" }
{ "command": "admin agents create --name <name> --port <port> --mcp-server content-manager --skill research --enabled" }
```

---

## Edit Agent Configuration

```json
{ "command": "admin agents edit <name> --enable" }
{ "command": "admin agents edit <name> --disable" }
{ "command": "admin agents edit <name> --display-name \"Updated Name\"" }
{ "command": "admin agents edit <name> --provider anthropic --model claude-3-5-sonnet-20241022" }
{ "command": "admin agents edit <name> --mcp-server filesystem" }
{ "command": "admin agents edit <name> --remove-mcp-server filesystem" }
{ "command": "admin agents edit <name> --skill new_skill" }
{ "command": "admin agents edit <name> --remove-skill old_skill" }
{ "command": "admin agents edit <name> --system-prompt \"You are a helpful assistant\"" }
{ "command": "admin agents edit <name> --port 9050" }
```

---

## Delete Agent

```json
{ "command": "admin agents delete <name> -y" }
{ "command": "admin agents delete --all -y" }
```

---

## Restart Agents

```json
{ "command": "infra services restart agent <name>" }
{ "command": "infra services restart --agents" }
```

---

## Troubleshooting

**Agent not responding** -- Check status with `admin agents status <name>`, check logs with `admin agents logs <name> -n 100`, restart with `infra services restart agent <name>`.

**Agent not in registry** -- Verify agent is enabled with `admin agents show <name>`. Restart services if needed.

**Message timeout** -- Use `--blocking --timeout 120` for long tasks, or send async and check later with `admin agents task <name> --task-id <id>`.

**Validation errors** -- Run `admin agents validate <name>`. Common issues: missing API keys, invalid MCP server references.

---

## Quick Reference

| Task | Command |
|------|---------|
| List agents | `admin agents list` |
| List enabled | `admin agents list --enabled` |
| Show config | `admin agents show <name>` |
| Check status | `admin agents status <name>` |
| View logs | `admin agents logs <name>` |
| Running agents | `admin agents registry` |
| Send message | `admin agents message <name> -m "text" --blocking` |
| Send with timeout | `admin agents message <name> -m "text" --blocking --timeout 120` |
| Get task | `admin agents task <name> --task-id <id>` |
| List tools | `admin agents tools <name>` |
| Validate all | `admin agents validate` |
| Create agent | `admin agents create --name <name> --port <port>` |
| Enable agent | `admin agents edit <name> --enable` |
| Disable agent | `admin agents edit <name> --disable` |
| Delete agent | `admin agents delete <name> -y` |
| Restart agent | `infra services restart agent <name>` |