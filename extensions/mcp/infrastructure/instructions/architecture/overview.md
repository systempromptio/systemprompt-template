# MCP Extension Architecture

This document defines the architecture for building SystemPrompt-compliant MCP (Model Context Protocol) server extensions.

---

## Overview

MCP servers are **standalone extensions** to systemprompt-core. Each extension:

- Is a separate Rust crate with its own `Cargo.toml`
- Uses the `systemprompt` facade for core functionality
- Implements the `rmcp::ServerHandler` trait for MCP protocol compliance
- Runs as an independent HTTP service

---

## Component Architecture

```
MCP Extension (standalone crate)
    │
    ├── src/
    │   ├── main.rs              → Bootstrap & serve
    │   ├── lib.rs               → Public exports
    │   │
    │   ├── server/
    │   │   ├── mod.rs           → ServerHandler trait impl
    │   │   ├── constructor.rs   → Server struct definition
    │   │   └── handlers/
    │   │       ├── mod.rs       → Handler exports
    │   │       ├── initialization.rs
    │   │       └── tools.rs     → Tool call handling
    │   │
    │   ├── tools/               → Tool implementations
    │   │   ├── mod.rs           → register_tools(), handle_tool_call()
    │   │   └── {tool_name}.rs   → Individual tool handlers
    │   │
    │   ├── prompts/             → Prompt implementations
    │   │   ├── mod.rs           → {Extension}Prompts struct
    │   │   └── {prompt_name}.rs → Prompt content builders
    │   │
    │   └── resources/           → Resource implementations
    │       └── mod.rs           → {Extension}Resources struct
    │
    └── Cargo.toml               → Standalone crate configuration
```

---

## Core Integration Points

MCP extensions integrate with systemprompt-core through these key components:

| Component | Import | Purpose |
|-----------|--------|---------|
| `AppContext` | `systemprompt::system::AppContext` | Configuration, database, registries |
| `DbPool` | `systemprompt::database::DbPool` | PostgreSQL connection pool |
| `McpServerId` | `systemprompt::identifiers::McpServerId` | Typed service identifier |
| `Config` | `systemprompt::models::Config` | Application configuration |
| `create_router()` | `systemprompt_core_mcp::create_router` | Axum router with MCP endpoints |
| `init_logging()` | `systemprompt_core_logging::init_logging` | Database-persisted logging |

### Integration Flow

```
main.rs
    │
    ├─→ Config::init()                          Load configuration
    │
    ├─→ AppContext::new()                       Initialize app context
    │
    ├─→ init_logging(db_pool)                   Setup logging
    │
    ├─→ McpServerId::from_env()                 Get service identity
    │
    ├─→ MyServer::new(db_pool, service_id, ctx) Create server instance
    │
    ├─→ create_router(server, ctx)              Build Axum router
    │
    └─→ axum::serve(listener, router)           Start HTTP server
```

---

## Protocol Compliance

### MCP Version

| Specification | Value |
|---------------|-------|
| Protocol Version | `2024-11-05` |
| rmcp Crate | `0.8.1` |
| Transport | Streamable HTTP with SSE |
| Session Mode | Stateful with 30s keep-alive |

### Required rmcp Features

```toml
rmcp = { version = "0.8.1", features = [
    "server",
    "transport-streamable-http-server",
    "transport-streamable-http-server-session",
    "macros"
] }
```

### ServerHandler Trait

Extensions MUST implement `rmcp::ServerHandler` with these methods:

| Method | Required | Purpose |
|--------|----------|---------|
| `get_info()` | YES | Return server capabilities |
| `initialize()` | YES | Handle client initialization |
| `list_tools()` | YES | Return available tools |
| `call_tool()` | YES | Execute tool requests |
| `list_prompts()` | YES | Return available prompts |
| `get_prompt()` | YES | Return prompt content |
| `list_resources()` | YES | Return available resources |
| `read_resource()` | YES | Return resource content |
| `list_resource_templates()` | YES | Return resource templates |

---

## Server Capabilities

The `get_info()` method returns `ServerInfo` declaring server capabilities:

```rust
ServerInfo {
    protocol_version: ProtocolVersion::V2024_11_05,
    capabilities: ServerCapabilities {
        prompts: Some(PromptsCapability { list_changed: None }),
        tools: Some(ToolsCapability { list_changed: None }),
        resources: Some(ResourcesCapability {
            subscribe: None,
            list_changed: None,
        }),
        ..Default::default()
    },
    server_info: Implementation {
        name: "systemprompt-{name}".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        ..Default::default()
    },
    instructions: None,
}
```

---

## Configuration

### Environment Variables

| Variable | Required | Default | Description |
|----------|----------|---------|-------------|
| `MCP_SERVICE_ID` | NO | Crate default | Service identity (MUST match `mcp_servers` config key) |
| `MCP_PORT` | NO | Crate default | TCP port for HTTP server |
| `DATABASE_URL` | YES | - | PostgreSQL connection string |

### Service Identity

The `MCP_SERVICE_ID` MUST match the key in the `mcp_servers` configuration:

```yaml
mcp_servers:
  systemprompt-infrastructure:    # ← This key
    executable: "systemprompt-infrastructure"
    env:
      MCP_PORT: 5010
      MCP_SERVICE_ID: systemprompt-infrastructure  # ← MUST match
```

### Port Allocation

| Extension | Default Port |
|-----------|--------------|
| systemprompt-admin | 5002 |
| systemprompt-infrastructure | 5010 |
| system-tools | 5003 |

---

## Request Context

Every handler receives `RequestContext<RoleServer>` containing:

- Request metadata
- Session information
- Extensions for custom data

Access extensions via:

```rust
async fn call_tool(
    &self,
    request: CallToolRequestParam,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let extensions = ctx.extensions();
    handle_tool_with_extensions(request, extensions).await
}
```

---

## Logging

Use `tracing` macros with structured fields:

```rust
tracing::info!(
    service_id = %self.service_id,
    tool = %tool_name,
    "Executing tool"
);

tracing::error!(
    error = %e,
    "Tool execution failed"
);
```

Logs are automatically persisted to the database via `init_logging()`.

---

## See Also

- [boundaries.md](./boundaries.md) - Module boundaries and integration rules
- [../implementation/prompts.md](../implementation/prompts.md) - Prompt implementation
- [../implementation/tools.md](../implementation/tools.md) - Tool implementation
