---
title: "MCP Services"
description: "Configure MCP servers that provide tools to AI models. Set up OAuth authentication, tool definitions, and integration with Claude Desktop and other clients."
author: "SystemPrompt Team"
slug: "services/mcp"
keywords: "mcp, servers, tools, oauth, hosting, model context protocol"
image: "/files/images/docs/services-mcp.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# MCP Services

**TL;DR:** MCP (Model Context Protocol) servers provide tools that AI models can use during conversations. The MCP service configures which servers are available, their endpoints, OAuth authentication, and how they integrate with the AI service. When an agent needs to perform an action, it calls tools through MCP.

## The Problem

AI models are powerful at reasoning but limited in what they can do. They cannot access databases, call APIs, or interact with systems directly. Every action requires external tools.

MCP solves this by providing a standardized protocol for tools. MCP servers expose tools that AI models can discover and invoke. The AI service handles the integration, so agents automatically have access to all configured MCP tools.

## How MCP Works

MCP follows a client-server model:

1. **MCP Server** - A service that exposes tools through the MCP protocol
2. **AI Service** - Acts as an MCP client, discovering and calling tools
3. **Agent** - Uses tools through the AI service during conversations

When an agent processes a message, the AI model may decide to use a tool. The AI service finds the appropriate MCP server, invokes the tool, and returns the result to the model. This happens transparently to the user.

## Configuration

Configure MCP servers in `services/mcp/`:

<details>
<summary>MCP server configuration</summary>

```yaml
# services/mcp/systemprompt.yaml
mcp_servers:
  systemprompt:
    binary: "systemprompt-mcp-agent"
    package: "systemprompt"
    port: 5010
    endpoint: "http://localhost:8080/api/v1/mcp/systemprompt/mcp"
    enabled: true
    display_in_web: true
    description: "SystemPrompt MCP Server - Execute CLI commands"

    oauth:
      required: true
      scopes: ["admin"]
      audience: "mcp"
```

</details>

Each MCP server has its own YAML file defining its endpoint, authentication, and metadata.

## Server Configuration

| Field | Type | Description |
|-------|------|-------------|
| `binary` | string | Binary name to execute |
| `package` | string | Package/crate name |
| `port` | number | Server port within the configured range |
| `endpoint` | string | Full API endpoint URL |
| `enabled` | boolean | Whether server is active |
| `display_in_web` | boolean | Show in web UI |
| `description` | string | Human-readable description |

## OAuth Authentication

Secure MCP servers with OAuth to control who can use which tools:

```yaml
oauth:
  required: true          # Require authentication
  scopes: ["admin"]       # Required scopes
  audience: "mcp"         # JWT audience
  client_id: null         # Optional client restriction
```

Common scope patterns:
- `admin` - Administrative access, can execute any tool
- `user` - Standard user access
- `tools:read` - Read-only tool access
- `tools:write` - Tool execution permission

## Discovery

MCP servers are discoverable through the API:

| Endpoint | Description |
|----------|-------------|
| `/api/v1/mcp/registry` | List all MCP servers |
| `/api/v1/mcp/{server}/mcp` | Individual server endpoint |

The AI service uses discovery to find available tools and register them with the language model.

## Claude Desktop Integration

Add your MCP server to Claude Desktop by editing the configuration:

<details>
<summary>Claude Desktop configuration</summary>

```json
{
  "mcpServers": {
    "systemprompt": {
      "url": "http://localhost:8080/api/v1/mcp/systemprompt/mcp",
      "transport": "streamable-http"
    }
  }
}
```

</details>

This allows Claude Desktop to use your MCP tools directly. The URL must be accessible from the Claude Desktop client.

## Managing MCP Servers

Use the CLI to manage MCP servers:

```bash
# List MCP servers
systemprompt plugins mcp list

# Show server details
systemprompt plugins mcp show systemprompt

# List tools for a server
systemprompt plugins mcp tools systemprompt

# Test server connectivity
systemprompt plugins mcp test systemprompt
```

## Service Relationships

MCP servers connect to:

- **AI service** - Auto-discovers servers and exposes tools to models
- **Agents** - Use tools through the AI service during conversations
- **Config service** - Included through the aggregation pattern

The port range for MCP servers is configured in the config service (default: 5000-5999).

## Building MCP Servers

MCP servers are Rust binaries that implement the MCP protocol. The typical structure:

```
extensions/mcp/systemprompt/
├── Cargo.toml
├── manifest.yaml
└── src/
    ├── main.rs
    ├── server.rs
    └── tools.rs
```

See the MCP Extensions documentation for building custom servers.

## Syncing Configuration

After modifying MCP server configurations:

```bash
systemprompt cloud sync local mcp --direction to-db -y
```

## CLI Reference

| Command | Description |
|---------|-------------|
| `systemprompt plugins mcp list` | List MCP server configs |
| `systemprompt plugins mcp status` | Show running MCP server status with binary info |
| `systemprompt plugins mcp validate` | Validate MCP connection |
| `systemprompt plugins mcp logs` | View MCP server logs |
| `systemprompt plugins mcp list-packages` | List package names for build |
| `systemprompt plugins mcp tools` | List tools from running MCP servers |
| `systemprompt plugins mcp call <tool>` | Execute a tool on an MCP server |

See `systemprompt plugins mcp <command> --help` for detailed options.

## Troubleshooting

**Server not discovered** -- Check that `enabled: true` is set and the endpoint is accessible. Verify the AI service has `auto_discover: true`.

**Tool execution fails** -- Check server logs for errors. Verify OAuth tokens have the required scopes. Increase timeout settings if needed.

**Authentication errors** -- Ensure the client has the required scopes. Check that the audience matches the server configuration.