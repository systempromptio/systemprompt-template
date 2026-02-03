---
title: "Plugins & MCP Server Playbook"
description: "Manage extensions and MCP servers."
author: "SystemPrompt"
slug: "cli-plugins"
keywords: "plugins, extensions, mcp, tools"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2025-01-29"
updated_at: "2026-02-02"
---

# Plugins & MCP Server Playbook

Manage extensions and MCP servers.

---

## Extensions

### List Extensions

```json
{ "command": "plugins list" }
```

### Show Extension

```json
{ "command": "plugins show <name>" }
{ "command": "plugins show blog" }
{ "command": "plugins show content-manager" }
```

### Extension Config

```json
{ "command": "plugins config <extension-id>" }
{ "command": "plugins config blog" }
```

### Validate Extensions

```json
{ "command": "plugins validate" }
```

---

## MCP Servers

### List Servers

```json
{ "command": "plugins mcp list" }
{ "command": "plugins mcp list-packages" }
```

### Server Status

```json
{ "command": "plugins mcp status" }
```

### Validate Server

```json
{ "command": "plugins mcp validate <server-name>" }
{ "command": "plugins mcp validate systemprompt" }
```

### Server Logs

```json
{ "command": "plugins mcp logs <server-name>" }
{ "command": "plugins mcp logs systemprompt" }
```

---

## MCP Tools

### List Tools

```json
{ "command": "plugins mcp tools" }
{ "command": "plugins mcp tools --server <server-name>" }
```

### Call Tool

```json
{ "command": "plugins mcp call <server> <tool> --args '{...}'" }
{ "command": "plugins mcp call systemprompt systemprompt --args '{\"command\":\"admin session show\"}'" }
```

---

## Capabilities

View extension-provided schemas, jobs, templates, and tools.

```json
{ "command": "plugins capabilities schemas" }
{ "command": "plugins capabilities jobs" }
{ "command": "plugins capabilities templates" }
{ "command": "plugins capabilities tools" }
{ "command": "plugins capabilities roles" }
{ "command": "plugins capabilities llm-providers" }
```

---

## Troubleshooting

**Extension not found** -- Check name with `plugins list`.

**MCP server not responding** -- Check status and logs:

```json
{ "command": "plugins mcp status" }
{ "command": "plugins mcp logs <server-name>" }
```

Then restart via `infra services restart mcp <server-name>`.

**Tool not found** -- List available tools with `plugins mcp tools --server <server-name>`.

---

## Quick Reference

| Task | Command |
|------|---------|
| List extensions | `plugins list` |
| Show extension | `plugins show <name>` |
| Extension config | `plugins config <id>` |
| Validate extensions | `plugins validate` |
| List MCP servers | `plugins mcp list` |
| MCP status | `plugins mcp status` |
| Validate MCP | `plugins mcp validate <server>` |
| MCP logs | `plugins mcp logs <server>` |
| List packages | `plugins mcp list-packages` |
| List tools | `plugins mcp tools` |
| List server tools | `plugins mcp tools --server <name>` |
| Call tool | `plugins mcp call <server> <tool> --args '{...}'` |
| List schemas | `plugins capabilities schemas` |
| List jobs | `plugins capabilities jobs` |