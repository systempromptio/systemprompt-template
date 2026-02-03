---
title: "MCP Tool Patterns"
description: "Production patterns for organizing and implementing MCP tools with modular structure."
keywords:
  - mcp
  - tools
  - patterns
  - handlers
category: build
---

# MCP Tool Patterns

> **Help**: `{ "command": "core playbooks show build_mcp-tools" }`

**Production patterns for building well-organized, maintainable MCP tools.**

**Prerequisites:**
- [ ] Read [MCP Tutorial](build_mcp-tutorial)
- [ ] Read [MCP Checklist](build_mcp-checklist)

**Reference Implementations:**
- `extensions/mcp/content-manager/src/tools/` — Full production example
- `extensions/mcp/systemprompt/src/tools.rs` — Simpler example

---

## Tool Directory Structure

Each tool should have its own subdirectory with clear separation of concerns:

```
extensions/mcp/{server}/src/tools/
├── mod.rs                  # Registration & dispatch
├── shared/                 # Shared utilities
│   └── mod.rs
└── {tool_name}/            # Each tool in subdirectory
    ├── mod.rs              # Re-exports
    ├── handler.rs          # Implementation
    └── helpers.rs          # Schemas, utilities
```

### Why This Structure?

| Benefit | Description |
|---------|-------------|
| **Separation** | Schemas separate from logic, easy to update |
| **Testability** | Each handler can be unit tested in isolation |
| **Maintainability** | Changes localized to single directory |
| **Discoverability** | Tool name = directory name |

---

## Tool Registration (mod.rs)

The top-level `mod.rs` handles registration and dispatch:

```rust
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::database::DbPool;

// Import each tool module
pub mod research_blog;
pub mod create_blog_post;
pub mod shared;

// Re-export handlers for use outside module
pub use research_blog::handle as handle_research_blog;
pub use create_blog_post::handle as handle_create_blog_post;

/// Register all tools with their schemas
pub fn list_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "research_blog",
            "Research Blog Topic",
            "Research a topic using Google Search. Returns artifact_id.",
            research_blog::input_schema(),
            research_blog::output_schema(),
        ),
        create_tool(
            "create_blog_post",
            "Create Blog Post",
            "Create a blog post from research artifact.",
            create_blog_post::input_schema(),
            create_blog_post::output_schema(),
        ),
    ]
}

/// Helper to create Tool with proper schema wrapping
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

/// Dispatch tool calls to handlers
pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParams,
    ctx: RequestContext,
    db_pool: &DbPool,
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    artifact_repo: &ArtifactRepository,
    progress: Option<ProgressCallback>,
) -> Result<CallToolResult, McpError> {
    match name {
        "research_blog" => {
            research_blog::handle(
                db_pool, request, ctx, ai_service,
                skill_loader, artifact_repo, progress
            ).await
        }
        "create_blog_post" => {
            create_blog_post::handle(
                db_pool, request, ctx, ai_service,
                skill_loader, artifact_repo, progress
            ).await
        }
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: '{name}'"),
            None,
        )),
    }
}
```

---

## Tool Module Pattern

Each tool has a simple `mod.rs` that re-exports:

```rust
// tools/research_blog/mod.rs
mod handler;
mod helpers;

pub use handler::handle;
pub use helpers::{input_schema, output_schema};
```

---

## Schema Definition Pattern (helpers.rs)

Schemas define tool inputs and outputs using JSON Schema:

```rust
// tools/research_blog/helpers.rs
use serde_json::json;

pub fn input_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "topic": {
                "type": "string",
                "description": "The topic to research"
            },
            "skill_id": {
                "type": "string",
                "description": "Must be 'research_blog'"
            },
            "focus_areas": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional specific areas to focus on"
            }
        },
        "required": ["topic", "skill_id"]
    })
}

pub fn output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": {
                "type": "string",
                "description": "UUID to pass to create_blog_post"
            },
            "topic": {
                "type": "string",
                "description": "The researched topic"
            },
            "source_count": {
                "type": "integer",
                "description": "Number of sources found"
            }
        }
    })
}
```

### Schema Best Practices

| Practice | Description |
|----------|-------------|
| **Required fields** | Always specify `"required": [...]` |
| **Descriptions** | Every property needs a description |
| **Enums for constraints** | Use `"enum": ["a", "b"]` for valid values |
| **Default values** | Document defaults in description |
| **Array items** | Always specify `"items"` for arrays |

---

## Handler Signature Pattern

Handlers follow a consistent signature for dependency injection:

```rust
// tools/research_blog/handler.rs
use anyhow::Result;
use rmcp::model::{CallToolRequestParams, CallToolResult, Content};
use rmcp::ErrorData as McpError;
use serde_json::json;
use std::sync::Arc;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::AiService;
use systemprompt::database::DbPool;
use systemprompt::models::execution::context::RequestContext;

use super::helpers::extract_string_array;
use crate::server::ProgressCallback;

pub async fn handle(
    db_pool: &DbPool,
    request: CallToolRequestParams,
    ctx: RequestContext,
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    artifact_repo: &ArtifactRepository,
    progress: Option<ProgressCallback>,
) -> Result<CallToolResult, McpError> {
    // Implementation here
}
```

### Handler Structure

1. **Extract arguments** — Get and validate inputs
2. **Load dependencies** — Skills, context, etc.
3. **Report progress** — For long operations
4. **Execute logic** — Call services, not direct DB access
5. **Create artifact** — If storing results
6. **Return result** — Both text and structured content

---

## Parameter Extraction Pattern

Always validate parameters at the start:

```rust
// Extract required arguments
let args = request.arguments.as_ref().ok_or_else(|| {
    McpError::invalid_request("Missing arguments", None)
})?;

// Get required string parameter
let topic = args
    .get("topic")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: topic", None)
    })?;

// Validate enum values
let skill_id = args
    .get("skill_id")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: skill_id", None)
    })?;

if skill_id != "research_blog" {
    return Err(McpError::invalid_params(
        "skill_id must be 'research_blog'",
        None,
    ));
}

// Get optional array parameter
let focus_areas = extract_string_array(args, "focus_areas");
```

---

## Progress Callback Pattern

For long-running operations, report progress:

```rust
use std::future::Future;
use std::pin::Pin;

pub type ProgressCallback = Box<
    dyn Fn(f64, Option<f64>, Option<String>) -> Pin<Box<dyn Future<Output = ()> + Send>>
        + Send
        + Sync,
>;

// In handler:
pub async fn handle(
    // ... other params
    progress: Option<ProgressCallback>,
) -> Result<CallToolResult, McpError> {
    // Report progress (0-100 scale)
    if let Some(ref notify) = progress {
        notify(0.0, Some(100.0), Some("Starting...".to_string())).await;
    }

    // Do work...

    if let Some(ref notify) = progress {
        notify(50.0, Some(100.0), Some("Processing...".to_string())).await;
    }

    // More work...

    if let Some(ref notify) = progress {
        notify(100.0, Some(100.0), Some("Complete".to_string())).await;
    }

    Ok(result)
}
```

### Progress Callback Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `progress` | `f64` | Current progress value |
| `total` | `Option<f64>` | Total value (typically 100.0) |
| `message` | `Option<String>` | Human-readable status |

---

## Shared Utilities Pattern

Common helpers go in `tools/shared/mod.rs`:

```rust
// tools/shared/mod.rs
use serde_json::Map;

/// Extract string array from JSON arguments
pub fn extract_string_array(
    args: &Map<String, serde_json::Value>,
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

/// Extract optional string
pub fn extract_optional_string(
    args: &Map<String, serde_json::Value>,
    key: &str,
) -> Option<String> {
    args.get(key)
        .and_then(|v| v.as_str())
        .map(String::from)
}

/// Extract integer with default
pub fn extract_int_or_default(
    args: &Map<String, serde_json::Value>,
    key: &str,
    default: i64,
) -> i64 {
    args.get(key)
        .and_then(|v| v.as_i64())
        .unwrap_or(default)
}
```

---

## Response Pattern

Return both human-readable and structured content:

```rust
Ok(CallToolResult {
    // Human-readable text for display
    content: vec![Content::text(format!(
        "Research complete for '{topic}'.\n\n\
         Found {source_count} sources.\n\n\
         **Artifact ID: {artifact_id}**"
    ))],
    // Structured data for programmatic use
    structured_content: Some(json!({
        "artifact_id": artifact_id.to_string(),
        "topic": topic,
        "source_count": source_count,
        "status": "completed"
    })),
    is_error: Some(false),
    meta: None,
})
```

### Response Best Practices

| Field | Purpose |
|-------|---------|
| `content` | Human-readable, can include markdown |
| `structured_content` | Machine-readable JSON |
| `is_error` | `true` for errors, `false` for success |
| `meta` | Optional metadata (artifact IDs, etc.) |

---

## Error Handling Pattern

Use `McpError` for MCP protocol errors:

```rust
// Invalid parameters (client error)
Err(McpError::invalid_params(
    "Missing required parameter: topic",
    None,
))

// Invalid request
Err(McpError::invalid_request(
    "Arguments must be provided",
    None,
))

// Internal error (server error)
Err(McpError::internal_error(
    format!("Failed to save artifact: {e}"),
    None,
))

// Method not found
Err(McpError::method_not_found::<CallToolRequestMethod>())
```

---

## Skill Loading Pattern

Load skills from the database for prompts:

```rust
let skill_content = skill_loader
    .load_skill(skill_id, &ctx)
    .await
    .map_err(|e| {
        McpError::internal_error(
            format!("Failed to load skill '{skill_id}': {e}"),
            None,
        )
    })?;

// Use in AI messages
let messages = vec![
    AiMessage::system(&skill_content),
    AiMessage::user(&user_prompt),
];
```

---

## Checklist

- [ ] Tool in its own subdirectory
- [ ] `mod.rs` re-exports handler and schemas
- [ ] `helpers.rs` contains `input_schema()` and `output_schema()`
- [ ] `handler.rs` contains `handle()` function
- [ ] All required parameters validated
- [ ] Progress reported for long operations
- [ ] Both `content` and `structured_content` returned
- [ ] Errors use `McpError` types
- [ ] Shared utilities in `tools/shared/`

---

## Quick Reference

| File | Contains |
|------|----------|
| `tools/mod.rs` | `list_tools()`, `handle_tool_call()`, `create_tool()` |
| `tools/{name}/mod.rs` | `pub use handler::handle; pub use helpers::*;` |
| `tools/{name}/handler.rs` | `pub async fn handle(...) -> Result<CallToolResult, McpError>` |
| `tools/{name}/helpers.rs` | `input_schema()`, `output_schema()`, utilities |
| `tools/shared/mod.rs` | Common extraction helpers |

---

## Related Playbooks

- [MCP Tutorial](build_mcp-tutorial) — Building your first MCP server
- [MCP Artifacts](build_mcp-artifacts) — Creating and storing artifacts
- [MCP Checklist](build_mcp-checklist) — Full requirements checklist
- [Rust Standards](build_rust-standards) — Code quality guidelines
