---
title: "MCP Server Tutorial"
description: "Step-by-step guide to building your first MCP server from scratch."
keywords:
  - mcp
  - tutorial
  - server
  - build
category: build
---

# MCP Server Tutorial

> **Help**: `{ "command": "core playbooks show build_mcp-tutorial" }`

**This tutorial walks you through building a complete MCP server from scratch.**

**Prerequisites:**
- [ ] Read [Coding Standards](guide_coding-standards)
- [ ] Read [MCP Checklist](build_mcp-checklist)
- [ ] Rust toolchain installed
- [ ] Access to SystemPrompt codebase

---

## Part 1: Create Basic Server

### Step 1: Create Directory Structure

```bash
mkdir -p extensions/mcp/my-server/src/{server,tools}
```

Target structure:
```
extensions/mcp/my-server/
├── Cargo.toml
└── src/
    ├── main.rs             # Entry point
    ├── lib.rs              # Library exports
    ├── server.rs           # ServerHandler implementation
    └── tools/
        ├── mod.rs          # Tool registration
        └── my_tool/        # Each tool in subdirectory
            ├── mod.rs
            ├── handler.rs
            └── helpers.rs
```

### Step 2: Create Cargo.toml

Create `extensions/mcp/my-server/Cargo.toml`:

```toml
[package]
name = "systemprompt-mcp-my-server"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "systemprompt-mcp-my-server"
path = "src/main.rs"

[lib]
path = "src/lib.rs"

[dependencies]
# SystemPrompt facade (provides all core functionality)
systemprompt = { workspace = true }

# MCP Protocol
rmcp = { workspace = true }

# Async runtime
tokio = { workspace = true, features = ["full"] }
axum = { workspace = true }

# Serialization
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }

# Error handling
anyhow = { workspace = true }
thiserror = { workspace = true }

# Logging
tracing = { workspace = true }
```

### Step 3: Create main.rs (Entry Point)

Create `extensions/mcp/my-server/src/main.rs`:

```rust
use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt::identifiers::McpServerId;
use systemprompt::models::{Config, ProfileBootstrap, SecretsBootstrap};
use systemprompt::system::AppContext;
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "my-server";
const DEFAULT_PORT: u16 = 5050;

#[tokio::main]
async fn main() -> Result<()> {
    // Bootstrap from profile and secrets
    ProfileBootstrap::init().context("Failed to initialize profile")?;
    SecretsBootstrap::init().context("Failed to initialize secrets")?;
    Config::init().context("Failed to initialize configuration")?;

    // Create application context with database access
    let ctx = Arc::new(
        AppContext::new()
            .await
            .context("Failed to initialize application context")?,
    );

    systemprompt::logging::init_logging(ctx.db_pool().clone());

    // Get service ID from environment or use default
    let service_id = McpServerId::from_env().unwrap_or_else(|_| {
        tracing::warn!("MCP_SERVICE_ID not set, using default: {DEFAULT_SERVICE_ID}");
        McpServerId::new(DEFAULT_SERVICE_ID)
    });

    let port = env::var("MCP_PORT")
        .ok()
        .and_then(|p| p.parse::<u16>().ok())
        .unwrap_or(DEFAULT_PORT);

    // Create MCP server and router
    let server = systemprompt_mcp_my_server::MyServer::new(
        ctx.db_pool().clone(),
        service_id.clone(),
    );
    let router = systemprompt::mcp::create_router(server, ctx.db_pool());

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "My MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
```

