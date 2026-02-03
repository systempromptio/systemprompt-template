---
title: "MCP Server Checklist Playbook"
description: "Complete checklist for building MCP servers on systemprompt-core."
keywords:
  - mcp
  - server
  - checklist
  - build
category: build
playbook_references:
  - id: "build_mcp-tutorial"
    description: "Step-by-step MCP server tutorial"
  - id: "build_mcp-tools"
    description: "Tool organization patterns"
  - id: "build_mcp-artifacts"
    description: "Artifacts and UI resources"
  - id: "build_mcp-review"
    description: "Code review process"
  - id: "build_rust-standards"
    description: "Rust coding standards"
  - id: "build_extension-checklist"
    description: "General extension patterns"
---

# MCP Server Checklist

> **Help**: `{ "command": "core playbooks show build_mcp-checklist" }`

**Applies to:** All MCP server crates in `extensions/mcp/`

> **Reference Implementation**: See `extensions/mcp/systemprompt/` for a working example.

---

## Core Principle

**MCP servers are Rust code and belong in `/extensions/mcp/`, not `/services/mcp/`.**

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

## Tool Response Requirements (MANDATORY)

**All MCP tools MUST return proper tracking metadata and structured content.** Tools that fail to include these will be marked as FAILED by the agent framework.

### Required Components

| Field | Required | Purpose |
|-------|----------|---------|
| `content` | Yes | Human-readable text response (markdown) |
| `structured_content` | Yes | Machine-readable JSON artifact data |
| `is_error` | Yes | Boolean success/failure flag |
| `meta` | Yes | Execution tracking metadata from `ExecutionMetadata.to_meta()` |

### RBAC and Context Setup

Every `call_tool` handler MUST:

1. Enforce RBAC to get authenticated context
2. Extract `RequestContext` for tracking
3. Generate `McpExecutionId` for the tool call

```rust
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::middleware::enforce_rbac_from_registry;
use systemprompt::models::execution::context::RequestContext;
use systemprompt::models::ExecutionMetadata;

async fn call_tool(
    &self,
    request: CallToolRequestParams,
    ctx: RmcpRequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // 1. Enforce RBAC
    let auth_result = enforce_rbac_from_registry(&ctx, self.service_id.as_str()).await?;
    let authenticated_ctx = auth_result
        .expect_authenticated("my-server requires OAuth")?;

    // 2. Extract context for tracking
    let request_context = authenticated_ctx.context.clone();

    // 3. Generate execution ID
    let mcp_execution_id = McpExecutionId::generate();

    // 4. Handle tool with context
    handle_my_tool(request, &request_context, &mcp_execution_id).await
}
```

---

## ToolResponse and Artifacts (CRITICAL)

**MCP tools MUST return artifacts using `ToolResponse<T>` where `T` is a core artifact type.** This is how the agent framework persists tool outputs.

### Understanding the Data Flow

```
┌─────────────────────────────────────────────────────────────────┐
│ MCP Tool Handler                                                │
│                                                                 │
│  1. Create artifact using CORE artifact type (TextArtifact,    │
│     ResearchArtifact, ImageArtifact, etc.)                     │
│                                                                 │
│  2. Wrap in ToolResponse<T> with metadata                      │
│                                                                 │
│  3. Return as CallToolResult.structured_content                │
└─────────────────────────────────────────────────────────────────┘
                              ↓
┌─────────────────────────────────────────────────────────────────┐
│ Agent Framework (Core)                                          │
│                                                                 │
│  1. Parses structured_content as ToolResponse                  │
│     Expected schema: {artifact_id, mcp_execution_id,           │
│                       artifact, _metadata}                     │
│                                                                 │
│  2. Transforms artifact field → A2A Artifact with Parts        │
│                                                                 │
│  3. Persists to database via ArtifactPublishingService         │
└─────────────────────────────────────────────────────────────────┘
```

### ToolResponse Schema

`ToolResponse<T>` serializes to this exact JSON structure:

```json
{
  "artifact_id": "uuid-string",
  "mcp_execution_id": "uuid-string",
  "artifact": { <T serialized - your artifact data> },
  "_metadata": {
    "context_id": "...",
    "trace_id": "...",
    "session_id": "...",
    "user_id": "...",
    "agent_name": "...",
    "timestamp": "...",
    "tool_name": "...",
    "skill_id": "...",
    "skill_name": "..."
  }
}
```

