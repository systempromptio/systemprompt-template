# Agentic Execution Flow

The agentic execution system follows a three-stage model: **PLAN → EXECUTE → RESPOND**.

---

## Stages Overview

| Stage | Type | AI Calls | Description |
|-------|------|----------|-------------|
| **PLAN** | AI | 1 | Analyze request, output tool calls or direct response |
| **EXECUTE** | Mechanical | 0 | Run tools sequentially, resolve templates |
| **RESPOND** | AI | 1 | Generate response with full agent context |

---

## Stage Details

### PLAN Stage

The AI analyzes the user's request and decides how to respond.

**Input to AI:**
- Full message history (skills, agent system prompt, conversation history, current message)
- Single callable tool: `__planning__` with dynamically generated schema

**Output Options:**

1. **Direct Response** - No tools needed
```json
{
  "type": "direct_response",
  "content": "Hello! How can I help you today?"
}
```

2. **Tool Calls** - Tools needed (with template references)
```json
{
  "type": "tool_calls",
  "reasoning": "User wants a blog post. I need to research the topic first, then create the blog.",
  "calls": [
    {"tool_name": "research_blog", "arguments": {"topic": "AI trends 2025", "skill_id": "research_blog"}},
    {"tool_name": "create_blog_post", "arguments": {
      "artifact_id": "$0.output.artifact_id",
      "skill_id": "blog_writing",
      "instructions": "Write about emerging trends"
    }}
  ]
}
```

---

### EXECUTE Stage

Mechanical execution with template resolution. No AI involved.

**Process:**
1. Validate all template references against output schemas (fail fast)
2. Iterate through `calls` array in order
3. Resolve templates using prior results
4. Execute each tool with resolved arguments
5. Collect results
6. On failure: halt and proceed to RESPOND with partial results

---

### RESPOND Stage

The AI generates a final response using full agent context and tool execution results.

---

## Template System

Templates allow tools in a plan to reference outputs from earlier tools.

### Syntax

```
$<tool_index>.output.<field_path>
```

| Example | Meaning |
|---------|---------|
| `$0.output.artifact_id` | First tool's `artifact_id` field |
| `$1.output.sources` | Second tool's `sources` array |
| `$0.output.metadata.title` | Nested field access |

### Validation

**All template validation happens at PLAN time, before execution.**

| Rule | Check |
|------|-------|
| Index bounds | `$N` where N < number of preceding tools |
| Field existence | Referenced field exists in tool's `outputSchema` |
| Type compatibility | Referenced field type matches target argument type |
| No circular refs | Tool cannot reference itself or later tools |

### Validation Flow

```
AI produces plan
       │
       ▼
┌─────────────────┐
│ Parse templates │
└────────┬────────┘
         │
         ▼
┌─────────────────────────┐
│ For each template ref:  │
│  - Check tool index     │
│  - Check field in schema│
│  - Check type matches   │
└────────┬────────────────┘
         │
    ┌────┴────┐
    │         │
  Valid    Invalid
    │         │
    ▼         ▼
 EXECUTE   Return validation
            error to synthesis
```

### Error Handling

If validation fails, skip EXECUTE and go directly to synthesis:

```rust
PlanValidationError {
    tool_index: 1,
    argument: "artifact_id",
    template: "$0.output.artifact_id",
    error: FieldNotFound {
        tool: "research_blog",
        field: "artifact_id",
        available_fields: ["topic", "sources"]
    }
}
```

---

## MCP Tool Response Structure

### `content` vs `structuredContent`

MCP tool responses have two output mechanisms:

| Field | Purpose | Type | Consumer |
|-------|---------|------|----------|
| `content` | Human/LLM readable | `Vec<Content>` (text, images) | LLM, backwards compat |
| `structuredContent` | Machine readable | JSON object | Templates, programmatic access |

**Key rule:** A tool returning structured content SHOULD also return serialized JSON in a TextContent block for backwards compatibility.

### Relationship to `outputSchema`

```
Tool Definition                    Tool Response
─────────────────                  ─────────────────
inputSchema  ──────────────────►   arguments (validated against inputSchema)
outputSchema ──────────────────►   structuredContent (validated against outputSchema)
                                   content (human-readable duplicate)
```

