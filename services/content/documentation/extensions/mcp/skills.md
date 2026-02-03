---
title: "MCP Skill Integration"
description: "Loading and using skills in MCP servers for AI-powered tool implementations."
author: "SystemPrompt Team"
slug: "extensions/mcp/skills"
keywords: "mcp, skills, prompts, ai, llm, generation"
image: "/files/images/docs/mcp-skills.svg"
kind: "reference"
public: true
tags: []
published_at: "2026-02-02"
updated_at: "2026-02-02"
---

# MCP Skill Integration

Skills are reusable prompts stored in the database that guide AI behavior. MCP servers can load skills to provide consistent, high-quality AI-powered tool implementations.

## What Are Skills?

Skills are prompt templates that define:

- **Voice** — Writing style and tone
- **Instructions** — What the AI should do
- **Constraints** — Rules and limitations
- **Output format** — Expected response structure

Skills are stored in `services/skills/*.yaml` and synced to the database.

## Setting Up SkillService

Initialize the SkillService in your server:

```rust
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::database::DbPool;

#[derive(Clone)]
pub struct MyServer {
    db_pool: DbPool,
    service_id: McpServerId,
    skill_loader: Arc<SkillService>,
}

impl MyServer {
    pub async fn new(db_pool: DbPool, service_id: McpServerId) -> Result<Self> {
        let skill_loader = Arc::new(
            SkillService::new(db_pool.clone()).await?
        );

        Ok(Self {
            db_pool,
            service_id,
            skill_loader,
        })
    }
}
```

## Loading Skills

Load a skill by its ID:

```rust
use systemprompt::agent::services::SkillService;
use systemprompt::models::execution::context::RequestContext;

pub async fn handle(
    skill_loader: &SkillService,
    ctx: &RequestContext,
    skill_id: &str,
) -> Result<CallToolResult, McpError> {
    // Load skill content
    let skill_content = skill_loader
        .load_skill(skill_id, ctx)
        .await
        .map_err(|e| {
            McpError::internal_error(
                format!("Failed to load skill '{skill_id}': {e}"),
                None,
            )
        })?;

    // Use skill_content as system prompt
    let messages = vec![
        AiMessage::system(&skill_content),
        AiMessage::user(&user_prompt),
    ];

    // Call AI service...
}
```

## Skill ID Validation

Validate that the tool is called with the correct skill:

```rust
let skill_id = args
    .get("skill_id")
    .and_then(|v| v.as_str())
    .ok_or_else(|| {
        McpError::invalid_params("Missing required parameter: skill_id", None)
    })?;

// Validate specific skill
if skill_id != "research_blog" {
    return Err(McpError::invalid_params(
        format!("skill_id must be 'research_blog', got '{skill_id}'"),
        None,
    ));
}

// Or validate against allowed list
let allowed_skills = ["blog_writing", "technical_content_writing"];
if !allowed_skills.contains(&skill_id) {
    return Err(McpError::invalid_params(
        format!(
            "Invalid skill_id: '{skill_id}'. Must be one of: {}",
            allowed_skills.join(", ")
        ),
        None,
    ));
}
```

## Using Skills with AI Service

### Basic Generation

```rust
use systemprompt::ai::{AiMessage, AiService};

pub async fn generate_content(
    ai_service: &Arc<AiService>,
    skill_content: &str,
    user_prompt: &str,
) -> Result<String, McpError> {
    let messages = vec![
        AiMessage::system(skill_content),
        AiMessage::user(user_prompt),
    ];

    let response = ai_service
        .generate(&messages, None)
        .await
        .map_err(|e| McpError::internal_error(format!("AI error: {e}"), None))?;

    Ok(response.content)
}
```

### Google Search Grounding

For research tools that need web search:

```rust
use systemprompt::ai::{AiMessage, AiService, GoogleSearchParams};

pub async fn research_with_search(
    ai_service: &Arc<AiService>,
    skill_content: &str,
    topic: &str,
) -> Result<SearchGroundedResponse, McpError> {
    let messages = vec![
        AiMessage::system(skill_content),
        AiMessage::user(&format!("Research the topic: {topic}")),
    ];

    let params = GoogleSearchParams {
        messages,
        sampling: None,
        max_output_tokens: 8192,
        model: Some("gemini-2.0-flash"),
        urls: None,
        response_schema: None,
    };

    let response = ai_service
        .generate_with_google_search(params)
        .await
        .map_err(|e| McpError::internal_error(format!("Search error: {e}"), None))?;

    // response.content - Generated text
    // response.sources - Vec<WebSource> with title, uri, relevance
    // response.web_search_queries - Queries used

    Ok(response)
}
```

### Structured Output

Generate JSON that matches a schema:

```rust
use systemprompt::ai::{AiMessage, AiService, StructuredOutputParams};
use serde::Deserialize;

#[derive(Deserialize)]
struct BlogOutline {
    title: String,
    sections: Vec<Section>,
}

#[derive(Deserialize)]
struct Section {
    heading: String,
    key_points: Vec<String>,
}

pub async fn generate_outline(
    ai_service: &Arc<AiService>,
    skill_content: &str,
    topic: &str,
) -> Result<BlogOutline, McpError> {
    let schema = serde_json::json!({
        "type": "object",
        "properties": {
            "title": {"type": "string"},
            "sections": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "heading": {"type": "string"},
                        "key_points": {
                            "type": "array",
                            "items": {"type": "string"}
                        }
                    },
                    "required": ["heading", "key_points"]
                }
            }
        },
        "required": ["title", "sections"]
    });

    let params = StructuredOutputParams {
        messages: vec![
            AiMessage::system(skill_content),
            AiMessage::user(&format!("Create an outline for: {topic}")),
        ],
        schema,
        model: None,
    };

    let outline: BlogOutline = ai_service
        .generate_structured(params)
        .await
        .map_err(|e| McpError::internal_error(format!("AI error: {e}"), None))?;

    Ok(outline)
}
```

