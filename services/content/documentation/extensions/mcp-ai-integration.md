---
title: "MCP Server AI Integration"
description: "Complete guide to integrating AI services (Gemini, Anthropic, OpenAI) into MCP servers with search grounding and artifact persistence."
author: "SystemPrompt Team"
slug: "extensions/mcp-ai-integration"
keywords: "mcp, ai, gemini, search, grounding, artifacts, integration"
image: "/files/images/docs/extensions-mcp-ai.svg"
kind: "guide"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Server AI Integration

This guide covers integrating AI services into MCP servers, including Gemini with Google Search grounding, content generation, and artifact persistence.

## Prerequisites

- Working MCP server (see [MCP Extensions](/documentation/extensions/domains/mcp))
- `systemprompt` facade crate with `features = ["full"]`
- Configured AI providers in `services/ai/config.yaml`

## Dependencies

```toml
[dependencies]
systemprompt = { workspace = true, features = ["full"] }
rmcp = { workspace = true }
axum = { workspace = true }
tokio = { workspace = true }
anyhow = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
```

## Import Reference

The `systemprompt` facade crate re-exports modules from systemprompt-core:

```rust
// AI Services
use systemprompt::ai::{
    AiService,           // Main AI service
    AiConfig,            // Provider configuration
    AiMessage,           // Message for conversations
    AiRequest,           // Request builder
    AiResponse,          // Standard response
    MessageRole,         // User, System, Assistant
    GoogleSearchParams,  // Search grounding params
    SearchGroundedResponse, // Response with sources
    NoopToolProvider,    // Dummy tool provider
};

// Agent Services
use systemprompt::agent::services::SkillService;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::{Artifact, ArtifactMetadata, DataPart, Part};

// Identifiers
use systemprompt::identifiers::{
    ArtifactId,
    ContextId,
    TaskId,
    McpServerId,
};

// Config Loading
use systemprompt::loader::EnhancedConfigLoader;

// Database
use systemprompt::database::DbPool;
```

## Server Constructor with AI Service

```rust
use std::sync::Arc;
use anyhow::{Context, Result};
use systemprompt::ai::{AiService, NoopToolProvider};
use systemprompt::agent::services::SkillService;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpServerId;
use systemprompt::loader::EnhancedConfigLoader;
use systemprompt::system::AppContext;

#[derive(Clone)]
pub struct MyAiServer {
    pub db_pool: DbPool,
    pub service_id: McpServerId,
    pub ai_service: Arc<AiService>,
    pub skill_loader: Arc<SkillService>,
    pub artifact_repo: ArtifactRepository,
}

impl MyAiServer {
    pub fn new(
        db_pool: DbPool,
        service_id: McpServerId,
        _ctx: Arc<AppContext>,
    ) -> Result<Self> {
        // Load services configuration
        let config_loader = EnhancedConfigLoader::from_env()
            .context("Failed to create config loader")?;
        let services_config = config_loader.load()
            .context("Failed to load services config")?;

        // Create AI service with NoopToolProvider
        // NoopToolProvider is required even if not using tool execution
        let tool_provider = Arc::new(NoopToolProvider::new());
        let ai_service = Arc::new(
            AiService::new(
                db_pool.clone(),
                &services_config.ai,
                tool_provider,
                None,  // No session provider
            ).context("Failed to initialize AiService")?
        );

        // Initialize other services
        let skill_loader = Arc::new(SkillService::new(db_pool.clone()));
        let artifact_repo = ArtifactRepository::new(db_pool.clone());

        Ok(Self {
            db_pool,
            service_id,
            ai_service,
            skill_loader,
            artifact_repo,
        })
    }
}
```

## Building AiMessage

AiMessage requires three fields: `role`, `content`, and `parts`.

```rust
use systemprompt::ai::{AiMessage, MessageRole};

// Method 1: Full struct construction
let message = AiMessage {
    role: MessageRole::System,
    content: "You are a helpful assistant.".to_string(),
    parts: vec![],  // Required, even if empty
};

// Method 2: Helper methods (recommended)
let system_msg = AiMessage::system("You are a helpful assistant.");
let user_msg = AiMessage::user("What is MCP?");
```

## Gemini with Google Search Grounding

Use `generate_with_google_search()` to leverage Google Search for real-time information:

```rust
use systemprompt::ai::{GoogleSearchParams, SearchGroundedResponse, AiMessage};

pub async fn research_topic(
    ai_service: &AiService,
    skill_content: &str,
    topic: &str,
) -> Result<SearchGroundedResponse> {
    // Build messages
    let messages = vec![
        AiMessage::system(skill_content),
        AiMessage::user(&format!(
            "Research the following topic thoroughly using web search:\n\n{}",
            topic
        )),
    ];

    // Configure search params
    let params = GoogleSearchParams {
        messages,
        sampling: None,
        max_output_tokens: 8192,
        model: Some("gemini-2.5-flash"),
        urls: None,  // Optional: specific URLs to analyze
        response_schema: None,  // Optional: structured output schema
    };

    // Execute search-grounded generation
    let response = ai_service
        .generate_with_google_search(params)
        .await
        .context("Gemini search failed")?;

    Ok(response)
}
```