> **Note:** The MCP spawner automatically passes environment variables like `AI_CONFIG_PATH`, `SYSTEM_PATH`, `DATABASE_URL`, and API keys. See [MCP Checklist - Environment Variables](build_mcp-checklist#environment-variables) for the complete list.

### Step 4: Create lib.rs

Create `extensions/mcp/my-server/src/lib.rs`:

```rust
pub mod server;
pub mod tools;

pub use server::MyServer;
```

### Step 5: Create server.rs (ServerHandler)

Create `extensions/mcp/my-server/src/server.rs`:

```rust
use crate::tools;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, InitializeRequestParams,
    InitializeResult, ListToolsResult, PaginatedRequestParams, ProtocolVersion,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;

#[derive(Clone)]
pub struct MyServer {
    db_pool: DbPool,
    service_id: McpServerId,
}

impl MyServer {
    #[must_use]
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self { db_pool, service_id }
    }
}

impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: ProtocolVersion::V_2024_11_05,
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .build(),
            server_info: Implementation {
                name: format!("My Server ({})", self.service_id),
                version: env!("CARGO_PKG_VERSION").to_string(),
                icons: None,
                title: Some("My MCP Server".to_string()),
                website_url: None,
            },
            instructions: Some(
                "This server provides tools for X, Y, Z.".to_string()
            ),
        }
    }

    async fn initialize(
        &self,
        _request: InitializeRequestParams,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<InitializeResult, McpError> {
        tracing::info!("My MCP server initialized");
        Ok(self.get_info())
    }

    async fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListToolsResult, McpError> {
        Ok(ListToolsResult {
            tools: tools::list_tools(),
            next_cursor: None,
            meta: None,
        })
    }

    async fn call_tool(
        &self,
        request: CallToolRequestParams,
        ctx: RequestContext<RoleServer>,
    ) -> Result<CallToolResult, McpError> {
        let tool_name = request.name.to_string();

        // Optional: Enforce authentication
        // let auth = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;

        tools::handle_tool_call(&tool_name, request, &self.db_pool).await
    }
}
```

### Step 6: Create tools/mod.rs

Create `extensions/mcp/my-server/src/tools/mod.rs`:

```rust
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::database::DbPool;

pub mod my_tool;

pub fn list_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "my_tool",
            "My Tool",
            "Does something useful. Provide 'input' parameter.",
            my_tool::input_schema(),
            my_tool::output_schema(),
        ),
    ]
}

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: serde_json::Value,
    output_schema: serde_json::Value,
) -> Tool {
    let input_obj = input_schema.as_object().cloned().unwrap_or_default();
    let output_obj = output_schema.as_object().cloned().unwrap_or_default();

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
        meta: None,
    }
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParams,
    db_pool: &DbPool,
) -> Result<CallToolResult, McpError> {
    match name {
        "my_tool" => my_tool::handle(db_pool, request).await,
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: '{name}'"),
            None,
        )),
    }
}
```

---

## Part 2: Add a Tool

### Step 1: Create Tool Directory

```bash
mkdir -p extensions/mcp/my-server/src/tools/my_tool
```

### Step 2: Create mod.rs

Create `extensions/mcp/my-server/src/tools/my_tool/mod.rs`:

```rust
mod handler;
mod helpers;

pub use handler::handle;
pub use helpers::{input_schema, output_schema};
```

### Step 3: Create helpers.rs (Schemas)

Create `extensions/mcp/my-server/src/tools/my_tool/helpers.rs`:

```rust
use serde_json::json;

pub fn input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "input": {
                "type": "string",
                "description": "The input to process"
            },
            "options": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional processing options"
            }
        },
        "required": ["input"]
    })
}

pub fn output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "result": {
                "type": "string",
                "description": "The processed result"
            },
            "status": {
                "type": "string",
                "enum": ["success", "error"],
                "description": "Operation status"
            }
        }
    })
}

pub fn extract_string_array(
    args: &serde_json::Map<String, serde_json::Value>,
    key: &str,
) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default()
}
```

### Step 4: Create handler.rs (Implementation)

Create `extensions/mcp/my-server/src/tools/my_tool/handler.rs`:

```rust
use rmcp::model::{CallToolRequestParams, CallToolResult, Content};
use rmcp::ErrorData as McpError;
use serde_json::json;
use systemprompt::database::DbPool;

use super::helpers::extract_string_array;

pub async fn handle(
    _db_pool: &DbPool,
    request: CallToolRequestParams,
) -> Result<CallToolResult, McpError> {
    // Extract arguments
    let args = request.arguments.as_ref().ok_or_else(|| {
        McpError::invalid_request("Missing arguments", None)
    })?;

    // Get required parameter
    let input = args
        .get("input")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::invalid_params("Missing required parameter: input", None)
        })?;

    // Get optional parameter
    let options = extract_string_array(args, "options");

    // Process the input (your business logic here)
    let result = format!("Processed: {}", input);

    tracing::info!(
        input = %input,
        options = ?options,
        "Tool executed successfully"
    );

    // Return result with both human-readable and structured content
    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Successfully processed input.\n\nResult: {result}"
        ))],
        structured_content: Some(json!({
            "result": result,
            "status": "success",
            "options_used": options
        })),
        is_error: Some(false),
        meta: None,
    })
}
```

---

## Part 3: Register the Server

### Step 1: Add to Workspace

Edit `Cargo.toml` (workspace root) and add to members:

```toml
[workspace]
members = [
    # ... existing members ...
    "extensions/mcp/my-server",
]
```

### Step 2: Create Config File

Create `services/mcp/my-server.yaml`:

```yaml
mcp_servers:
  my-server:
    binary: "systemprompt-mcp-my-server"
    package: "my-server"
    port: 5050
    endpoint: "http://localhost:8080/api/v1/mcp/my-server/mcp"
    enabled: true
    display_in_web: true
    description: "My MCP Server - does X, Y, Z"

    oauth:
      required: true
      scopes: ["admin"]
      audience: "mcp"
      client_id: null
```

### Step 3: Add to Config Includes

Edit `services/config/config.yaml`:

```yaml
includes:
  # ... existing includes ...
  - ../mcp/my-server.yaml
```

---

## Part 4: Build and Test

### Build

```bash
# Build via CLI (recommended)
systemprompt build mcp

# Or build specific server
cargo build -p systemprompt-mcp-my-server

# Lint
cargo clippy -p systemprompt-mcp-my-server -- -D warnings

# Format
cargo fmt -p systemprompt-mcp-my-server -- --check
```

### Test

```bash
# Start services
systemprompt infra services start --all

# Check server status
systemprompt plugins mcp status

# List tools
systemprompt plugins mcp tools

# Call tool
systemprompt plugins mcp call my-server my_tool --args '{"input": "hello"}'
```

---

## Verification Checklist

- [ ] Server starts without errors
- [ ] `systemprompt plugins mcp status` shows server as running
- [ ] `systemprompt plugins mcp tools` lists your tools
- [ ] Tool calls return expected results
- [ ] `cargo clippy` passes with no warnings
- [ ] `cargo fmt --check` passes

---

## Next Steps

- [Tool Patterns](build_mcp-tools) — Modular tool organization
- [Artifacts](build_mcp-artifacts) — Creating and storing artifacts
- [MCP Checklist](build_mcp-checklist) — Full requirements checklist
- [Rust Standards](build_rust-standards) — Code quality guidelines

---

## Quick Reference

| Task | Command |
|------|---------|
| Build all MCP | `systemprompt build mcp` |
| Build single | `cargo build -p systemprompt-mcp-my-server` |
| Check status | `systemprompt plugins mcp status` |
| List tools | `systemprompt plugins mcp tools` |
| Call tool | `systemprompt plugins mcp call {server} {tool} --args '{json}'` |
| Lint | `cargo clippy -p systemprompt-mcp-my-server -- -D warnings` |
