---
title: "Plugins & MCP Server Playbook"
description: "Manage extensions and MCP servers."
keywords:
  - plugins
  - mcp
  - extensions
  - tools
---

# Plugins & MCP Server Playbook

Manage extensions and MCP servers.

> **Help**: `{ "command": "plugins" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session Playbook](session.md)

---

## Extensions

```json
// MCP: systemprompt
{ "command": "plugins list" }
{ "command": "plugins show blog" }
{ "command": "plugins show content-manager" }
{ "command": "plugins validate" }
```

Extension config (`<extension-id>` is required):
```json
{ "command": "plugins config blog" }
{ "command": "plugins config content-manager" }
```

Capabilities (subcommand required -- schemas, jobs, templates, tools, roles, llm-providers):
```json
{ "command": "plugins capabilities schemas" }
{ "command": "plugins capabilities jobs" }
{ "command": "plugins capabilities templates" }
```

---

## MCP Servers

```json
// MCP: systemprompt
{ "command": "plugins mcp list" }
{ "command": "plugins mcp status" }
{ "command": "plugins mcp validate systemprompt" }
{ "command": "plugins mcp validate content-manager" }
{ "command": "plugins mcp logs systemprompt" }
{ "command": "plugins mcp logs content-manager" }
{ "command": "plugins mcp list-packages" }
```

---

## MCP Tools

```json
// MCP: systemprompt
{ "command": "plugins mcp tools" }
{ "command": "plugins mcp tools --server systemprompt" }
{ "command": "plugins mcp tools --server content-manager" }
{ "command": "plugins mcp call systemprompt systemprompt --args '{\"command\":\"admin session show\"}'" }
```

---

## Troubleshooting

**MCP server not responding:**
```json
// MCP: systemprompt
{ "command": "plugins mcp status" }
{ "command": "plugins mcp logs <server-name>" }
{ "command": "infra services restart mcp <server-name>" }
```

**Tool not found** -- list available tools with `plugins mcp tools` or `plugins mcp tools --server <server-name>`.

**Extension validation failed** -- run `plugins validate`.

-> See [Services Playbook](services.md) for restarting MCP servers

---

## Quick Reference

| Task | Command |
|------|---------|
| List extensions | `plugins list` |
| Show extension | `plugins show <name>` |
| Validate extensions | `plugins validate` |
| Extension config | `plugins config <id>` |
| List schemas | `plugins capabilities schemas` |
| List jobs | `plugins capabilities jobs` |
| List MCP servers | `plugins mcp list` |
| MCP status | `plugins mcp status` |
| Validate MCP | `plugins mcp validate <server>` |
| MCP logs | `plugins mcp logs <server>` |
| List packages | `plugins mcp list-packages` |
| List tools | `plugins mcp tools` |
| List server tools | `plugins mcp tools --server <name>` |
| Call tool | `plugins mcp call <server> <tool> --args '{...}'` |