### Available Artifact Types (from Core)

**You MUST use these types for the `T` in `ToolResponse<T>`:**

| Type | Import | Use For |
|------|--------|---------|
| `TextArtifact` | `systemprompt::models::artifacts::TextArtifact` | Text content (blog posts, articles, documents) |
| `ResearchArtifact` | `systemprompt::models::artifacts::ResearchArtifact` | Research with sources |
| `ImageArtifact` | `systemprompt::models::artifacts::ImageArtifact` | Generated images |
| `TableArtifact` | `systemprompt::models::artifacts::TableArtifact` | Tabular data |
| `ListArtifact` | `systemprompt::models::artifacts::ListArtifact` | Lists |
| `ChartArtifact` | `systemprompt::models::artifacts::ChartArtifact` | Charts/graphs |
| `CopyPasteTextArtifact` | `systemprompt::models::artifacts::CopyPasteTextArtifact` | Copyable text snippets |

**DO NOT create custom artifact structs.** Use core types or request a new type be added to core.

### Correct Implementation

```rust
use systemprompt::models::artifacts::{TextArtifact, ToolResponse};
use systemprompt::models::ExecutionMetadata;
use systemprompt::identifiers::{ArtifactId, McpExecutionId};

async fn handle_my_tool(
    request: CallToolRequestParams,
    ctx: &RequestContext,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    // 1. Do your work and generate content
    let content = generate_content().await?;

    // 2. Create artifact using CORE type
    let artifact = TextArtifact::new(&content, ctx)
        .with_title("My Generated Content")
        .with_skill(skill_id, "My Skill");

    // 3. Build execution metadata
    let metadata = ExecutionMetadata::with_request(ctx)
        .with_tool("my_tool")
        .with_skill(skill_id, "My Skill");

    // 4. Wrap in ToolResponse
    let response = ToolResponse::new(
        ArtifactId::generate(),
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    // 5. Return with structured_content
    Ok(CallToolResult {
        content: vec![Content::text("Human readable summary")],
        structured_content: response.to_json().ok(),  // REQUIRED
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
```

### WRONG - Do Not Do This

```rust
// ❌ WRONG: Custom struct instead of core artifact type
struct MyCustomArtifact {
    title: String,
    preview: String,  // Only partial content!
}

// ❌ WRONG: Raw JSON instead of ToolResponse
let structured = json!({
    "data": { "result": "..." },
    "artifact_type": "text"
});

// ❌ WRONG: Content not included in artifact
let artifact = BlogPostArtifact {
    title: title,
    content_preview: content.chars().take(1000).collect(),  // LOSING DATA!
};
```

### Checklist

- [ ] Using core artifact type (`TextArtifact`, `ImageArtifact`, etc.)
- [ ] Full content included in artifact (not truncated/preview)
- [ ] Wrapped in `ToolResponse::new(artifact_id, mcp_execution_id, artifact, metadata)`
- [ ] `structured_content` set to `response.to_json().ok()`
- [ ] `meta` set to `metadata.to_meta()`
- [ ] `is_error` always set to `Some(true)` or `Some(false)`

### Why This Matters

1. **Persistence**: Core parses `ToolResponse` schema exactly - wrong format = lost data
2. **Full Content**: Artifact data is persisted as-is - truncated content is permanently lost
3. **Type Safety**: Core artifact types have proper schemas for UI rendering
4. **Traceability**: `_metadata` links artifacts to traces, users, sessions

---

## Tool Execution Tracking (MANDATORY)

**All MCP tools MUST record their executions** to the `mcp_tool_executions` database table. This enables trace visibility and debugging.

### Why This is Critical

The `mcp_calls` counter displayed in traces is **computed dynamically** by counting rows:

```sql
SELECT COUNT(*) FROM mcp_tool_executions WHERE trace_id = ?
```

If you don't insert rows, `mcp_calls` will always be 0, making debugging impossible.

### Required Integration

Add `ToolUsageRepository` to your server and wrap tool calls:

