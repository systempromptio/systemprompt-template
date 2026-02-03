---
title: "MCP Artifacts and Resources"
description: "Production patterns for creating artifacts, storing results, and exposing UI resources."
keywords:
  - mcp
  - artifacts
  - resources
  - ui
  - mcp-app
category: build
playbook_references:
  - id: "build_extension-checklist"
    description: "Adding new artifact types to core"
  - id: "build_mcp-checklist"
    description: "MCP server requirements"
---

# MCP Artifacts and Resources

> **Help**: `{ "command": "core playbooks show build_mcp-artifacts" }`

**Patterns for creating artifacts, storing tool results, and exposing UI resources.**

**Prerequisites:**
- [ ] Read [MCP Tutorial](build_mcp-tutorial)
- [ ] Read [MCP Tool Patterns](build_mcp-tools)

**Reference Implementations:**
- `extensions/mcp/content-manager/src/tools/research_blog/` — Artifact creation
- `extensions/mcp/systemprompt/src/server.rs` — UI resources
- `extensions/mcp/systemprompt/src/artifacts.rs` — Artifact rendering
- `systemprompt-core/crates/domain/mcp/src/services/ui_renderer/` — MCP UI renderers

---

## MCP UI: Artifact → HTML Rendering

**Artifacts are converted to MCP UI assets** — interactive HTML components that can be displayed in MCP-compatible clients.

### What is MCP UI?

MCP UI is a standard for rendering tool outputs as interactive HTML. When an MCP tool returns an artifact:

1. **Artifact stored** with type (e.g., `text`, `table`, `chart`)
2. **UI Renderer** converts artifact data → HTML
3. **MCP App** returned with MIME type `text/html;profile=mcp-app`
4. **Client displays** the interactive HTML component

### The Rendering Pipeline

```
Artifact (persisted)
    │
    ▼
UiRendererRegistry.render(artifact)
    │
    ▼
Type-specific renderer (TextRenderer, TableRenderer, etc.)
    │
    ▼
UiResource { html, csp_policy }
    │
    ▼
MCP Resource Response (MIME: text/html;profile=mcp-app)
```

### Available UI Renderers

Core provides renderers for each artifact type:

| Artifact Type | Renderer | Output |
|--------------|----------|--------|
| `text` | `TextRenderer` | Formatted text with copy button |
| `table` | `TableRenderer` | Interactive sortable table |
| `chart` | `ChartRenderer` | Chart.js visualization |
| `list` | `ListRenderer` | Ordered/unordered lists |
| `image` | `ImageRenderer` | Image display with zoom |
| `form` | `FormRenderer` | Interactive form |
| `dashboard` | `DashboardRenderer` | Multi-section dashboard |

### Resource URI Pattern

Artifacts are exposed as MCP resources via URI:

```
ui://{server_name}/{artifact_id}
```

Example: `ui://content-manager/550e8400-e29b-41d4-a716-446655440000`

### UiMetadata in Tool Responses

Include UI metadata in tool responses to enable client rendering:

```rust
use systemprompt::mcp::services::ui_renderer::UiMetadata;

// In tool response meta
let ui_meta = UiMetadata::for_artifact(artifact_id.as_str(), Some("my-server"));
meta.insert("ui".to_string(), ui_meta.to_json());
```

This adds to the response:
```json
{
  "ui": {
    "resourceUri": "ui://my-server/{artifact_id}",
    "visibility": ["model"]
  }
}
```

---

## Adding New Artifact Types

**If you need a new artifact type, add it to systemprompt-core.**

See [Extension Checklist](build_extension-checklist) for the full process. Summary:

1. **Add artifact struct** in `systemprompt-core/crates/shared/models/src/artifacts/`
2. **Add to ArtifactType enum** in `types.rs`
3. **Export from mod.rs**
4. **Create UI renderer** in `systemprompt-core/crates/domain/mcp/src/services/ui_renderer/templates/`
5. **Register in default registry**

### Example: Adding a new artifact type

