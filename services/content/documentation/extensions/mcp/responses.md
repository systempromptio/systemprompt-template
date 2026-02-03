---
title: "MCP Response Patterns"
description: "Best practices for returning tool results with both human-readable and structured content."
author: "SystemPrompt Team"
slug: "extensions/mcp/responses"
keywords: "mcp, responses, content, structured, artifacts, results"
image: "/files/images/docs/mcp-responses.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Response Patterns

MCP tools return `CallToolResult` which contains both human-readable content and machine-readable structured data. This document covers best practices for constructing responses.

## CallToolResult Structure

```rust
pub struct CallToolResult {
    pub content: Vec<Content>,              // Human-readable output
    pub structured_content: Option<Value>,  // Machine-readable JSON
    pub is_error: Option<bool>,             // Success/failure flag
    pub meta: Option<Map<String, Value>>,   // Additional metadata
}
```

## The content Field

The `content` field contains human-readable output displayed to users. It's a vector of `Content` items.

### Text Content

Most common - plain text or markdown:

```rust
use rmcp::model::Content;

// Simple text
Content::text("Operation completed successfully.")

// Markdown formatting
Content::text(format!(
    "## Research Complete\n\n\
     **Topic:** {topic}\n\n\
     **Sources Found:** {count}\n\n\
     Use artifact ID `{artifact_id}` for the next step."
))

// Multi-paragraph
Content::text(format!(
    "Created blog post: {title}\n\n\
     Word count: {word_count}\n\n\
     The content has been saved. You can now generate social media posts."
))
```

### Multiple Content Items

Return multiple content blocks:

```rust
Ok(CallToolResult {
    content: vec![
        Content::text("## Summary\n\nOperation completed."),
        Content::text(format!("### Details\n\n{details}")),
    ],
    // ...
})
```

### Image Content

For tools that generate images:

```rust
Content::image(
    base64_encoded_data,
    "image/png",
)
```

## The structured_content Field

Machine-readable JSON for programmatic consumption. Include data that clients might parse.

### When to Include

| Scenario | Include structured_content? |
|----------|----------------------------|
| Tool creates an artifact | Yes - include artifact_id |
| Tool returns data clients parse | Yes |
| Tool performs action with no data | Optional |
| Error response | Optional - can include error details |

### Basic Structure

```rust
use serde_json::json;

Ok(CallToolResult {
    content: vec![Content::text("...")],
    structured_content: Some(json!({
        "artifact_id": artifact_id.to_string(),
        "topic": topic,
        "source_count": sources.len(),
        "status": "completed"
    })),
    is_error: Some(false),
    meta: None,
})
```

### Artifact Reference

When a tool creates an artifact:

```rust
structured_content: Some(json!({
    "artifact_id": artifact_id.to_string(),
    "topic": topic,
    "status": "completed",
    "next_step": "Call create_blog_post with this artifact_id"
}))
```

### Rich Data

Include data that clients can display or process:

```rust
structured_content: Some(json!({
    "artifact_id": artifact_id.to_string(),
    "topic": topic,
    "source_count": sources.len(),
    "query_count": queries.len(),
    "research_summary": summary,
    "sources": sources.iter().map(|s| json!({
        "title": s.title,
        "uri": s.uri,
        "relevance": s.relevance
    })).collect::<Vec<_>>(),
    "status": "completed"
}))
```

### Artifact Type Header

Use `x-artifact-type` for typed artifacts:

```rust
structured_content: Some(json!({
    "x-artifact-type": "blog_artifact",
    "skill_id": skill_id,
    "artifact": {
        "title": title,
        "slug": slug,
        "content": content,
        "word_count": word_count
    }
}))
```

## The is_error Field

Always set explicitly:

```rust
// Success
is_error: Some(false)

// Error
is_error: Some(true)
```

## The meta Field

Optional metadata for cross-references:

```rust
pub fn create_result_meta(artifact_id: &str) -> Map<String, Value> {
    let mut meta = serde_json::Map::new();
    meta.insert("artifact_id".to_string(), json!(artifact_id));
    meta.insert(
        "ui_uri".to_string(),
        json!(format!("ui://my-server/{artifact_id}"))
    );
    meta
}

// Usage
Ok(CallToolResult {
    content: vec![Content::text("...")],
    structured_content: Some(json!({...})),
    is_error: Some(false),
    meta: Some(create_result_meta(artifact_id.as_str())),
})
```

## Success Response Patterns

### Simple Success

```rust
Ok(CallToolResult {
    content: vec![Content::text("Task completed successfully.")],
    structured_content: Some(json!({"status": "completed"})),
    is_error: Some(false),
    meta: None,
})
```

### With Artifact

