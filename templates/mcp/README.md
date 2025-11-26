# MCP Server Templates

This directory contains example MCP (Model Context Protocol) server implementations for SystemPrompt OS.

## Structure

```
templates/mcp/
├── README.md                    # This file
├── template/                    # Basic MCP server template
└── systemprompt-admin/          # Admin MCP server (full example)
```

## Usage

### 1. Create New MCP Server

Copy the template:

```bash
cp -r templates/mcp/template templates/mcp/my-mcp-server
cd templates/mcp/my-mcp-server
```

### 2. Configure Server

Edit `Cargo.toml`:

```toml
[package]
name = "my-mcp-server"
version = "0.1.0"
```

### 3. Implement Tools

Add tools in `src/tools/mod.rs`:

```rust
use serde_json::{json, Value};
use anyhow::Result;

pub async fn my_tool(params: Value) -> Result<Value> {
    // Tool implementation
    Ok(json!({"result": "success"}))
}
```

### 4. Register Tools

Update `src/server.rs` to register your tools:

```rust
server.add_tool(
    "my-tool",
    "Description of what the tool does",
    json!({"type": "object", "properties": {...}}),
    my_tool,
);
```

### 5. Register in SystemPrompt

Add to `/metadata/mcp/my-mcp-server.json`:

```json
{
  "name": "my-mcp-server",
  "displayName": "My MCP Server",
  "description": "Description of the MCP server",
  "version": "0.1.0",
  "transport": "stdio",
  "command": "cargo",
  "args": ["run", "--bin", "my-mcp-server"],
  "env": {},
  "tools": [
    {
      "name": "my-tool",
      "description": "Tool description",
      "inputSchema": {...}
    }
  ]
}
```

### 6. Start Server

```bash
just mcp start my-mcp-server
```

## Template Structure

### Basic Template (`template/`)

Minimal MCP server with:
- Tool registration framework
- Prompt management
- Resource handling
- Error handling

**Files:**
- `Cargo.toml` - Package configuration
- `src/lib.rs` - Library exports
- `src/server.rs` - MCP server implementation
- `src/tools/mod.rs` - Tool implementations
- `src/prompts/mod.rs` - Prompt templates

### Full Example (`systemprompt-admin/`)

Production MCP server with:
- Agent CRUD operations
- System analytics
- Complex tool schemas
- Input validation
- Database integration

**Features:**
- Multi-operation tools (create, read, update, delete)
- Rich input validation
- Structured output schemas
- Error handling patterns

## Tool Development

### Basic Tool Pattern

```rust
use serde_json::{json, Value};
use anyhow::{Result, anyhow};

pub async fn example_tool(params: Value) -> Result<Value> {
    // 1. Extract parameters
    let name = params["name"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing 'name' parameter"))?;

    // 2. Validate input
    if name.is_empty() {
        return Err(anyhow!("Name cannot be empty"));
    }

    // 3. Execute logic
    let result = perform_operation(name).await?;

    // 4. Return structured output
    Ok(json!({
        "success": true,
        "data": result,
        "message": "Operation completed"
    }))
}
```

### Tool Registration

```rust
use rmcp::protocol::Tool;

// Define input schema
let schema = json!({
    "type": "object",
    "properties": {
        "name": {
            "type": "string",
            "description": "Name parameter"
        }
    },
    "required": ["name"]
});

// Register tool
server.add_tool(
    "example-tool",
    "Description of the tool",
    schema,
    example_tool,
);
```

### Error Handling

```rust
use anyhow::{Result, Context};

pub async fn safe_tool(params: Value) -> Result<Value> {
    let data = fetch_data()
        .await
        .context("Failed to fetch data")?;

    process_data(&data)
        .context("Failed to process data")?;

    Ok(json!({"status": "success"}))
}
```

## Prompts

MCP servers can provide reusable prompts:

```rust
use rmcp::protocol::Prompt;

server.add_prompt(
    "example-prompt",
    "Description of the prompt",
    vec![
        ("param1", "Parameter description", true),
    ],
    |params| async move {
        let param1 = params.get("param1")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        Ok(format!("Prompt content with {}", param1))
    },
);
```

## Resources

MCP servers can expose resources (files, data):

```rust
use rmcp::protocol::Resource;

server.add_resource(
    "resource://example",
    "Description of the resource",
    "text/plain",
    || async move {
        Ok("Resource content".to_string())
    },
);
```

## Testing

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tool() {
        let params = json!({"name": "test"});
        let result = example_tool(params).await.unwrap();

        assert_eq!(result["success"], true);
    }
}
```

### Integration Tests

```bash
# Start MCP server
just mcp start my-mcp-server

# Test tool execution
just mcp exec my-mcp-server my-tool '{"name": "test"}'

# View logs
just log
```

## Best Practices

1. **Clear tool names** - Use descriptive, action-oriented names
2. **Validate inputs** - Check all parameters before execution
3. **Structured outputs** - Return consistent JSON schemas
4. **Error context** - Use `.context()` for helpful error messages
5. **Document schemas** - Add descriptions to all schema fields
6. **Async by default** - Use `async/await` for all operations
7. **Resource cleanup** - Handle cleanup in Drop implementations

## Common Patterns

### CRUD Operations

See `systemprompt-admin/src/tools/agents/handlers/` for examples of:
- Create operations with validation
- Read operations with filtering
- Update operations with partial updates
- Delete operations with confirmations

### Database Integration

```rust
use systemprompt_database::DatabaseProvider;

pub async fn db_tool(
    params: Value,
    db: &dyn DatabaseProvider,
) -> Result<Value> {
    let row = db.fetch_optional(
        "SELECT * FROM table WHERE id = ?",
        &[&params["id"]],
    ).await?;

    Ok(json!({"data": row}))
}
```

### Authentication

```rust
pub async fn auth_tool(params: Value) -> Result<Value> {
    let token = params["token"]
        .as_str()
        .ok_or_else(|| anyhow!("Missing token"))?;

    verify_token(token)?;

    // Proceed with authenticated operation
    Ok(json!({"authenticated": true}))
}
```

## Troubleshooting

**Server not starting:**
```bash
# Check server status
just mcp status

# View detailed logs
just log

# Restart server
just mcp restart my-mcp-server
```

**Tool not found:**
```bash
# List available tools
just mcp exec my-mcp-server --list-tools

# Check registration in server.rs
grep "add_tool" templates/mcp/my-mcp-server/src/server.rs
```

**Compilation errors:**
```bash
# Build server standalone
cd templates/mcp/my-mcp-server
cargo build

# Check dependencies
cargo tree
```

## See Also

- `/templates/agents/` - Agent templates
- `/metadata/mcp/` - MCP server registry
- `CLAUDE.md` - MCP architecture overview
- [MCP Protocol Spec](https://spec.modelcontextprotocol.io/)
