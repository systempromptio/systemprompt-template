---
title: "MCP Server Configuration"
description: "Configure and manage MCP servers with tools, OAuth, and transport protocols."
keywords:
  - mcp
  - servers
  - tools
  - configuration
  - oauth
category: domain
---

# MCP Server Configuration

MCP server setup. Config: `services/mcp/*.yaml`

> **Help**: `{ "command": "core playbooks show domain_mcp-servers" }` via `systemprompt_help`
> **Requires**: Active session -> See [Session](../cli/session.md)

---

## Configure Server

Step 1: Create `services/mcp/<name>.yaml`:

```yaml
mcp_servers:
  my-server:
    binary: "node"
    args: ["dist/index.js"]
    package: "my-mcp-server"
    port: 5011
    endpoint: "http://localhost:8080/api/v1/mcp/my-server/mcp"
    enabled: true
    display_in_web: true
    description: "My custom MCP server"
    env:
      API_KEY: ${MY_SERVER_API_KEY}
      DEBUG: "false"
    oauth:
      required: true
      scopes: ["user"]
      audience: "mcp"
      client_id: null
    working_dir: "/path/to/server"
```

Step 2: Set secrets

{ "command": "cloud secrets set MY_SERVER_API_KEY \"your-api-key\"" }
{ "command": "cloud secrets list" }

Step 3: Sync and start

{ "command": "cloud sync local mcp --direction to-db -y" }
{ "command": "plugins mcp start my-server" }
{ "command": "plugins mcp status" }

Step 4: Verify tools

{ "command": "plugins mcp tools my-server" }

---

## Configure OAuth

Public (no auth):

```yaml
oauth:
  required: false
```

Authenticated:

```yaml
oauth:
  required: true
  scopes: ["user"]
  audience: "mcp"
  client_id: null
```

Admin only:

```yaml
oauth:
  required: true
  scopes: ["admin"]
  audience: "mcp"
  client_id: null
```

{ "command": "plugins mcp show my-server" }

---

## Lifecycle Management

Start:

{ "command": "plugins mcp start my-server" }
{ "command": "plugins mcp start --all" }

Stop:

{ "command": "plugins mcp stop my-server" }
{ "command": "plugins mcp stop --all" }

Restart:

{ "command": "plugins mcp restart my-server" }
{ "command": "plugins mcp restart --all" }

Status:

{ "command": "plugins mcp status" }
{ "command": "plugins mcp status my-server" }

Logs:

{ "command": "plugins mcp logs my-server" }
{ "command": "plugins mcp logs my-server --follow" }
{ "command": "plugins mcp logs my-server --level error" }

---

## SystemPrompt MCP Server

Built-in admin tools:

```yaml
mcp_servers:
  systemprompt:
    binary: "systemprompt-mcp-agent"
    package: "systemprompt"
    port: 5010
    endpoint: "http://localhost:8080/api/v1/mcp/systemprompt/mcp"
    enabled: true
    display_in_web: true
    description: "SystemPrompt MCP Server - admin only"
    oauth:
      required: true
      scopes: ["admin"]
      audience: "mcp"
      client_id: null
```

{ "command": "plugins mcp tools systemprompt" }

---

## Tool Discovery

{ "command": "plugins mcp refresh" }
{ "command": "plugins mcp tools my-server" }

---

## Environment Variables

```yaml
mcp_servers:
  my-server:
    env:
      API_KEY: ${MY_SERVER_API_KEY}
      SECRET: ${MY_SERVER_SECRET}
      DEBUG: "true"
      NODE_ENV: "production"
```

{ "command": "cloud secrets set MY_SERVER_API_KEY \"key-value\"" }
{ "command": "plugins mcp logs my-server" }

---

## AI Integration

In `services/ai/config.yaml`:

```yaml
ai:
  mcp:
    auto_discover: true
    connect_timeout_ms: 5000
    execution_timeout_ms: 30000
    retry_attempts: 3
```

{ "command": "admin agents tools welcome" }

---

## Configuration Reference

| Field | Description |
|-------|-------------|
| `binary` | Executable to run |
| `args` | Command-line arguments |
| `working_dir` | Working directory |
| `package` | Package identifier |
| `port` | Port to listen on |
| `endpoint` | Full endpoint URL |
| `enabled` | Server is active |
| `display_in_web` | Show in UI |
| `description` | Human-readable description |
| `env` | Environment variables |
| `oauth.required` | Require auth |
| `oauth.scopes` | Required OAuth scopes |

---

## Troubleshooting

- Server not starting: `{ "command": "plugins mcp logs my-server" }`, check binary exists
- Tools not appearing: `{ "command": "plugins mcp refresh" }`, `{ "command": "plugins mcp restart my-server" }`
- OAuth failures: Check scopes, use `oauth.required: false` for public

---

## Quick Reference

| Task | Command |
|------|---------|
| List | `plugins mcp list` |
| Show | `plugins mcp show <name>` |
| Start | `plugins mcp start <name>` |
| Stop | `plugins mcp stop <name>` |
| Restart | `plugins mcp restart <name>` |
| Status | `plugins mcp status` |
| Logs | `plugins mcp logs <name>` |
| Tools | `plugins mcp tools <name>` |
| Refresh | `plugins mcp refresh` |

---

## Related

-> See [MCP Troubleshooting](mcp-troubleshooting.md)
-> See [AI Providers](ai-providers.md)
-> See [MCP Service](/documentation/services/mcp)
