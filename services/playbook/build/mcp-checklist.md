---
title: "MCP Server Checklist Playbook"
description: "Complete checklist for building MCP servers on systemprompt-core."
keywords:
  - mcp
  - server
  - checklist
  - build
---

# MCP Server Checklist

**Applies to:** All MCP server crates in `extensions/mcp/`

> **Help**: `{ "command": "playbook build" }` via `systemprompt_help`

---

## Core Principle

**MCP servers are Rust code and belong in `/extensions/mcp/`, not `/services/mcp/`.**

If it's a Rust crate, it's an extension.

---

## Required Structure

```
extensions/mcp/{name}/
├── Cargo.toml
├── module.yml              # Server configuration
└── src/
    ├── main.rs             # Entry point
    ├── lib.rs              # Library for testing
    ├── server/             # Server implementation
    │   ├── mod.rs
    │   ├── constructor.rs  # Server initialization
    │   └── handlers/
    │       ├── mod.rs
    │       ├── tools.rs
    │       └── initialization.rs
    ├── tools/              # Tool implementations
    │   ├── mod.rs          # Registration & dispatch
    │   └── {tool_name}/    # Each tool in subdirectory
    │       ├── mod.rs
    │       ├── models.rs
    │       ├── repository.rs
    │       └── schema.rs
    ├── prompts/            # Prompt templates (optional)
    └── resources/          # Resource handlers (optional)
```

---

## Cargo.toml

- [ ] Package name follows `systemprompt-mcp-{name}` pattern
- [ ] Located in `extensions/mcp/`, NOT `services/mcp/`
- [ ] Correct dependencies:
  - `systemprompt-core-mcp` (router, protocol)
  - `systemprompt-models` (shared types)
  - `rmcp` (MCP protocol)
- [ ] Binary target defined

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

- [ ] Server metadata defined
- [ ] Default port specified
- [ ] Tools documented

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

- [ ] Initializes logging
- [ ] Loads configuration
- [ ] Creates server instance
- [ ] Registers router
- [ ] Binds to configured port

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

## Tool Implementation

### Current Pattern (Manual Dispatch)

```rust
pub fn register_tools() -> Vec<Tool> {
    vec![
        create_tool("my_tool", "My Tool", "Description", input_schema(), output_schema()),
    ]
}

pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParam,
    db_pool: &DbPool,
) -> Result<CallToolResult, McpError> {
    match name {
        "my_tool" => handle_my_tool(db_pool, request).await,
        _ => Err(McpError::method_not_found())
    }
}
```

### Recommended Pattern (Proc Macro - Future)

When available, prefer proc macros for type-safe tool definitions:

```rust
#[mcp_server]
pub struct MyServer {
    db_pool: DbPool,
    logger: LogService,
}

#[mcp_tools]
impl MyServer {
    /// My tool description
    #[tool(description = "Does something useful")]
    async fn my_tool(
        &self,
        #[arg(description = "Input parameter")] input: String,
    ) -> Result<MyOutput, ToolError> {
        // Implementation
    }
}
```

Benefits of proc macro approach:
- Schema generated from function signature
- Dispatch generated from impl block
- Description co-located with implementation
- Compile-time type checking

---

## Tool Quality

- [ ] Each tool has unique name
- [ ] Clear description of purpose
- [ ] Input schema defines all parameters
- [ ] Output schema documents response format
- [ ] No business logic in tool handlers (delegate to services)
- [ ] Proper error handling with descriptive messages
- [ ] Structured logging with context
- [ ] Input validation before processing

---

## Error Handling

- [ ] Implements `ExtensionError` trait for tool errors
- [ ] Converts to MCP protocol errors via `to_mcp_error()`
- [ ] Machine-readable error codes

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

## Prompts (if applicable)

- [ ] Each prompt has unique name
- [ ] Clear description of purpose
- [ ] Argument definitions with types
- [ ] Template with proper placeholders

---

## Resources (if applicable)

- [ ] Each resource has unique URI pattern
- [ ] Proper MIME type specification
- [ ] Efficient data retrieval
- [ ] Caching where appropriate

---

## Boundary Rules

- [ ] Located in `extensions/mcp/`, NOT `services/mcp/`
- [ ] Can import from `systemprompt-core-mcp`
- [ ] Can import from `systemprompt-models`
- [ ] Can import from other extensions for tool implementations
- [ ] Uses services for business logic (no direct repository access in handlers)

---

## Configuration

- [ ] Port configurable via environment or config file
- [ ] Secrets loaded from environment variables
- [ ] Sensible defaults for optional settings

---

## Idiomatic Rust

- [ ] Iterator chains over imperative loops
- [ ] `?` operator for error propagation
- [ ] No unnecessary `.clone()`
- [ ] Async/await used correctly
- [ ] Implements `ExtensionError` trait
- [ ] Single dispatch point for tools

---

## Code Quality

- [ ] File length <= 300 lines
- [ ] Function length <= 75 lines
- [ ] Cognitive complexity <= 15
- [ ] Function parameters <= 5
- [ ] No `unsafe`
- [ ] No `unwrap()` / `panic!()`
- [ ] No inline comments (`//`)
- [ ] No TODO/FIXME/HACK
- [ ] `cargo clippy -p {crate} -- -D warnings` passes
- [ ] `cargo fmt -p {crate} -- --check` passes

---

## Testing

```bash
# Build MCP server
cargo build -p systemprompt-mcp-my-server

# Run in development
cargo run -p systemprompt-mcp-my-server

# Lint
cargo clippy -p systemprompt-mcp-my-server -- -D warnings

# Format
cargo fmt -p systemprompt-mcp-my-server -- --check
```

---

## Migration from services/mcp/ to extensions/mcp/

If your MCP server is currently in `services/mcp/`:

1. Move directory: `mv services/mcp/my-server extensions/mcp/my-server`
2. Update `Cargo.toml` workspace members
3. Update any references in `justfile`
4. Update crate name to `systemprompt-mcp-{name}` pattern
5. Update imports in dependent code

---

## Quick Reference

| Task | Command |
|------|---------|
| Build | `cargo build -p systemprompt-mcp-{name}` |
| Run | `cargo run -p systemprompt-mcp-{name}` |
| Lint | `cargo clippy -p systemprompt-mcp-{name} -- -D warnings` |
| Format | `cargo fmt -p systemprompt-mcp-{name} -- --check` |