### SearchGroundedResponse Fields

```rust
pub struct SearchGroundedResponse {
    pub content: String,                    // Generated text
    pub sources: Vec<WebSource>,            // Search result sources
    pub confidence_scores: Vec<f32>,        // Confidence per source
    pub web_search_queries: Vec<String>,    // Queries used
    pub url_context_metadata: Option<Vec<UrlMetadata>>,
    pub tokens_used: Option<u32>,
    pub latency_ms: u64,
    pub finish_reason: Option<String>,
    pub safety_ratings: Option<Vec<serde_json::Value>>,
}

pub struct WebSource {
    pub title: String,
    pub uri: String,
    pub relevance: f32,
}
```

## Standard AI Generation

For content generation without search:

```rust
use systemprompt::ai::{AiRequest, AiRequestBuilder, AiResponse};

pub async fn generate_content(
    ai_service: &AiService,
    system_prompt: &str,
    user_prompt: &str,
    ctx: &RequestContext,
) -> Result<String> {
    let request = AiRequestBuilder::new(
        vec![
            AiMessage::system(system_prompt),
            AiMessage::user(user_prompt),
        ],
        "gemini",           // provider
        "gemini-2.5-pro",   // model
        32768,              // max_output_tokens
        ctx.clone(),
    ).build();

    let response = ai_service
        .generate(&request)
        .await
        .context("Content generation failed")?;

    Ok(response.content)
}
```

## Artifact Storage

Store results as artifacts for later retrieval:

```rust
use systemprompt::agent::{Artifact, ArtifactMetadata, DataPart, Part};
use systemprompt::identifiers::{ArtifactId, ContextId, TaskId};

pub async fn store_research_artifact(
    artifact_repo: &ArtifactRepository,
    topic: &str,
    response: &SearchGroundedResponse,
    context_id: &str,
    skill_id: &str,
) -> Result<String> {
    let artifact_id = ArtifactId::generate();
    let task_id = TaskId::generate();
    let context_id = ContextId::new(context_id);

    // Build artifact data
    let research_data = serde_json::json!({
        "topic": topic,
        "summary": response.content,
        "sources": response.sources.iter().map(|s| {
            serde_json::json!({
                "title": s.title,
                "uri": s.uri,
                "relevance": s.relevance
            })
        }).collect::<Vec<_>>(),
        "queries": response.web_search_queries,
        "tokens_used": response.tokens_used,
    });

    let artifact = Artifact {
        id: artifact_id.clone(),
        name: Some(format!("Research: {}", topic)),
        description: Some("Blog research with sources".to_string()),
        parts: vec![Part::Data(DataPart {
            data: research_data.as_object().unwrap().clone(),
        })],
        extensions: vec![],
        metadata: ArtifactMetadata {
            artifact_type: "research".to_string(),
            context_id: context_id.clone(),
            task_id: task_id.clone(),
            created_at: chrono::Utc::now().to_rfc3339(),
            tool_name: Some("research_topic".to_string()),
            skill_id: Some(skill_id.to_string()),
            skill_name: Some("Research".to_string()),
            ..Default::default()
        },
    };

    artifact_repo
        .create_artifact(&task_id, &context_id, &artifact)
        .await
        .context("Failed to store artifact")?;

    Ok(artifact_id.to_string())
}
```

## Loading Artifacts

Retrieve previously stored artifacts:

```rust
pub async fn load_artifact(
    artifact_repo: &ArtifactRepository,
    artifact_id: &str,
) -> Result<serde_json::Map<String, serde_json::Value>> {
    let artifact = artifact_repo
        .get_artifact_by_id(&ArtifactId::new(artifact_id))
        .await?
        .ok_or_else(|| anyhow::anyhow!(
            "Artifact not found: {}. Use artifact_id from previous tool call.",
            artifact_id
        ))?;

    // Extract data from artifact parts
    let data = artifact.parts.iter()
        .find_map(|p| match p {
            Part::Data(DataPart { data }) => Some(data.clone()),
            _ => None,
        })
        .ok_or_else(|| anyhow::anyhow!("Invalid artifact format"))?;

    Ok(data)
}
```

## Error Handling

Convert between AI errors and MCP errors:

```rust
use rmcp::ErrorData as McpError;

// Helper function for error conversion
fn ai_error_to_mcp(e: anyhow::Error, context: &str) -> McpError {
    McpError::internal_error(
        format!("{}: {}", context, e),
        None
    )
}

// Usage in tool handler
pub async fn handle_my_tool(
    server: &MyAiServer,
    request: CallToolRequestParams,
) -> Result<CallToolResult, McpError> {
    let response = server.ai_service
        .generate_with_google_search(params)
        .await
        .map_err(|e| ai_error_to_mcp(e, "Research failed"))?;

    // ... rest of handler
}
```

