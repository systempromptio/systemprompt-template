---
title: "MCP Tool Structure"
description: "Detailed reference for organizing tools with modular directory patterns, handler signatures, and schema definitions."
author: "SystemPrompt Team"
slug: "extensions/mcp/tool-structure"
keywords: "mcp, tools, structure, handlers, schemas, patterns"
image: "/files/images/docs/mcp-tools.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Tool Structure

This document provides detailed reference material for organizing and implementing MCP tools. The patterns here are extracted from production MCP servers in the SystemPrompt codebase.

## Why Modular Structure?

MCP tools can grow complex quickly. A modular directory structure provides:

- **Separation of concerns** — Schemas, handlers, and utilities in separate files
- **Testability** — Each handler can be unit tested in isolation
- **Maintainability** — Changes are localized to a single directory
- **Discoverability** — Tool name matches directory name

## Directory Layout

```
extensions/mcp/{server}/src/tools/
├── mod.rs                  # Registration and dispatch
├── shared/                 # Shared utilities
│   └── mod.rs
└── {tool_name}/            # Each tool
    ├── mod.rs              # Re-exports
    ├── handler.rs          # Implementation
    └── helpers.rs          # Schemas and utilities
```

### Root mod.rs

The root `mod.rs` file handles two responsibilities:

1. **Tool registration** — Returns the list of available tools
2. **Tool dispatch** — Routes calls to the correct handler

```rust
use rmcp::model::{CallToolRequestParams, CallToolResult, Tool};
use rmcp::ErrorData as McpError;
use std::sync::Arc;

// Import tool modules
pub mod research_blog;
pub mod create_blog_post;
pub mod shared;

// Re-export for external use
pub use research_blog::handle as handle_research_blog;

/// Returns all tools with their schemas
pub fn list_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "research_blog",
            "Research Blog Topic",
            "Research a topic using Google Search.",
            research_blog::input_schema(),
            research_blog::output_schema(),
        ),
    ]
}

/// Routes tool calls to handlers
pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParams,
    // ... service dependencies
) -> Result<CallToolResult, McpError> {
    match name {
        "research_blog" => research_blog::handle(/* args */).await,
        _ => Err(McpError::invalid_params(
            format!("Unknown tool: '{name}'"),
            None,
        )),
    }
}
```

### Tool mod.rs

Each tool's `mod.rs` is minimal, just re-exporting:

```rust
mod handler;
mod helpers;

pub use handler::handle;
pub use helpers::{input_schema, output_schema};
```

## Schema Definitions

Schemas use JSON Schema format and live in `helpers.rs`:

### Input Schema

```rust
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
                "description": "Must be 'research_blog'",
                "enum": ["research_blog"]
            },
            "focus_areas": {
                "type": "array",
                "items": {"type": "string"},
                "description": "Optional areas to focus on"
            },
            "limit": {
                "type": "integer",
                "description": "Maximum results (default: 10)",
                "default": 10,
                "minimum": 1,
                "maximum": 100
            }
        },
        "required": ["topic", "skill_id"]
    })
}
```

### Output Schema

```rust
pub fn output_schema() -> serde_json::Value {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": {
                "type": "string",
                "format": "uuid",
                "description": "UUID to pass to subsequent tools"
            },
            "topic": {
                "type": "string"
            },
            "source_count": {
                "type": "integer"
            },
            "status": {
                "type": "string",
                "enum": ["completed", "partial", "error"]
            }
        },
        "required": ["artifact_id", "status"]
    })
}
```

### Schema Best Practices

| Practice | Example |
|----------|---------|
| Always specify `required` | `"required": ["topic", "skill_id"]` |
| Add descriptions to all properties | `"description": "The topic to research"` |
| Use `enum` for constrained values | `"enum": ["option_a", "option_b"]` |
| Document defaults | `"default": 10` or in description |
| Specify array item types | `"items": {"type": "string"}` |
| Use format hints | `"format": "uuid"`, `"format": "uri"` |
| Set bounds for numbers | `"minimum": 1, "maximum": 100` |

## Handler Implementation

Handlers live in `handler.rs` and follow a consistent pattern:

### Handler Signature

```rust
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
    // Implementation
}
```

### Handler Structure

A well-structured handler follows this flow:

