---
title: "MCP Servers"
description: "Add and configure MCP (Model Context Protocol) servers that provide tools to AI agents. Manage binaries, ports, endpoints, and OAuth settings from the admin dashboard."
author: "systemprompt.io"
slug: "mcp-servers"
keywords: "mcp, servers, tools, model context protocol, oauth, configuration, admin, dashboard"
kind: "guide"
public: true
tags: ["mcp", "tools", "admin", "configuration"]
published_at: "2026-02-18"
updated_at: "2026-02-18"
after_reading_this:
  - "Navigate the MCP Servers list page and understand each column"
  - "Add a new MCP server with binary, port, and endpoint settings"
  - "Configure OAuth authentication for an MCP server"
  - "Enable, disable, or delete MCP server configurations"
related_docs:
  - title: "Agents"
    url: "/documentation/agents"
  - title: "Plugins"
    url: "/documentation/plugins"
  - title: "Hooks"
    url: "/documentation/hooks"
---

# MCP Servers

**TL;DR:** MCP (Model Context Protocol) servers expose tools that AI agents use during conversations. When an agent needs to search a database, call an API, or perform any external action, it invokes a tool through an MCP server. The admin dashboard lets you add, configure, and manage MCP server connections -- including binary paths, ports, endpoints, and OAuth authentication settings.


## What You'll See

Navigate to **MCP Servers** in the admin sidebar to reach the server list page. The page includes:

1. **Toolbar** -- a search box to filter servers and a **+ New MCP Server** button.
2. **Server table** -- a data table with these columns:

| Column | What it shows |
|--------|---------------|
| **Name** | The server's identifier |
| **Server ID** | The unique ID (shown as inline code) |
| **Binary** | The binary or executable that runs the server (shown as inline code) |
| **Port** | The port number the server listens on |
| **Description** | A truncated summary of the server's purpose |
| **Status** | A toggle switch to enable or disable the server |
| **Actions** | An action menu with Edit and Delete options |

If no MCP servers are configured, you see an empty state: "No MCP servers found."

### RBAC Visibility

Non-admin users only see MCP servers referenced by plugins assigned to their roles. Admins see all servers.

## Adding an MCP Server

1. Click **+ New MCP Server** (or navigate to `/admin/mcp-servers/edit/`).
2. Fill in the configuration form:

### Basic Settings

| Field | Required | Description |
|-------|----------|-------------|
| **ID** | Yes | A unique kebab-case identifier (e.g., `my-tools-server`). Cannot be changed after creation. |
| **Binary** | No | The binary name or path to execute (e.g., `systemprompt-mcp-agent`). |
| **Package** | No | The package or crate name associated with this server. |
| **Port** | No | The port number the server listens on. Defaults to 5000 if not specified. |
| **Endpoint** | No | The full endpoint URL (e.g., `/api/v1/mcp/my-server/mcp`). |
| **Description** | No | A brief description of the server's purpose. |
| **Enabled** | -- | Checkbox, enabled by default for new servers. |

### OAuth Settings

Below a divider, the form includes OAuth configuration fields:

| Field | Description |
|-------|-------------|
| **OAuth Required** | Checkbox to enforce OAuth authentication for this server. |
| **OAuth Scopes** | Comma-separated list of required scopes (e.g., `admin, tools:read`). |
| **OAuth Audience** | The audience URI for JWT validation (e.g., `mcp`). |

3. Click **Save** to create the server configuration.

## Editing an MCP Server

1. Click the action menu on any server row and select **Edit**, or navigate to `/admin/mcp-servers/edit/?id=<server-id>`.
2. Modify any field except the ID.
3. Click **Save**.

## Enabling and Disabling Servers

Toggle the switch in the **Status** column to enable or disable a server. Disabled servers are not available to agents -- their tools will not appear in tool discovery.

## Deleting an MCP Server

1. Click the action menu on the server row.
2. Select **Delete**.
3. Confirm the deletion.

Deletion requires admin access.

## Searching Servers

Use the search box to filter the server list by name/ID.

## OAuth Configuration

OAuth controls who can use an MCP server's tools. When OAuth is required:

- Requests must include a valid JWT token
- The token must contain the required scopes
- The token audience must match the configured audience

Common scope patterns:

| Scope | Use case |
|-------|----------|
| `admin` | Full administrative access to all tools |
| `user` | Standard authenticated user access |
| `tools:read` | Read-only tool access |
| `tools:write` | Tool execution permission |
| `anonymous` | No authentication required (set OAuth Required to off) |

## Connecting MCP Servers to Agents

Agents reference MCP servers in their YAML configuration to gain tool access:

```yaml
# In services/agents/your-agent.yaml
metadata:
  mcpServers:
    - my-tools-server
```

The agent then has access to all tools exposed by the listed servers.

## Connecting MCP Servers to Plugins

Plugins declare which MCP servers they require:

```yaml
# In services/plugins/your-plugin/config.yaml
plugin:
  mcp_servers:
    - my-tools-server
```

This ensures the server is included when the plugin is active.

## How MCP Servers Connect to Other Concepts

- **Agents** use MCP servers to access tools during conversations. An agent without MCP servers has no tool access.
- **Plugins** list required MCP servers in their configuration, bundling them alongside agents and skills.
- **Hooks** can fire on `PostToolUse` and `PostToolUseFailure` events, which are generated when MCP tools are invoked.
- **Skills** describe capabilities, while MCP servers provide the tools that implement them.

## Discovery

MCP servers are discoverable through the platform API:

| Endpoint | Description |
|----------|-------------|
| `/api/v1/mcp/registry` | List all available MCP servers |
| `/api/v1/mcp/{server-id}/mcp` | Individual server MCP endpoint |