## Logging

Use `tracing` for logging (LogService is not exported via facade):

```rust
use tracing::{info, warn, error, debug};

// In tool handlers
info!(topic = %topic, "Starting research");
debug!(model = "gemini-2.5-flash", "Calling AI service");

if response.sources.is_empty() {
    warn!("No sources returned from search");
}

if let Err(e) = result {
    error!(error = %e, "Tool execution failed");
}
```

## Loading Skills

Load skill content for prompts:

```rust
use systemprompt::models::execution::context::RequestContext;

pub async fn load_skill_content(
    skill_loader: &SkillService,
    skill_id: &str,
    ctx: &RequestContext,
) -> Result<String, McpError> {
    skill_loader
        .load_skill(skill_id, ctx)
        .await
        .map_err(|e| McpError::internal_error(
            format!("Failed to load skill '{}': {}", skill_id, e),
            None
        ))
}
```

## RBAC Enforcement

Enforce authentication before tool execution:

```rust
use systemprompt::mcp::middleware::enforce_rbac_from_registry;

pub async fn call_tool(
    &self,
    request: CallToolRequestParams,
    ctx: RequestContext<RoleServer>,
) -> Result<CallToolResult, McpError> {
    // Enforce RBAC - takes 2 arguments
    let auth_result = enforce_rbac_from_registry(
        &ctx,
        self.service_id.as_str()
    ).await?;

    // Get authenticated context
    let authenticated_ctx = auth_result
        .expect_authenticated("This MCP server requires authentication")?;

    let request_context = authenticated_ctx.context.clone();

    // ... execute tool with request_context
}
```

## Complete Tool Handler Example

```rust
pub async fn handle_research_blog(
    server: &MyAiServer,
    request: CallToolRequestParams,
    ctx: RequestContext,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments
        .ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;

    let topic = args.get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing: topic", None))?;

    let skill_id = args.get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing: skill_id", None))?;

    // Load skill content
    let skill_content = server.skill_loader
        .load_skill(skill_id, &ctx)
        .await
        .map_err(|e| McpError::internal_error(
            format!("Failed to load skill: {}", e), None
        ))?;

    // Call Gemini with search
    let params = GoogleSearchParams {
        messages: vec![
            AiMessage::system(&skill_content),
            AiMessage::user(&format!("Research: {}", topic)),
        ],
        sampling: None,
        max_output_tokens: 8192,
        model: Some("gemini-2.5-flash"),
        urls: None,
        response_schema: None,
    };

    let response = server.ai_service
        .generate_with_google_search(params)
        .await
        .map_err(|e| McpError::internal_error(
            format!("Research failed: {}", e), None
        ))?;

    // Store artifact
    let artifact_id = store_research_artifact(
        &server.artifact_repo,
        topic,
        &response,
        ctx.context_id().as_str(),
        skill_id,
    ).await.map_err(|e| McpError::internal_error(
        format!("Failed to store artifact: {}", e), None
    ))?;

    // Return result
    Ok(CallToolResult {
        content: vec![McpContent::text(format!(
            "Research complete for '{}'. Found {} sources.\n\n**Artifact ID: {}**",
            topic,
            response.sources.len(),
            artifact_id
        ))],
        structured_content: Some(serde_json::json!({
            "artifact_id": artifact_id,
            "topic": topic,
            "source_count": response.sources.len(),
        })),
        is_error: Some(false),
        meta: None,
    })
}
```

## AI Configuration

Ensure `services/ai/config.yaml` has Gemini configured:

```yaml
ai:
  default_provider: gemini
  default_max_output_tokens: 8192

  providers:
    gemini:
      enabled: true
      api_key: ${GEMINI_API_KEY}
      endpoint: https://generativelanguage.googleapis.com/v1beta
      default_model: gemini-2.5-flash
```

## Environment Variables

| Variable | Description | Required |
|----------|-------------|----------|
| `GEMINI_API_KEY` | Google AI API key | Yes |
| `DATABASE_URL` | PostgreSQL connection | Yes |
| `MCP_PORT` | Server port | No |
| `MCP_SERVICE_ID` | Service identifier | No |
| `SYSTEM_PATH` | SystemPrompt root | Yes |

## Troubleshooting

### "No provider with Google Search support"

Ensure Gemini is enabled in `services/ai/config.yaml` and `GEMINI_API_KEY` is set.

### "Missing tool_provider"

AiService requires a ToolProvider even for non-tool operations. Use `NoopToolProvider::new()`.

### Empty Search Results

Check `response.finish_reason` for "SAFETY" or "RECITATION" which indicate blocked content.

### Artifact Not Found

Artifacts require both `task_id` and `context_id`. Ensure both are provided when creating.