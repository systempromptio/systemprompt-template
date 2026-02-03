---
title: "MCP Extensions"
description: "Build standalone MCP server extensions that expose tools for AI agents via the Model Context Protocol."
author: "SystemPrompt Team"
slug: "extensions/domains/mcp"
keywords: "mcp, extensions, tools, servers, protocol, standalone"
image: "/files/images/docs/extensions-mcp.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-01-30"
updated_at: "2026-02-02"
---

# MCP Extensions

MCP (Model Context Protocol) extensions are standalone binaries that expose tools for AI agents. Unlike library extensions that compile into the main binary, MCP servers run as separate processes. They listen on TCP ports and serve tool requests via the MCP protocol, enabling AI clients like Claude to execute operations in your SystemPrompt environment.

## Standalone Binary Pattern

MCP extensions are not library extensions. They do not implement the Extension trait or register with the runtime. Instead, they are independent executables with their own entry point, their own database connection, and their own lifecycle.

This separation provides important benefits:

- **Independent scaling** - Run multiple MCP server instances on different machines
- **Process isolation** - MCP server crashes do not affect the main runtime
- **Resource control** - Allocate specific CPU and memory limits
- **Separate deployment** - Update MCP servers without redeploying the main binary

For CLI tools that agents invoke via subprocess rather than the MCP protocol, see [CLI Extensions](/documentation/extensions/domains/cli).

## Project Structure

```
extensions/mcp/systemprompt/
├── Cargo.toml
└── src/
    ├── main.rs         # Entry point with bootstrap
    ├── lib.rs          # Server implementation
    ├── server.rs       # MCP protocol handler
    ├── tools.rs        # Tool definitions
    └── artifacts.rs    # Artifact storage
```

## Entry Point

The MCP server entry point bootstraps from SystemPrompt's configuration system and starts an HTTP server:

```rust
use anyhow::{Context, Result};
use std::{env, sync::Arc};
use systemprompt::identifiers::McpServerId;
use systemprompt::models::{Config, ProfileBootstrap, SecretsBootstrap};
use systemprompt::system::AppContext;
use tokio::net::TcpListener;

const DEFAULT_SERVICE_ID: &str = "systemprompt";
const DEFAULT_PORT: u16 = 5010;

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
    let server = SystempromptServer::new(ctx.db_pool().clone(), service_id.clone());
    let router = systemprompt::mcp::create_router(server, ctx.db_pool());

    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(&addr).await?;

    tracing::info!(
        service_id = %service_id,
        addr = %addr,
        "SystemPrompt MCP server listening"
    );

    axum::serve(listener, router).await?;

    Ok(())
}
```

Key bootstrap steps:

1. **ProfileBootstrap** - Loads the active profile configuration
2. **SecretsBootstrap** - Loads secrets from environment or files
3. **Config::init** - Initializes the configuration system
4. **AppContext** - Creates database pool and shared resources

This allows MCP servers to share the same configuration and database as the main runtime.

## Server Implementation

The MCP server implements the protocol handler and registers tools:

```rust
use rmcp::{Server, ServerHandler, Tool, ToolResult};
use sqlx::PgPool;
use std::sync::Arc;

pub struct SystempromptServer {
    pool: Arc<PgPool>,
    service_id: McpServerId,
}

impl SystempromptServer {
    pub fn new(pool: Arc<PgPool>, service_id: McpServerId) -> Self {
        Self { pool, service_id }
    }
}

#[async_trait]
impl ServerHandler for SystempromptServer {
    async fn list_tools(&self) -> Vec<Tool> {
        vec![
            Tool::new("execute_cli", "Execute a SystemPrompt CLI command"),
            Tool::new("query_content", "Query content from the database"),
            Tool::new("list_agents", "List configured agents"),
        ]
    }

    async fn call_tool(&self, name: &str, args: Value) -> ToolResult {
        match name {
            "execute_cli" => self.execute_cli(args).await,
            "query_content" => self.query_content(args).await,
            "list_agents" => self.list_agents(args).await,
            _ => ToolResult::error(format!("Unknown tool: {}", name)),
        }
    }
}
```

## Tool Definitions

Each tool has a name, description, and input schema:

```rust
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
struct ExecuteCliInput {
    command: String,
    args: Vec<String>,
}

impl SystempromptServer {
    async fn execute_cli(&self, args: Value) -> ToolResult {
        let input: ExecuteCliInput = serde_json::from_value(args)?;

        let output = std::process::Command::new("systemprompt")
            .arg(&input.command)
            .args(&input.args)
            .output()?;

        if output.status.success() {
            ToolResult::success(String::from_utf8_lossy(&output.stdout))
        } else {
            ToolResult::error(String::from_utf8_lossy(&output.stderr))
        }
    }

    fn execute_cli_schema() -> Value {
        json!({
            "type": "object",
            "properties": {
                "command": {
                    "type": "string",
                    "description": "CLI command to execute"
                },
                "args": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Command arguments"
                }
            },
            "required": ["command"]
        })
    }
}
```

## Configuration

Register MCP servers in `services/mcp/`:

```yaml
# services/mcp/systemprompt.yaml
mcp_servers:
  systemprompt:
    binary: "systemprompt-mcp-agent"
    port: 5010
    endpoint: "http://localhost:8080/api/v1/mcp/systemprompt/mcp"
    enabled: true

    oauth:
      required: true
      scopes: ["admin"]
```

## Cargo Configuration

```toml
[package]
name = "systemprompt-mcp-agent"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "systemprompt-mcp-agent"
path = "src/main.rs"

[dependencies]
systemprompt = { workspace = true }
rmcp = { workspace = true }
axum = { workspace = true }
tokio = { workspace = true, features = ["full"] }
sqlx = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
anyhow = { workspace = true }
async-trait = { workspace = true }
tracing = { workspace = true }
```

## Building

```bash
# Build MCP server
cargo build --release -p systemprompt-mcp-agent

# Or build all MCP servers
systemprompt build mcp --release
```

## Testing

```bash
# Test server connectivity
systemprompt plugins mcp test systemprompt

# List available tools
systemprompt plugins mcp tools systemprompt

# Start server manually
./target/release/systemprompt-mcp-agent
```

## Claude Desktop Integration

Add to Claude Desktop configuration:

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

## Detailed Documentation

For in-depth guides on specific topics:

| Topic | Document |
|-------|----------|
| Tool Organization | [Tool Structure](/documentation/extensions/mcp/tool-structure) |
| Resources & Templates | [MCP Resources](/documentation/extensions/mcp/resources) |
| Skill Integration | [MCP Skills](/documentation/extensions/mcp/skills) |
| Response Patterns | [MCP Responses](/documentation/extensions/mcp/responses) |
| AI Integration | [MCP AI Integration](/documentation/extensions/mcp-ai-integration) |

## Related Playbooks

| Playbook | Description |
|----------|-------------|
| [MCP Tutorial](/playbooks/build/mcp-tutorial) | Build your first MCP server |
| [MCP Tool Patterns](/playbooks/build/mcp-tools) | Handler and schema patterns |
| [MCP Artifacts](/playbooks/build/mcp-artifacts) | Artifacts and UI resources |
| [MCP Checklist](/playbooks/build/mcp-checklist) | Complete requirements checklist |