```rust
// 1. Define artifact struct (in core)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MyNewArtifact {
    #[serde(rename = "x-artifact-type")]
    pub artifact_type: String,
    pub content: String,
    // ... fields
}

// 2. Create renderer (in core)
pub struct MyNewRenderer;

#[async_trait]
impl UiRenderer for MyNewRenderer {
    fn artifact_type(&self) -> ArtifactType {
        ArtifactType::Custom("my_new".to_string())
    }

    async fn render(&self, artifact: &Artifact) -> Result<UiResource> {
        // Convert artifact.parts to HTML
        let html = format!("<div>...</div>");
        Ok(UiResource::new(html))
    }
}

// 3. Register in create_default_registry()
registry.register(MyNewRenderer::new());
```

**DO NOT create custom artifact types in MCP servers.** They won't have UI renderers and won't be properly supported.

---

## Artifact Pipeline Overview

**CRITICAL**: MCP tools do NOT save artifacts directly. The agent handles persistence.

```
MCP Tool Handler
    │
    ▼
Return structured_content (artifact data)
    │
    ▼
Agent receives response
    │
    ▼
ArtifactBuilder transforms structured_content → Artifact
    │
    ▼
ArtifactPublishingService.publish_from_a2a() saves to DB
    │
    ▼
UI Resource (optional)
    │
    ▼
HTML Rendering
```

### Why This Architecture?

| Concern | Reason |
|---------|--------|
| **Task ownership** | Agent creates tasks in DB, owns valid task_ids |
| **FK integrity** | Agent's task_id exists in DB, MCP-generated IDs don't |
| **Separation** | MCP = business logic, Agent = persistence orchestration |

---

## Creating Artifacts (Correct Pattern)

MCP tools return artifact data in `structured_content`. The **agent** handles persistence.

```rust
use systemprompt::identifiers::{ArtifactId, McpExecutionId};
use systemprompt::models::artifacts::{ResearchArtifact, SourceCitation, ToolResponse};
use systemprompt::models::ExecutionMetadata;

pub async fn handle(
    ctx: RequestContext,
    ai_service: &Arc<AiService>,
    mcp_execution_id: &McpExecutionId,
    // ... other params (NO artifact_repo for writing!)
) -> Result<CallToolResult, McpError> {
    // Generate artifact_id (UUID) - this ID will be preserved by the agent
    let artifact_id = uuid::Uuid::new_v4().to_string();

    // Execute business logic (AI calls, etc.)
    let research_result = ai_service.generate_with_google_search(...).await?;

    // Build typed artifact for response
    let sources: Vec<SourceCitation> = research_result.sources
        .iter()
        .map(|s| SourceCitation::new(&s.title, &s.uri, s.relevance))
        .collect();

    let artifact = ResearchArtifact::new(topic, card, sources)
        .with_query_count(query_count as u32);

    // Build metadata from request context
    let metadata = ExecutionMetadata::with_request(&ctx)
        .with_tool("research_blog")
        .with_skill(skill_id, "Blog Research");

    // Create typed response (agent will transform and persist this)
    let response = ToolResponse::new(
        ArtifactId::new(&artifact_id),
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    // Return artifact in structured_content - DO NOT save to database!
    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Research complete. **Artifact ID: {artifact_id}**"
        ))],
        structured_content: response.to_json().ok(),  // Agent persists this
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
```

### What Happens After Return

1. Agent receives `CallToolResult` with `structured_content`
2. `ArtifactBuilder.build_artifacts()` transforms JSON → `Artifact`
3. Agent's `task_id` (valid FK) is attached to artifact
4. `ArtifactPublishingService.publish_from_a2a()` persists to database

### Anti-Pattern (DO NOT DO)

```rust
// WRONG - MCP tools should NOT save artifacts directly
let task_id = TaskId::generate();  // This ID doesn't exist in DB!
artifact_repo
    .create_artifact(&task_id, &context_id, &artifact)  // FK violation!
    .await?;
```

---

## Artifact Structure

### Artifact Fields

