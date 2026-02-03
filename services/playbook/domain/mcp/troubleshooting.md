---
title: "MCP Troubleshooting"
description: "Diagnose and fix MCP server issues: startup failures, tool discovery, execution timeouts, auth errors."
author: "SystemPrompt"
slug: "domain-mcp-troubleshooting"
keywords: "mcp, troubleshooting, debug, tools, servers"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Troubleshooting

Diagnose and fix MCP server issues. Config: `services/mcp/*.yaml`

> **Help**: `{ "command": "core playbooks show domain_mcp-troubleshooting" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Diagnostic Checklist

{ "command": "plugins mcp status" }
{ "command": "plugins mcp list" }
{ "command": "infra logs --context mcp --limit 50" }
{ "command": "admin config show --section ai" }

---

## Issue: Server Not Starting

Symptoms: Status "stopped" or "failed", tools unavailable

Step 1: Check status

{ "command": "plugins mcp status" }

Step 2: View logs

{ "command": "plugins mcp logs my-server --limit 50" }

Step 3: Try starting

{ "command": "plugins mcp start my-server" }

Solutions:

Binary not found:

```bash
which node
```

Fix path in config: `binary: "/full/path/to/binary"`

Port in use:

```bash
lsof -i :5011
```

Change port or stop conflicting process

Config error:

{ "command": "plugins mcp show my-server" }

Check YAML syntax in `services/mcp/my-server.yaml`

---

## Issue: Tools Not Appearing

Symptoms: Empty tool list, agent can't use tools

Step 1: List tools

{ "command": "plugins mcp tools my-server" }

Step 2: Check running

{ "command": "plugins mcp status" }

Step 3: Check auto-discover

{ "command": "admin config show --section ai" }

Solutions:

Server not running:

{ "command": "plugins mcp start my-server" }

Auto-discover disabled in `services/ai/config.yaml`:

```yaml
ai:
  mcp:
    auto_discover: true
```

Force refresh:

{ "command": "plugins mcp refresh" }
{ "command": "plugins mcp restart my-server" }
{ "command": "plugins mcp tools my-server" }

---

## Issue: Tool Execution Timeout

Symptoms: "Tool execution timed out", long delays

Step 1: Check timeout config

{ "command": "admin config show --section ai" }

Step 2: Check tool logs

{ "command": "infra logs --context mcp --limit 50" }

Step 3: Check server logs

{ "command": "plugins mcp logs my-server" }

Solutions:

Increase timeout in `services/ai/config.yaml`:

```yaml
ai:
  mcp:
    execution_timeout_ms: 60000
    retry_attempts: 3
```

Check external services:

```bash
curl -I https://api.example.com/health
```

---

## Issue: Authentication Failures

Symptoms: 401/403 errors, tools work for some users

Step 1: Check OAuth config

{ "command": "plugins mcp show my-server" }

Step 2: Check user scopes

{ "command": "admin session status" }

Step 3: View auth logs

{ "command": "infra logs --context oauth --limit 50" }

Solutions:

User lacks required scope: Reduce requirements:

```yaml
oauth:
  required: true
  scopes: ["user"]
```

No auth required:

```yaml
oauth:
  required: false
```

Token expired:

{ "command": "admin session logout" }
{ "command": "admin session login" }

---

## Issue: Connection Failures

Symptoms: "Connection refused", intermittent connectivity

Step 1: Check running

{ "command": "plugins mcp status" }

Step 2: Check port

```bash
nc -zv localhost 5011
```

Step 3: Check timeout

{ "command": "admin config show --section ai" }

Solutions:

Server not running:

{ "command": "plugins mcp start my-server" }

Increase connection timeout in `services/ai/config.yaml`:

```yaml
ai:
  mcp:
    connect_timeout_ms: 10000
```

---

## Issue: Environment Variables Not Set

Symptoms: Server crashes, "API key not found"

Step 1: Check secrets

{ "command": "cloud secrets list" }

Step 2: Check config

{ "command": "plugins mcp show my-server" }

Step 3: View logs

{ "command": "plugins mcp logs my-server" }

Solutions:

Set missing secrets:

{ "command": "cloud secrets set MY_SERVER_API_KEY \"your-api-key\"" }
{ "command": "plugins mcp restart my-server" }

Verify env config:

```yaml
env:
  API_KEY: ${MY_SERVER_API_KEY}
```

---

## Issue: Server Crashes

Symptoms: Starts then stops, intermittent availability

{ "command": "plugins mcp logs my-server --limit 100" }
{ "command": "plugins mcp status" }

Solutions:

Out of memory: Reduce usage or increase limits

Uncaught exceptions: Check logs for stack traces

Dependency issues:

```bash
cd /path/to/server && npm install
```

---

## Log Messages

| Message | Meaning |
|---------|---------|
| `MCP server started on port XXXX` | Running |
| `Tool registered: tool_name` | Tool available |
| `Tool execution started` | Tool called |
| `Tool execution completed` | Tool finished |
| `Tool execution failed` | Tool error |
| `Connection refused` | Can't reach server |

{ "command": "infra logs --context mcp" }
{ "command": "plugins mcp logs my-server --level error" }
{ "command": "plugins mcp logs my-server --follow" }

---

## Quick Reference

| Problem | First Command |
|---------|---------------|
| Not starting | `plugins mcp logs <name>` |
| Tools missing | `plugins mcp status` |
| Timeout | `admin config show --section ai` |
| Auth failures | `plugins mcp show <name>` |
| Connection | `plugins mcp status` |
| Env vars | `cloud secrets list` |
| Crashes | `plugins mcp logs <name>` |
| Any issue | `infra logs --context mcp` |

---

## Related

-> See [MCP Servers](mcp-servers.md)
-> See [AI Troubleshooting](ai-troubleshooting.md)
-> See [Agent Troubleshooting](agents-troubleshooting.md)
-> See [MCP Service](/documentation/services/mcp)