**The `outputSchema` defines what `structuredContent` must contain.**
**Templates reference fields from `structuredContent` via `$N.output.field`.**

---

## Standard Response Shape

All tools that produce artifacts MUST return a consistent `structuredContent` shape:

```json
{
  "artifact_id": "uuid",
  "artifact": {
    // Tool-specific artifact data (presentation card, blog post, etc.)
  },
  "_metadata": {
    "context_id": "uuid",
    "request_id": "uuid",
    "trace_id": "uuid",
    "user_id": "string",
    "tool_name": "string",
    "executed_at": "ISO8601"
  }
}
```

### Field Descriptions

| Field | Type | Description |
|-------|------|-------------|
| `artifact_id` | string | Unique identifier for the created artifact |
| `artifact` | object | Tool-specific output (card, blog, social post, etc.) |
| `_metadata.context_id` | string | Conversation context for DB lookup |
| `_metadata.request_id` | string | Request ID for tracing |
| `_metadata.trace_id` | string | Distributed trace ID |
| `_metadata.user_id` | string | User who initiated the request |
| `_metadata.tool_name` | string | Name of the tool that produced this |
| `_metadata.executed_at` | string | ISO8601 timestamp |

### Standard Output Schema

Tools producing artifacts should declare this output schema:

```json
{
  "type": "object",
  "properties": {
    "artifact_id": {
      "type": "string",
      "description": "Unique identifier for the artifact"
    },
    "artifact": {
      "type": "object",
      "description": "Tool-specific artifact data"
    },
    "_metadata": {
      "type": "object",
      "properties": {
        "context_id": {"type": "string"},
        "request_id": {"type": "string"},
        "trace_id": {"type": "string"},
        "user_id": {"type": "string"},
        "tool_name": {"type": "string"},
        "executed_at": {"type": "string", "format": "date-time"}
      }
    }
  },
  "required": ["artifact_id", "artifact", "_metadata"]
}
```

---

## Example: Research → Create Blog Flow

### Tool Definitions

**research_blog outputSchema:**
```json
{
  "properties": {
    "artifact_id": {"type": "string"},
    "artifact": {
      "type": "object",
      "properties": {
        "title": {"type": "string"},
        "sections": {"type": "array"},
        "sources": {"type": "array"}
      }
    },
    "_metadata": {"type": "object"}
  },
  "required": ["artifact_id", "artifact", "_metadata"]
}
```

**create_blog_post inputSchema:**
```json
{
  "properties": {
    "artifact_id": {
      "type": "string",
      "description": "Research artifact ID from research_blog"
    },
    "skill_id": {"type": "string"},
    "instructions": {"type": "string"}
  },
  "required": ["artifact_id", "skill_id", "instructions"]
}
```

### Plan

```json
{
  "type": "tool_calls",
  "reasoning": "Research the topic, then use that research to create the blog post.",
  "calls": [
    {
      "tool_name": "research_blog",
      "arguments": {"topic": "AI trends 2025", "skill_id": "research_blog"}
    },
    {
      "tool_name": "create_blog_post",
      "arguments": {
        "artifact_id": "$0.output.artifact_id",
        "skill_id": "blog_writing",
        "instructions": "Focus on practical applications"
      }
    }
  ]
}
```

### Validation

1. `$0.output.artifact_id` → tool 0 is `research_blog`
2. Check `research_blog.outputSchema.properties.artifact_id` exists ✓
3. Type is `string`, matches `create_blog_post.inputSchema.properties.artifact_id` ✓

**Result:** Valid → proceed to EXECUTE

### Execution

**Step 1:** Execute `research_blog`
```json
{
  "artifact_id": "abc-123",
  "artifact": {
    "title": "AI Trends 2025",
    "sections": [...],
    "sources": [...]
  },
  "_metadata": {
    "context_id": "ctx-456",
    "request_id": "req-789",
    "trace_id": "trace-012",
    "user_id": "user-345",
    "tool_name": "research_blog",
    "executed_at": "2025-01-15T10:30:00Z"
  }
}
```

**Step 2:** Resolve templates for `create_blog_post`
- `$0.output.artifact_id` → `"abc-123"`