```rust
use systemprompt::mcp::repository::ToolUsageRepository;
use systemprompt::mcp::models::{ExecutionStatus, ToolExecutionRequest, ToolExecutionResult};

// In server constructor
pub struct MyServer {
    // ... other fields ...
    pub tool_usage_repo: Arc<ToolUsageRepository>,
}

// In call_tool handler
async fn call_tool(&self, request: CallToolRequestParams, ctx: RequestContext<RoleServer>) -> Result<CallToolResult, McpError> {
    let started_at = Utc::now();

    // ... RBAC and context setup ...

    // 1. Record execution start
    let execution_request = ToolExecutionRequest {
        tool_name: tool_name.clone(),
        server_name: self.service_id.to_string(),
        input: serde_json::to_value(&request.arguments).unwrap_or_default(),
        started_at,
        context: request_context.clone(),
        request_method: Some("mcp".to_string()),
        request_source: Some("my-server".to_string()),
        ai_tool_call_id: None,
    };

    let mcp_execution_id = self.tool_usage_repo
        .start_execution(&execution_request)
        .await?;

    // 2. Execute tool
    let result = handle_tool_call(..., &mcp_execution_id).await;

    // 3. Record execution completion
    let execution_result = ToolExecutionResult {
        output: result.as_ref().ok().and_then(|r| r.structured_content.clone()),
        output_schema: None,
        status: if result.is_ok() { "success" } else { "failed" }.to_string(),
        error_message: result.as_ref().err().map(|e| e.message.clone()),
        started_at,
        completed_at: Utc::now(),
    };

    self.tool_usage_repo.complete_execution(&mcp_execution_id, &execution_result).await?;

    result
}
```

### Checklist

- [ ] `ToolUsageRepository` added to server struct
- [ ] `start_execution()` called before tool execution
- [ ] `complete_execution()` called after tool execution (even on error)
- [ ] Execution errors logged but don't block tool response

### Reference Implementation

See `extensions/mcp/content-manager/src/server/mod.rs` for a working example.

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

## Environment Variables

The MCP spawner automatically passes these environment variables to all MCP servers:

### Required Variables

| Variable | Description |
|----------|-------------|
| `SYSTEMPROMPT_PROFILE` | Path to the active profile configuration |
| `JWT_SECRET` | JWT signing secret for authentication |
| `DATABASE_URL` | Database connection string |
| `DATABASE_TYPE` | Database type (e.g., `postgres`) |
| `MCP_SERVICE_ID` | Unique service identifier |
| `MCP_PORT` | Port the server should listen on |
| `AI_CONFIG_PATH` | Path to AI configuration file (required for AiService) |
| `SYSTEM_PATH` | Path to system root directory |

### Configuration Variables

| Variable | Description |
|----------|-------------|
| `MCP_TOOLS_CONFIG` | JSON-serialized tool configuration |
| `MCP_SERVER_MODEL_CONFIG` | JSON-serialized model configuration |

### Optional API Keys

These are passed only if configured in secrets:

| Variable | Description |
|----------|-------------|
| `GEMINI_API_KEY` | Google Gemini API key |
| `ANTHROPIC_API_KEY` | Anthropic API key |
| `OPENAI_API_KEY` | OpenAI API key |
| `GITHUB_TOKEN` | GitHub access token |

### Custom Secrets

| Variable | Description |
|----------|-------------|
| `SYSTEMPROMPT_CUSTOM_SECRETS` | Comma-separated list of custom secret names |
| `{SECRET_NAME}` | Each custom secret is passed as its own env var |

### Reading Environment Variables

```rust
// Required - will error if not set
let ai_config_path = std::env::var("AI_CONFIG_PATH")
    .context("AI_CONFIG_PATH environment variable must be set")?;

// Optional with default
let port = std::env::var("MCP_PORT")
    .ok()
    .and_then(|p| p.parse::<u16>().ok())
    .unwrap_or(5050);
```

### Path Resolution (CRITICAL)

**NEVER hardcode absolute paths like `/app/storage/`. Use `FilesConfig` for validated storage paths.**

Paths must work for both:
- **Local profiles**: `/var/www/html/systemprompt-web/storage/...`
- **Cloud profiles**: `/app/storage/...`

```rust
use systemprompt::files::FilesConfig;

// Initialize and get validated config
FilesConfig::init().context("Failed to initialize FilesConfig")?;
let files_config = FilesConfig::get().context("Failed to get FilesConfig")?;

// Use validated paths and URLs
let storage_config = StorageConfig::new(
    files_config.generated_images(),  // Validated path
    format!("{}/images/generated", files_config.url_prefix()),  // Validated URL
);
```