```rust
Ok(CallToolResult {
    content: vec![Content::text(format!(
        "## Research Complete\n\n\
         **Topic:** {topic}\n\n\
         Found {source_count} sources.\n\n\
         **Artifact ID:** `{artifact_id}`\n\n\
         Use this artifact_id when calling create_blog_post."
    ))],
    structured_content: Some(json!({
        "artifact_id": artifact_id.to_string(),
        "topic": topic,
        "source_count": source_count,
        "status": "completed"
    })),
    is_error: Some(false),
    meta: Some(create_result_meta(artifact_id.as_str())),
})
```

### With Rich Data

```rust
Ok(CallToolResult {
    content: vec![Content::text(format!(
        "## Blog Post Created\n\n\
         **Title:** {title}\n\n\
         **Slug:** {slug}\n\n\
         **Word Count:** {word_count}\n\n\
         Content ID: `{content_id}`"
    ))],
    structured_content: Some(json!({
        "x-artifact-type": "blog_artifact",
        "content_id": content_id.to_string(),
        "title": title,
        "slug": slug,
        "word_count": word_count,
        "skill_id": skill_id,
        "status": "completed"
    })),
    is_error: Some(false),
    meta: None,
})
```

## Error Response Patterns

### Parameter Error

```rust
return Err(McpError::invalid_params(
    "Missing required parameter: topic",
    None,
));
```

### Validation Error

```rust
if skill_id != "research_blog" {
    return Err(McpError::invalid_params(
        format!("skill_id must be 'research_blog', got '{skill_id}'"),
        None,
    ));
}
```

### Operation Error (Partial Result)

When an operation fails but there's useful context:

```rust
Ok(CallToolResult {
    content: vec![Content::text(format!(
        "## Error\n\n\
         Failed to complete research: {error}\n\n\
         Partial results may be available."
    ))],
    structured_content: Some(json!({
        "status": "error",
        "error": error.to_string(),
        "partial_results": partial_data
    })),
    is_error: Some(true),
    meta: None,
})
```

### Command Failure

For CLI-style tools:

```rust
if !output.success {
    return Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Command failed (exit code {}):\n\n```\n{}\n```",
            output.exit_code, output.stderr
        ))],
        structured_content: Some(json!({
            "status": "error",
            "exit_code": output.exit_code,
            "stderr": output.stderr
        })),
        is_error: Some(true),
        meta: None,
    });
}
```

## Best Practices

### 1. Always Set is_error

```rust
// Don't
meta: None,
// Do
is_error: Some(false),  // or Some(true)
meta: None,
```

### 2. Include Next Steps

Guide the user on what to do next:

```rust
Content::text(format!(
    "Research complete. Artifact ID: `{artifact_id}`\n\n\
     **Next step:** Call `create_blog_post` with:\n\
     - `artifact_id`: `{artifact_id}`\n\
     - `skill_id`: `blog_writing` or `technical_content_writing`\n\
     - `slug`: your-post-slug\n\
     - `description`: SEO description\n\
     - `keywords`: [\"keyword1\", \"keyword2\"]\n\
     - `instructions`: specific writing instructions"
))
```

### 3. Format IDs Consistently

Use backticks for IDs in text:

```rust
// Good
format!("Artifact ID: `{artifact_id}`")

// Also good for emphasis
format!("**Artifact ID:** `{artifact_id}`")
```

### 4. Match Schema to Output

Your output should match `output_schema()`:

```rust
// In helpers.rs
pub fn output_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "artifact_id": {"type": "string"},
            "status": {"type": "string", "enum": ["completed", "error"]}
        },
        "required": ["artifact_id", "status"]
    })
}

// In handler.rs - ensure response matches
structured_content: Some(json!({
    "artifact_id": artifact_id.to_string(),  // Required
    "status": "completed"                     // Required, valid enum
}))
```

### 5. Log Completion

Log successful operations:

```rust
tracing::info!(
    topic = %topic,
    artifact_id = %artifact_id,
    source_count = %source_count,
    "Research completed"
);

Ok(CallToolResult { /* ... */ })
```

## Quick Reference

| Field | Type | Purpose |
|-------|------|---------|
| `content` | `Vec<Content>` | Human-readable output |
| `structured_content` | `Option<Value>` | Machine-readable JSON |
| `is_error` | `Option<bool>` | Success/failure flag |
| `meta` | `Option<Map>` | Artifact references, URIs |

| Content Type | Use For |
|--------------|---------|
| `Content::text()` | Text/markdown output |
| `Content::image()` | Image data (base64) |

| Error Type | Use For |
|------------|---------|
| `McpError::invalid_params()` | Bad parameter values |
| `McpError::invalid_request()` | Malformed request |
| `McpError::internal_error()` | Server-side failures |
| `McpError::method_not_found()` | Unknown tool |