| Field | Type | Description |
|-------|------|-------------|
| `id` | `ArtifactId` | Unique identifier (UUID) |
| `name` | `Option<String>` | Human-readable name |
| `description` | `Option<String>` | Description of contents |
| `parts` | `Vec<Part>` | Data parts (usually DataPart) |
| `metadata` | `ArtifactMetadata` | Type, context, task, tool info |
| `extensions` | `Vec<Value>` | Extension URIs for rendering |

### ArtifactMetadata Fields

| Field | Type | Description |
|-------|------|-------------|
| `artifact_type` | `String` | Type identifier (e.g., "research", "blog") |
| `context_id` | `ContextId` | Conversation context |
| `task_id` | `TaskId` | Task that created this |
| `tool_name` | `Option<String>` | Tool that created this |
| `skill_id` | `Option<String>` | Skill used |
| `skill_name` | `Option<String>` | Human-readable skill name |

### Part Types

```rust
use systemprompt::models::a2a::{DataPart, FilePart, Part};

// Data part - JSON object
let data_part = Part::Data(DataPart {
    data: serde_json::Map::from_iter([
        ("key".to_string(), json!("value")),
    ]),
});

// File part - binary data
let file_part = Part::File(FilePart {
    file_data: FileBlobData {
        data: base64_encoded_string,
        mime_type: "image/png".to_string(),
    },
});
```

---

## Artifact Types (MUST Use Core Types)

**You MUST use artifact types from `systemprompt::models::artifacts`.** Do NOT create custom structs.

| Core Type | Import | Use For |
|-----------|--------|---------|
| `TextArtifact` | `systemprompt::models::artifacts::TextArtifact` | Blog posts, articles, documents, any text content |
| `ResearchArtifact` | `systemprompt::models::artifacts::ResearchArtifact` | Research with sources |
| `ImageArtifact` | `systemprompt::models::artifacts::ImageArtifact` | Generated images |
| `TableArtifact` | `systemprompt::models::artifacts::TableArtifact` | Tabular data |
| `ListArtifact` | `systemprompt::models::artifacts::ListArtifact` | Lists |
| `ChartArtifact` | `systemprompt::models::artifacts::ChartArtifact` | Charts and graphs |
| `CopyPasteTextArtifact` | `systemprompt::models::artifacts::CopyPasteTextArtifact` | Social content, snippets |
| `AudioArtifact` | `systemprompt::models::artifacts::AudioArtifact` | Audio files |
| `VideoArtifact` | `systemprompt::models::artifacts::VideoArtifact` | Video files |
| `DashboardArtifact` | `systemprompt::models::artifacts::DashboardArtifact` | Dashboard layouts |
| `PresentationCardArtifact` | `systemprompt::models::artifacts::PresentationCardArtifact` | Card presentations |

### Why Core Types Only?

1. **Schema Compatibility**: Core's `ArtifactBuilder` knows how to parse these types
2. **UI Rendering**: Renderers exist for core types
3. **Data Integrity**: Core types include required fields (full content, not previews)
4. **Type Safety**: Prevents data loss from missing fields

---

## Structured Responses

Return both human-readable and machine-readable content:

```rust
Ok(CallToolResult {
    // Human-readable text (markdown supported)
    content: vec![Content::text(format!(
        "## Research Complete\n\n\
         **Topic:** {topic}\n\n\
         **Sources Found:** {source_count}\n\n\
         **Artifact ID:** `{artifact_id}`\n\n\
         Use this artifact_id when calling create_blog_post."
    ))],

    // Machine-readable structured data
    structured_content: Some(json!({
        "artifact_id": artifact_id.to_string(),
        "topic": topic,
        "source_count": source_count,
        "research_summary": summary,
        "sources": sources,
        "status": "completed"
    })),

    // Success/error flag
    is_error: Some(false),

    // Optional metadata (artifact references)
    meta: Some(create_result_meta(artifact_id.as_str())),
})
```

### When to Use Each Field

| Field | Use When |
|-------|----------|
| `content` | Always — primary response for display |
| `structured_content` | Tool produces data clients may parse |
| `is_error` | Always — indicate success/failure |
| `meta` | Tool creates artifacts or has cross-references |

---