**FilesConfig provides these validated paths:**
| Method | Returns |
|--------|---------|
| `storage()` | Storage root path |
| `images()` | `storage/files/images` |
| `generated_images()` | `storage/files/images/generated` |
| `files()` | `storage/files` |
| `audio()` | `storage/files/audio` |
| `video()` | `storage/files/video` |
| `documents()` | `storage/files/documents` |
| `uploads()` | `storage/files/uploads` |

**FilesConfig provides these validated URLs:**
| Method | Returns |
|--------|---------|
| `url_prefix()` | Base URL prefix (e.g., `/files`) |
| `image_url(path)` | `/files/images/{path}` |
| `generated_image_url(name)` | `/files/images/generated/{name}` |
| `file_url(path)` | `/files/files/{path}` |

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

## Building & Testing

### Build via CLI (Recommended)

```bash
# Build all MCP servers
systemprompt build mcp

# Build all MCP servers in release mode
systemprompt build mcp --release

# Check MCP server status
systemprompt plugins mcp status

# List available MCP tools
systemprompt plugins mcp tools
```

### Build via Cargo (Alternative)

```bash
# Build specific MCP server
cargo build -p systemprompt-mcp-my-server

# Run in development
cargo run -p systemprompt-mcp-my-server

# Lint
cargo clippy -p systemprompt-mcp-my-server -- -D warnings

# Format
cargo fmt -p systemprompt-mcp-my-server -- --check
```

### Testing Tools

```bash
# List tools from a specific MCP server
systemprompt plugins mcp tools

# Call a tool directly (positional args: server tool)
systemprompt plugins mcp call moltbook moltbook_read --args '{"limit": 5}'
```

---

## Registering a New MCP Server

After building your MCP server, you must register it to make it discoverable.

### Step 1: Create Config File

Create `services/mcp/{name}.yaml`:

```yaml
mcp_servers:
  my-server:
    binary: "systemprompt-mcp-my-server"
    package: "my-server"
    port: 5030
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

### Step 2: Add to Config Includes

Edit `services/config/config.yaml` and add your config to the includes list:

```yaml
includes:
  # ... existing includes ...
  - ../mcp/my-server.yaml
```

### Step 3: Verify Registration

```bash
# Should show your server
systemprompt plugins mcp list

# Start services
systemprompt infra services start --all
```

**Note:** If you get "Failed to parse include" errors, compare your YAML byte-by-byte with a working config. The error messages don't show which field failed validation.

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
| Build all MCP | `systemprompt build mcp` |
| Build release | `systemprompt build mcp --release` |
| Check status | `systemprompt plugins mcp status` |
| List tools | `systemprompt plugins mcp tools` |
| Call tool | `systemprompt plugins mcp call --server {name} --tool {tool} --args '{json}'` |
| Build single | `cargo build -p systemprompt-mcp-{name}` |
| Run single | `cargo run -p systemprompt-mcp-{name}` |
| Lint | `cargo clippy -p systemprompt-mcp-{name} -- -D warnings` |
| Format | `cargo fmt -p systemprompt-mcp-{name} -- --check` |

## AI Service Integration

MCP servers that need AI capabilities (content generation, search grounding) must integrate the AiService.

### Required Imports

```rust
// AI Services (from facade crate)
use systemprompt::ai::{
    AiService,              // Main service
    AiConfig,               // Configuration
    AiMessage,              // Conversation message
    MessageRole,            // User, System, Assistant
    GoogleSearchParams,     // Search grounding params
    SearchGroundedResponse, // Response with sources
    NoopToolProvider,       // Required dummy provider
};

// Agent Services
use systemprompt::agent::services::SkillService;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::{Artifact, ArtifactMetadata, DataPart, Part};

// Config Loading
use systemprompt::loader::EnhancedConfigLoader;

// Identifiers
use systemprompt::identifiers::{ArtifactId, ContextId, TaskId};
```

### AiService Initialization

```rust
use std::sync::Arc;
use systemprompt::ai::{AiService, NoopToolProvider};
use systemprompt::loader::EnhancedConfigLoader;