**Step 3:** Execute `create_blog_post` with resolved arguments
```json
{
  "artifact_id": "abc-123",
  "skill_id": "blog_writing",
  "instructions": "Focus on practical applications"
}
```

---

## AI Call Summary

| Scenario | Stages Used | AI Calls |
|----------|-------------|----------|
| Simple response (no tools) | PLAN | **1** |
| With tools (success) | PLAN → EXECUTE → RESPOND | **2** |
| With tools (failure) | PLAN → EXECUTE → RESPOND | **2** |
| Template validation failure | PLAN → RESPOND | **2** |

---

## Code References

| Component | Location |
|-----------|----------|
| Planning tool generation | `core/crates/modules/ai/src/services/execution_control.rs` |
| Data structures | `core/crates/shared/models/src/ai/execution_plan.rs` |
| Plan executor | `core/crates/modules/agent/src/services/a2a_server/processing/strategies/plan_executor.rs` |
| Strategy | `core/crates/modules/agent/src/services/a2a_server/processing/strategies/planned.rs` |
| AI service methods | `core/crates/modules/ai/src/services/core/ai_service.rs` |
| Artifact transformer | `core/crates/modules/agent/src/services/mcp/artifact_transformer.rs` |

---

## Skill ID Extraction

The `extract_skill_id()` function extracts skill identifiers from artifact JSON in this order:

1. **Top-level `skill_id`** (preferred): `{"skill_id": "research_blog", ...}`
2. **Nested in `_metadata`** (fallback): `{"_metadata": {"skill_id": "research_blog"}}`

Artifacts should use top-level `skill_id` fields. The `_metadata` fallback exists for backwards compatibility with the `ToolResponse` wrapper format.

### ToolResponse Format

```json
{
  "artifact_id": "uuid",
  "mcp_execution_id": "uuid",
  "artifact": {
    "skill_id": "research_blog",
    "title": "...",
    "content": "..."
  },
  "_metadata": {
    "context_id": "...",
    "trace_id": "..."
  }
}
```

The transformer auto-detects the format by checking for the presence of `artifact_id`, `artifact`, and `_metadata` fields.

---

## MCP Tool Compliance Checklist

When creating or updating MCP tools:

- [ ] Define artifact struct with `#[derive(Serialize, Deserialize, JsonSchema)]`
- [ ] Use `ToolResponse::<MyArtifact>::schema()` for type-safe output schema
- [ ] Use `ToolResponse::new(artifact_id, artifact, metadata).to_json()` for structured content
- [ ] Populate metadata with `ExecutionMetadata::with_request(&ctx).tool("tool_name")`
- [ ] Return human-readable summary in `content` field
- [ ] Document which fields can be referenced by downstream tools

---

## Implementation Pattern

### 1. Define Artifact Struct (with schemars)

```rust
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchArtifact {
    pub title: String,
    pub sections: Vec<ResearchSection>,
    pub source_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchSection {
    pub heading: String,
    pub content: String,
}
```

### 2. Generate Output Schema (type-safe)

```rust
use systemprompt_models::artifacts::ToolResponse;

pub fn research_blog_output_schema() -> serde_json::Value {
    ToolResponse::<ResearchArtifact>::schema()
}
```

### 3. Return Structured Response

```rust
use systemprompt_models::artifacts::{ExecutionMetadata, ToolResponse};

// In handler:
let metadata = ExecutionMetadata::with_request(&ctx)
    .tool("research_blog")
    .skill(&skill_id, "Research");

let artifact = ResearchArtifact {
    title: "AI Trends 2025".to_string(),
    sections: vec![...],
    source_count: 15,
};

let response = ToolResponse::new(&artifact_id, artifact, metadata);

Ok(CallToolResult {
    content: vec![Content::text(format!(
        "Research complete. Artifact ID: {}", artifact_id
    ))],
    structured_content: Some(response.to_json()),
    is_error: Some(false),
    meta: None,
})
```

### 4. Reference in Downstream Tools

With the above pattern, downstream tools can reference:
- `$0.output.artifact_id` → The artifact ID for chaining
- `$0.output.artifact.title` → Nested artifact fields
- `$0.output._metadata.context_id` → Execution context for DB lookup