## Combining Skills

### Voice + Content Skills

Combine a voice skill with a content skill:

```rust
pub async fn load_combined_skills(
    skill_loader: &SkillService,
    ctx: &RequestContext,
    voice_skill_id: &str,
    content_skill_id: &str,
) -> Result<String, McpError> {
    let voice_skill = skill_loader
        .load_skill(voice_skill_id, ctx)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let content_skill = skill_loader
        .load_skill(content_skill_id, ctx)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    // Combine with separator
    Ok(format!("{voice_skill}\n\n---\n\n{content_skill}"))
}

// Usage
let combined = load_combined_skills(
    skill_loader,
    &ctx,
    "edwards_voice",      // Voice/style skill
    "linkedin_post_writing"  // Platform skill
).await?;
```

### Platform-Specific Skills

Map platforms to skill IDs:

```rust
pub fn get_skill_for_platform(platform: &str) -> Result<&'static str, McpError> {
    match platform {
        "linkedin" => Ok("linkedin_post_writing"),
        "twitter" => Ok("twitter_post_writing"),
        "reddit" => Ok("reddit_post_writing"),
        "medium" => Ok("medium_article_writing"),
        _ => Err(McpError::invalid_params(
            format!("Unsupported platform: {platform}"),
            None,
        )),
    }
}

// Usage
let platform = args.get("platform").and_then(|v| v.as_str()).unwrap();
let skill_id = get_skill_for_platform(platform)?;
let skill_content = skill_loader.load_skill(skill_id, &ctx).await?;
```

## Skill-Driven Pipelines

### Research → Create Pipeline

```rust
// Step 1: Research with research_blog skill
pub async fn research_topic(
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    ctx: &RequestContext,
    topic: &str,
) -> Result<ResearchResult, McpError> {
    let skill = skill_loader.load_skill("research_blog", ctx).await?;

    let params = GoogleSearchParams {
        messages: vec![
            AiMessage::system(&skill),
            AiMessage::user(&format!("Research: {topic}")),
        ],
        // ...
    };

    let response = ai_service.generate_with_google_search(params).await?;

    Ok(ResearchResult {
        summary: response.content,
        sources: response.sources,
    })
}

// Step 2: Create content with blog_writing skill
pub async fn create_blog_post(
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    ctx: &RequestContext,
    research: &ResearchResult,
    instructions: &str,
) -> Result<BlogPost, McpError> {
    let skill = skill_loader.load_skill("blog_writing", ctx).await?;

    let prompt = format!(
        "Create a blog post based on this research:\n\n\
         {}\n\n\
         Sources:\n{}\n\n\
         Instructions: {}",
        research.summary,
        format_sources(&research.sources),
        instructions
    );

    // Generate structured blog post...
}
```

## Error Handling

Handle skill loading failures gracefully:

```rust
let skill_content = match skill_loader.load_skill(skill_id, &ctx).await {
    Ok(content) => content,
    Err(e) => {
        tracing::error!(
            skill_id = %skill_id,
            error = %e,
            "Failed to load skill"
        );
        return Err(McpError::internal_error(
            format!("Skill '{skill_id}' not found or failed to load: {e}"),
            None,
        ));
    }
};
```

## Skill Attribution

Track which skill was used in artifacts:

```rust
let metadata = ArtifactMetadata::new(
    "blog_artifact".to_string(),
    context_id.clone(),
    task_id.clone(),
)
.with_tool_name("create_blog_post".to_string())
.with_skill(skill_id.to_string(), "Blog Writing".to_string());
```

Include in response:

```rust
Ok(CallToolResult {
    content: vec![Content::text("...")],
    structured_content: Some(json!({
        "content_id": content_id,
        "skill_id": skill_id,
        "skill_name": "Blog Writing",
        "status": "completed"
    })),
    is_error: Some(false),
    meta: None,
})
```

## Skill File Format

Skills are defined in YAML:

```yaml
# services/skills/blog_writing.yaml
name: blog_writing
display_name: "Blog Writing"
description: "Write engaging blog posts"
category: content
content: |
  You are a skilled blog writer. Your task is to create engaging,
  well-structured blog posts that inform and captivate readers.

  ## Guidelines

  - Write in a conversational but professional tone
  - Use clear headings and subheadings
  - Include practical examples
  - End with actionable takeaways

  ## Output Format

  Return the blog post in markdown format with proper headings.
```

Sync skills to database:

```bash
systemprompt core skills sync --direction to-db -y
```

## Quick Reference

| Task | Code |
|------|------|
| Initialize service | `SkillService::new(db_pool).await` |
| Load skill | `skill_loader.load_skill(id, &ctx).await` |
| Use in prompt | `AiMessage::system(&skill_content)` |
| Validate skill ID | Check against allowed list |
| Combine skills | Join with `"\n\n---\n\n"` separator |
| Track in artifact | `.with_skill(id, name)` |