pub fn new(db_pool: DbPool, service_id: McpServerId, _ctx: Arc<AppContext>) -> Result<Self> {
    // Load config
    let config_loader = EnhancedConfigLoader::from_env()?;
    let services_config = config_loader.load()?;

    // Create AiService - NoopToolProvider required even if not using tools
    let tool_provider = Arc::new(NoopToolProvider::new());
    let ai_service = Arc::new(
        AiService::new(
            db_pool.clone(),
            &services_config.ai,  // AiConfig is in ServicesConfig
            tool_provider,
            None,  // No session provider
        )?
    );

    // ...
}
```

### Building AiMessage

AiMessage requires `role`, `content`, and `parts` fields:

```rust
// Helper methods (recommended)
let msg = AiMessage::system("You are helpful.");
let msg = AiMessage::user("Hello");

// Or full construction
let msg = AiMessage {
    role: MessageRole::System,
    content: "...".to_string(),
    parts: vec![],  // Required, even if empty
};
```

### Google Search Grounding

```rust
let params = GoogleSearchParams {
    messages: vec![
        AiMessage::system(&skill_content),
        AiMessage::user(&prompt),
    ],
    sampling: None,
    max_output_tokens: 8192,
    model: Some("gemini-2.5-flash"),
    urls: None,
    response_schema: None,
};

let response = ai_service.generate_with_google_search(params).await?;
// response.content - Generated text
// response.sources - Vec<WebSource> with title, uri, relevance
// response.web_search_queries - Queries used
```

### Logging

Use `tracing` directly (LogService is not exported via facade):

```rust
use tracing::{info, warn, error};

info!(topic = %topic, "Starting research");
error!(error = %e, "Operation failed");
```

### RBAC Middleware

```rust
use systemprompt::mcp::middleware::enforce_rbac_from_registry;

// Takes 2 arguments (not 3)
let auth_result = enforce_rbac_from_registry(&ctx, service_id.as_str()).await?;
let authenticated_ctx = auth_result.expect_authenticated("Auth required")?;
let request_context = authenticated_ctx.context.clone();
```

### Error Conversion

Convert anyhow errors to MCP errors:

```rust
use rmcp::ErrorData as McpError;

let response = ai_service.generate_with_google_search(params)
    .await
    .map_err(|e| McpError::internal_error(format!("AI error: {}", e), None))?;
```

---

## Import Mapping: Individual Crates vs Facade

When porting code from systemprompt-blog (individual crates) to systemprompt-web (facade):

| systemprompt-blog | systemprompt-web |
|-------------------|------------------|
| `systemprompt_core_ai::AiService` | `systemprompt::ai::AiService` |
| `systemprompt_core_agent::services::SkillService` | `systemprompt::agent::services::SkillService` |
| `systemprompt_core_database::DbPool` | `systemprompt::database::DbPool` |
| `systemprompt_models::*` | `systemprompt::models::*` |
| `systemprompt_identifiers::*` | `systemprompt::identifiers::*` |
| `systemprompt_core_logging::LogService` | Use `tracing` directly |

---

## Reference Implementation

| Concept | Location |
|---------|----------|
| MCP server | `extensions/mcp/systemprompt/` |
| MCP with AI | `extensions/mcp/content-manager/` |
| Tools | `extensions/mcp/systemprompt/src/tools/` |
| Server constructor | `extensions/mcp/systemprompt/src/server/` |

## Related Playbooks

| Playbook | Description |
|----------|-------------|
| [MCP Tutorial](build_mcp-tutorial) | Step-by-step guide to building your first MCP server |
| [MCP Tool Patterns](build_mcp-tools) | Modular tool organization and handler patterns |
| [MCP Artifacts](build_mcp-artifacts) | Creating artifacts and UI resources |
| [MCP Review](build_mcp-review) | Code review process |

## Related Documentation

| Document | Description |
|----------|-------------|
| [MCP Extensions](/documentation/extensions/domains/mcp) | High-level MCP overview |
| [Tool Structure](/documentation/extensions/mcp/tool-structure) | Detailed tool organization reference |
| [Resources](/documentation/extensions/mcp/resources) | MCP resources and templates |
| [Skills](/documentation/extensions/mcp/skills) | Skill integration patterns |
| [Responses](/documentation/extensions/mcp/responses) | Response formatting best practices |
| [MCP AI Integration](/documentation/extensions/mcp-ai-integration) | Full AI service guide |

-> See [Architecture](build_architecture) for layer model and dependency rules.
-> See [Extension Checklist](build_extension-checklist) for common patterns (errors, services, repositories).
-> See [Rust Standards](build_rust-standards) for code quality.