```rust
pub async fn handle(/* params */) -> Result<CallToolResult, McpError> {
    // 1. Report initial progress
    if let Some(ref notify) = progress {
        notify(0.0, Some(100.0), Some("Starting...".to_string())).await;
    }

    // 2. Extract and validate arguments
    let args = request.arguments.as_ref().ok_or_else(|| {
        McpError::invalid_request("Missing arguments", None)
    })?;

    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::invalid_params("Missing required parameter: topic", None)
        })?;

    // 3. Load dependencies (skills, etc.)
    let skill_content = skill_loader
        .load_skill("research_blog", &ctx)
        .await
        .map_err(|e| McpError::internal_error(format!("Skill error: {e}"), None))?;

    // 4. Execute business logic
    if let Some(ref notify) = progress {
        notify(30.0, Some(100.0), Some("Processing...".to_string())).await;
    }

    let result = do_work(topic, &skill_content).await?;

    // 5. Create artifact (if applicable)
    if let Some(ref notify) = progress {
        notify(80.0, Some(100.0), Some("Saving artifact...".to_string())).await;
    }

    let artifact_id = create_and_store_artifact(artifact_repo, &result).await?;

    // 6. Return result
    if let Some(ref notify) = progress {
        notify(100.0, Some(100.0), Some("Complete".to_string())).await;
    }

    Ok(CallToolResult {
        content: vec![Content::text(format!("Done. Artifact: {artifact_id}"))],
        structured_content: Some(json!({
            "artifact_id": artifact_id,
            "status": "completed"
        })),
        is_error: Some(false),
        meta: None,
    })
}
```

## Parameter Extraction

### Required Parameters

```rust
let args = request.arguments.as_ref().ok_or_else(|| {
    McpError::invalid_request("Missing arguments", None)
})?;

// String parameter
let topic = args
    .get("topic")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: topic", None)
    })?;

// Integer parameter
let limit = args
    .get("limit")
    .and_then(|v| v.as_i64())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: limit", None)
    })? as usize;
```

### Optional Parameters

```rust
// Optional string with default
let format = args
    .get("format")
    .and_then(|v| v.as_str())
    .unwrap_or("markdown");

// Optional integer with default
let limit = args
    .get("limit")
    .and_then(|v| v.as_i64())
    .unwrap_or(10) as usize;
```

### Array Parameters

Use a helper function:

```rust
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

// Usage
let focus_areas = extract_string_array(args, "focus_areas");
```

### Enum Validation

```rust
let skill_id = args
    .get("skill_id")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: skill_id", None)
    })?;

if skill_id != "research_blog" {
    return Err(McpError::invalid_params(
        format!("Invalid skill_id: '{skill_id}'. Must be 'research_blog'."),
        None,
    ));
}
```

## Shared Utilities

Common helpers go in `tools/shared/mod.rs`:

```rust
use serde_json::{Map, Value};

/// Extract string array from arguments
pub fn extract_string_array(args: &Map<String, Value>, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(|v| v.as_array())
        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
        .unwrap_or_default()
}

/// Extract optional string
pub fn extract_optional_string(args: &Map<String, Value>, key: &str) -> Option<String> {
    args.get(key).and_then(|v| v.as_str()).map(String::from)
}

/// Extract integer with default
pub fn extract_int_or_default(args: &Map<String, Value>, key: &str, default: i64) -> i64 {
    args.get(key).and_then(|v| v.as_i64()).unwrap_or(default)
}

/// Extract boolean with default
pub fn extract_bool_or_default(args: &Map<String, Value>, key: &str, default: bool) -> bool {
    args.get(key).and_then(|v| v.as_bool()).unwrap_or(default)
}
```

## Error Handling

Use appropriate `McpError` types:

```rust
// Client provided invalid parameters
Err(McpError::invalid_params("Missing required parameter: topic", None))

// Request structure is invalid
Err(McpError::invalid_request("Arguments must be provided", None))

// Server-side error
Err(McpError::internal_error(format!("Database error: {e}"), None))

// Tool not found
Err(McpError::method_not_found::<CallToolRequestMethod>())
```

## Testing Tools

### Unit Test Handler

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handle_valid_input() {
        let args = serde_json::json!({
            "topic": "Rust async",
            "skill_id": "research_blog"
        });

        let request = CallToolRequestParams {
            name: "research_blog".into(),
            arguments: Some(args.as_object().unwrap().clone()),
            meta: None,
        };

        // Mock dependencies and call handler
        let result = handle(/* mocked deps */, request, /* ... */).await;

        assert!(result.is_ok());
        let result = result.unwrap();
        assert_eq!(result.is_error, Some(false));
    }
}
```

## File Reference

| File | Purpose |
|------|---------|
| `tools/mod.rs` | `list_tools()`, `handle_tool_call()` |
| `tools/{name}/mod.rs` | Re-exports handler and schemas |
| `tools/{name}/handler.rs` | `pub async fn handle()` |
| `tools/{name}/helpers.rs` | `input_schema()`, `output_schema()`, utilities |
| `tools/shared/mod.rs` | Common extraction helpers |