## ToolResponse Pattern (Agent Framework)

When building tools that integrate with the agent framework, use the typed `ToolResponse` wrapper to ensure schema compatibility:

```rust
use systemprompt::models::ExecutionMetadata;
use systemprompt::models::artifacts::{ToolResponse, ResearchArtifact, SourceCitation};
use systemprompt::identifiers::McpExecutionId;

// Build typed artifact
let sources: Vec<SourceCitation> = search_response.sources
    .iter()
    .map(|s| SourceCitation::new(&s.title, &s.uri, s.relevance))
    .collect();

let artifact = ResearchArtifact::new(topic, card, sources)
    .with_query_count(query_count as u32);

// Build metadata from request context
let metadata = ExecutionMetadata::with_request(&ctx)
    .tool("research_blog")
    .skill(skill_id, "Blog Research");

// Create typed response
let response = ToolResponse::new(
    &artifact_id,
    mcp_execution_id.clone(),
    artifact,
    metadata.clone(),
);

Ok(CallToolResult {
    content: vec![Content::text("Human readable...")],
    structured_content: Some(response.to_json()),
    is_error: Some(false),
    meta: metadata.to_meta(),
})
```

### ToolResponse Schema

The agent framework expects `structured_content` to follow this schema:

```json
{
  "artifact_id": "uuid-string",
  "mcp_execution_id": "uuid-string",
  "artifact": { ... typed artifact ... },
  "_metadata": { ... execution metadata ... }
}
```

### Key Pattern

| Field | Purpose |
|-------|---------|
| `content` | Text for LLMs (human-readable markdown) |
| `structured_content` | Typed `ToolResponse<T>` artifact |

### Available Artifact Types

| Type | Import | Use Case |
|------|--------|----------|
| `ResearchArtifact` | `systemprompt::models::artifacts` | Research results with sources |
| `TextArtifact` | `systemprompt::models::artifacts` | Simple text content |
| `TableArtifact` | `systemprompt::models::artifacts` | Tabular data |
| `ChartArtifact` | `systemprompt::models::artifacts` | Charts/graphs |
| `DashboardArtifact` | `systemprompt::models::artifacts` | Dashboard layouts |
| `PresentationCardArtifact` | `systemprompt::models::artifacts` | Card presentations |

### DO NOT Create Custom Artifact Structs

**WRONG** - This causes data loss:

```rust
// ❌ WRONG - Custom struct missing full content
#[derive(Serialize, Deserialize)]
pub struct BlogPostArtifact {
    pub title: String,
    pub content_preview: Option<String>,  // Only 1000 chars - DATA LOSS!
}
```

**CORRECT** - Use `TextArtifact` which has full content:

```rust
// ✅ CORRECT - Use core type with full content
use systemprompt::models::artifacts::TextArtifact;

let artifact = TextArtifact::new(full_blog_content, &ctx)
    .with_title(title)
    .with_skill(skill_id, skill_name);

let response = ToolResponse::new(artifact_id, mcp_execution_id, artifact, metadata);
```

If you need a new artifact type, **add it to systemprompt-core**, not as a custom struct in your MCP server.

---

## UI Resources

MCP servers can expose artifacts as UI resources for rendering:

### Enable Resources in Server

```rust
impl ServerHandler for MyServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()  // Enable resources
                .build(),
            // ...
        }
    }
}
```

### Implement Resource Templates

```rust
use rmcp::model::{
    ListResourceTemplatesResult, RawResourceTemplate, ResourceTemplate,
};
use systemprompt::mcp::services::ui_renderer::MCP_APP_MIME_TYPE;

const SERVER_NAME: &str = "my-server";

async fn list_resource_templates(
    &self,
    _request: Option<PaginatedRequestParams>,
    _ctx: RequestContext<RoleServer>,
) -> Result<ListResourceTemplatesResult, McpError> {
    let template = ResourceTemplate {
        raw: RawResourceTemplate {
            uri_template: format!("ui://{SERVER_NAME}/{{artifact_id}}"),
            name: "artifact-ui".to_string(),
            title: Some("Artifact UI".to_string()),
            description: Some(
                "Interactive UI for artifacts. Use with artifact IDs.".to_string()
            ),
            mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
            icons: None,
        },
        annotations: None,
    };

    Ok(ListResourceTemplatesResult {
        resource_templates: vec![template],
        next_cursor: None,
        meta: None,
    })
}
```

