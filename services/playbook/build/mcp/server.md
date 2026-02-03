---
title: "Create MCP Server"
description: "Create an MCP server with tools for AI agents."
author: "SystemPrompt"
slug: "build-04-mcp-extensions-create-mcp-server"
keywords: "mcp, server, tools, ai"
image: ""
kind: "playbook"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# Create MCP Server

Create an MCP server with tools for AI agents. Reference: `extensions/mcp/systemprompt/` for working example.

> **Help**: `{ "command": "core playbooks show build_create-mcp-server" }`

---

## Structure

```
extensions/mcp/{name}/
├── Cargo.toml
├── module.yml
└── src/
    ├── main.rs
    ├── lib.rs
    ├── server/
    │   ├── mod.rs
    │   ├── constructor.rs
    │   └── handlers/
    ├── tools/
    │   ├── mod.rs
    │   └── {tool_name}/
    ├── prompts/
    └── resources/
```

---

## Cargo.toml

```toml
[package]
name = "systemprompt-mcp-my-server"
version = "1.0.0"
edition = "2021"

[[bin]]
name = "systemprompt-mcp-my-server"
path = "src/main.rs"

[dependencies]
systemprompt-core-mcp = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-core-database = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-models = { git = "https://github.com/systempromptio/systemprompt-core" }
systemprompt-identifiers = { git = "https://github.com/systempromptio/systemprompt-core" }
rmcp = "0.8"
tokio = { version = "1.47", features = ["full"] }
axum = "0.8"
tracing = "0.1"
anyhow = "1.0"
```

---

## module.yml

```yaml
name: my-server
display_name: "My MCP Server"
version: "1.0.0"
description: "Provides tools for X, Y, Z"

server:
  port: 5003
  host: "0.0.0.0"

tools:
  - name: my_tool
    description: "Does something useful"
```

---

## Main Entry Point

File: `src/main.rs`. See `extensions/mcp/systemprompt/src/main.rs:1-30` for reference.

```rust
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    systemprompt_core_logging::init();

    let config = load_config()?;
    let server = MyServer::new(config);

    let router = systemprompt_core_mcp::create_router(server);

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %addr, "MCP server listening");

    axum::serve(listener, router).await?;
    Ok(())
}
```

---

## CRITICAL: RBAC and RequestContext

**All `call_tool` handlers MUST extract RequestContext for proper execution tracking.**

Without this, artifact persistence fails with foreign key constraint errors.

```rust
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::models::{ToolExecutionRequest, ToolExecutionResult};
use chrono::Utc;

async fn call_tool(
    &self,
    request: CallToolRequestParams,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    let started_at = Utc::now();

    let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;
    let authenticated_ctx = auth_result.expect_authenticated("requires OAuth")?;
    let request_context = authenticated_ctx.context.clone();

    let execution_request = ToolExecutionRequest {
        tool_name: request.name.to_string(),
        server_name: self.service_id.to_string(),
        input: serde_json::to_value(&request.arguments).unwrap_or_default(),
        started_at,
        context: request_context.clone(),
        request_method: Some("mcp".to_string()),
        request_source: Some("my-server".to_string()),
        ai_tool_call_id: None,
    };

    let mcp_execution_id = self.tool_usage_repo
        .start_execution(&execution_request).await?;

    let result = handle_tool_call(&request.name, request, &request_context).await;

    self.tool_usage_repo.complete_execution(&mcp_execution_id, &ToolExecutionResult {
        output: result.as_ref().ok().and_then(|r| r.structured_content.clone()),
        output_schema: None,
        status: if result.is_ok() { "success" } else { "failed" }.to_string(),
        error_message: result.as_ref().err().map(|e| e.message.to_string()),
        started_at,
        completed_at: Utc::now(),
    }).await.ok();

    result
}
```

-> See [MCP Checklist](build_mcp-checklist) for complete requirements.

---

## Tool Implementation

File: `src/tools/mod.rs`. See `extensions/mcp/systemprompt/src/tools/` for reference.

```rust
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool("my_tool", "My Tool", "Description", input_schema(), output_schema()),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    ctx: &RequestContext,
) -> Result<CallToolResult, McpError> {
    match name {
        "my_tool" => handle_my_tool(ctx, request).await,
        _ => Err(McpError::method_not_found())
    }
}
```

---

## Error Handling

```rust
use systemprompt_traits::ExtensionError;

#[derive(Error, Debug)]
pub enum ToolError {
    #[error("Missing parameter: {name}")]
    MissingParameter { name: String },

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl ExtensionError for ToolError {
    fn code(&self) -> &'static str {
        match self {
            Self::MissingParameter { .. } => "MISSING_PARAMETER",
            Self::Io(_) => "IO_ERROR",
            Self::Database(_) => "DATABASE_ERROR",
        }
    }

    fn status(&self) -> StatusCode {
        match self {
            Self::MissingParameter { .. } => StatusCode::BAD_REQUEST,
            Self::Io(_) | Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}
```

---

## Checklist

- [ ] Package name follows `systemprompt-mcp-{name}` pattern
- [ ] Located in `extensions/mcp/`
- [ ] `module.yml` with server metadata and tools
- [ ] Binary target defined in `Cargo.toml`
- [ ] Initializes logging
- [ ] Loads configuration
- [ ] Registers tools
- [ ] Binds to configured port
- [ ] **CRITICAL**: `call_tool` extracts RequestContext via `enforce_rbac_from_registry`
- [ ] **CRITICAL**: Uses `ToolUsageRepository` for execution tracking

---

## Tool Quality

- [ ] Each tool has unique name
- [ ] Clear description of purpose
- [ ] Input schema defines all parameters
- [ ] Output schema documents response format
- [ ] Proper error handling with descriptive messages
- [ ] Structured logging with context

---

## Code Quality

| Metric | Limit |
|--------|-------|
| File length | 300 lines |
| Function length | 75 lines |
| No `unwrap()` | Use `?` or `ok_or_else()` |
| No inline comments | Code documents itself |

---

## Troubleshooting

| Issue | Solution |
|-------|----------|
| Port in use | Change port in `module.yml` |
| Tool not found | Check tool name in registration |
| Config error | Verify `module.yml` syntax |

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build -p systemprompt-mcp-{name}` |
| Run | `cargo run -p systemprompt-mcp-{name}` |
| Lint | `cargo clippy -p systemprompt-mcp-{name} -- -D warnings` |
| Format | `cargo fmt -p systemprompt-mcp-{name} -- --check` |

---

## Related

-> See [Rust Standards](../06-standards/rust-standards.md) for code style
-> See [MCP Extension](../../documentation/extensions/domains/mcp.md) for domain reference