### Implement Resource Reading

```rust
use rmcp::model::{
    ReadResourceRequestParams, ReadResourceResult, ResourceContents,
};

async fn read_resource(
    &self,
    request: ReadResourceRequestParams,
    _ctx: RequestContext<RoleServer>,
) -> Result<ReadResourceResult, McpError> {
    let uri = &request.uri;

    // Parse artifact ID from URI
    let artifact_id = parse_ui_uri(uri).ok_or_else(|| {
        McpError::invalid_params(
            format!("Invalid URI: {uri}. Expected: ui://{SERVER_NAME}/{{artifact_id}}"),
            None,
        )
    })?;

    // Render artifact to HTML
    let html = render_artifact_ui(&self.db_pool, &self.ui_registry, &artifact_id)
        .await
        .map_err(|e| {
            McpError::internal_error(format!("Failed to render: {e}"), None)
        })?;

    let contents = ResourceContents::TextResourceContents {
        uri: uri.clone(),
        mime_type: Some(MCP_APP_MIME_TYPE.to_string()),
        text: html,
        meta: None,
    };

    Ok(ReadResourceResult {
        contents: vec![contents],
    })
}
```

### URI Parsing Helper

```rust
pub fn parse_ui_uri(uri: &str) -> Option<String> {
    let prefix = format!("ui://{SERVER_NAME}/");
    if uri.starts_with(&prefix) {
        Some(uri[prefix.len()..].to_string())
    } else {
        None
    }
}
```

---

## UI Renderer Registry

The `UiRendererRegistry` maps artifact types to HTML renderers:

```rust
use systemprompt::mcp::services::ui_renderer::{
    registry::create_default_registry,
    UiRendererRegistry,
};

#[derive(Clone)]
pub struct MyServer {
    db_pool: DbPool,
    service_id: McpServerId,
    ui_registry: Arc<UiRendererRegistry>,
}

impl MyServer {
    pub fn new(db_pool: DbPool, service_id: McpServerId) -> Self {
        Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(create_default_registry()),
        }
    }

    // Or extend with custom renderers
    pub fn with_extended_registry<F>(
        db_pool: DbPool,
        service_id: McpServerId,
        extend_fn: F,
    ) -> Self
    where
        F: FnOnce(&mut UiRendererRegistry),
    {
        let mut registry = create_default_registry();
        extend_fn(&mut registry);
        Self {
            db_pool,
            service_id,
            ui_registry: Arc::new(registry),
        }
    }
}
```

### Default Renderers

The default registry includes renderers for:

| Artifact Type | Renderer | Output |
|---------------|----------|--------|
| `table` | Table renderer | HTML table |
| `list` | List renderer | Ordered/unordered list |
| `card` | Card renderer | Card layout |
| `chart` | Chart renderer | Chart visualization |
| `dashboard` | Dashboard renderer | Dashboard layout |
| `form` | Form renderer | Interactive form |
| `command_result` | CLI renderer | Command output |

---

## Artifact Rendering

```rust
use systemprompt::mcp::services::ui_renderer::UiRendererRegistry;

pub async fn render_artifact_ui(
    db_pool: &DbPool,
    ui_registry: &UiRendererRegistry,
    artifact_id: &str,
) -> Result<String> {
    // Load artifact from database
    let artifact = load_artifact(db_pool, artifact_id).await?;

    // Get renderer for artifact type
    let artifact_type = &artifact.metadata.artifact_type;
    let renderer = ui_registry
        .get(artifact_type)
        .ok_or_else(|| anyhow!("No renderer for type: {artifact_type}"))?;

    // Render to HTML
    let html = renderer.render(&artifact)?;

    Ok(html)
}
```

---

## Result Meta Pattern

Include artifact references in response meta:

```rust
pub fn create_result_meta(artifact_id: &str) -> serde_json::Map<String, serde_json::Value> {
    let mut meta = serde_json::Map::new();
    meta.insert(
        "artifact_id".to_string(),
        json!(artifact_id),
    );
    meta.insert(
        "ui_uri".to_string(),
        json!(format!("ui://{SERVER_NAME}/{artifact_id}")),
    );
    meta
}

// In handler:
Ok(CallToolResult {
    content: vec![Content::text("...")],
    structured_content: Some(json!({...})),
    is_error: Some(false),
    meta: Some(create_result_meta(artifact_id.as_str())),
})
```

---

## Loading Artifacts

Retrieve artifacts for subsequent tools:

```rust
pub async fn load_artifact_data(
    artifact_repo: &ArtifactRepository,
    artifact_id: &str,
) -> Result<serde_json::Map<String, serde_json::Value>, McpError> {
    let artifact_id = ArtifactId::parse(artifact_id)
        .map_err(|_| McpError::invalid_params("Invalid artifact_id format", None))?;

    let artifact = artifact_repo
        .get_artifact(&artifact_id)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to load artifact: {e}"), None))?
        .ok_or_else(|| McpError::invalid_params(
            format!("Artifact not found: {artifact_id}"),
            None,
        ))?;

    // Extract data from first DataPart
    let data = artifact
        .parts
        .iter()
        .find_map(|p| match p {
            Part::Data(d) => Some(d.data.clone()),
            _ => None,
        })
        .unwrap_or_default();

    Ok(data)
}
```

---

## Checklist

- [ ] **Use CORE artifact type** (`TextArtifact`, `ResearchArtifact`, `ImageArtifact`, etc.)
- [ ] **DO NOT create custom artifact structs** - use core types or request new ones in core
- [ ] **Include FULL content** in artifact - never truncate or use previews
- [ ] Generate unique artifact_id (UUID) for tracking
- [ ] Create `ExecutionMetadata` from request context with `.with_tool()` and `.with_skill()`
- [ ] Wrap in `ToolResponse::new(artifact_id, mcp_execution_id, artifact, metadata)`
- [ ] Return artifact in `structured_content: response.to_json().ok()`
- [ ] Include `meta: metadata.to_meta()`
- [ ] **DO NOT** call `artifact_repo.create_artifact()` for output artifacts
- [ ] **CAN** use `artifact_repo.get_artifact_by_id()` for input artifacts
- [ ] For UI resources: implement `list_resource_templates`
- [ ] For UI resources: implement `read_resource`

---

## Quick Reference

| Task | Code |
|------|------|
| Generate ID | `ArtifactId::generate()` or `uuid::Uuid::new_v4().to_string()` |
| Text content | `TextArtifact::new(full_content, &ctx).with_title(title)` |
| Research | `ResearchArtifact::new(topic, card, sources)` |
| Build metadata | `ExecutionMetadata::with_request(&ctx).with_tool(name).with_skill(id, name)` |
| Wrap response | `ToolResponse::new(artifact_id, mcp_execution_id, artifact, metadata)` |
| Return artifact | `structured_content: response.to_json().ok()` |
| Return meta | `meta: metadata.to_meta()` |
| Load input artifact | `artifact_repo.get_artifact_by_id(&artifact_id).await` (READ is OK) |

### Architecture Rules

| Rule | Reason |
|------|--------|
| Use CORE artifact types only | Schema compatibility, UI rendering, data integrity |
| Include FULL content | Truncated content is permanently lost |
| MCP tools DO NOT save artifacts | Agent owns task_ids, handles FK integrity |
| MCP tools CAN read artifacts | Loading input data is valid |
| Return data in `structured_content` | Agent transforms and persists |
| Use `ToolResponse<T>` wrapper | Ensures schema compatibility |

---

## Related Playbooks

- [MCP Tutorial](build_mcp-tutorial) — Building your first MCP server
- [MCP Tool Patterns](build_mcp-tools) — Modular tool organization
- [MCP Checklist](build_mcp-checklist) — Full requirements checklist
