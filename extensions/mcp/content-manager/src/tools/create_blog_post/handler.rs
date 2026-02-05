use anyhow::Result;
use chrono::Utc;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::ErrorData as McpError;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiMessage, AiRequest, AiService};
use systemprompt::database::DbPool;
use systemprompt::identifiers::{ArtifactId, McpExecutionId, SourceId};
use systemprompt::mcp::McpResponseBuilder;
use systemprompt::models::a2a::{Artifact, DataPart, Part};
use systemprompt::models::artifacts::TextArtifact;
use systemprompt::models::execution::context::RequestContext;
use systemprompt_web_extension::{ContentKind, ContentRepository, CreateContentParams};

use super::helpers::{build_user_prompt, extract_title};
use crate::server::ProgressCallback;
use crate::tools::shared::extract_string_array;

#[allow(clippy::too_many_lines)]
pub async fn handle(
    db_pool: &DbPool,
    request: CallToolRequestParams,
    ctx: RequestContext,
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    artifact_repo: &ArtifactRepository,
    progress: Option<ProgressCallback>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let pg_pool = db_pool
        .pool()
        .ok_or_else(|| McpError::internal_error("Database pool not available", None))?;
    let content_repo = ContentRepository::new(pg_pool);

    if let Some(ref notify) = progress {
        notify(
            0.0,
            Some(100.0),
            Some("Starting blog generation...".to_string()),
        )
        .await;
    }

    let args = request
        .arguments
        .ok_or_else(|| McpError::invalid_params("Missing arguments", None))?;

    let skill_id = args
        .get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: skill_id", None))?;

    let valid_skills = [
        "blog_writing",
        "technical_content_writing",
        "announcement_writing",
        "guide_writing",
    ];
    if !valid_skills.contains(&skill_id) {
        return Err(McpError::invalid_params(
            "skill_id must be one of: 'blog_writing', 'technical_content_writing', 'announcement_writing', 'guide_writing'",
            None,
        ));
    }

    // Extract category (default to "article")
    let category = args
        .get("category")
        .and_then(|v| v.as_str())
        .unwrap_or("article")
        .to_string();

    let content_skill = skill_loader.load_skill(skill_id, &ctx).await.map_err(|e| {
        McpError::internal_error(format!("Failed to load skill '{skill_id}': {e}"), None)
    })?;

    let voice_skill = skill_loader
        .load_skill("edwards_voice", &ctx)
        .await
        .unwrap_or_else(|_| String::new());

    if let Some(ref notify) = progress {
        notify(10.0, Some(100.0), Some("Skills loaded...".to_string())).await;
    }

    // artifact_id is optional for announcements (can be empty string or omitted)
    let artifact_id_str = args
        .get("artifact_id")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    let research_data = if artifact_id_str.is_empty() {
        // No research artifact - create empty research data (for announcements)
        if let Some(ref notify) = progress {
            notify(
                15.0,
                Some(100.0),
                Some("Skipping research (no artifact provided)...".to_string()),
            )
            .await;
        }
        ResearchData {
            summary: String::new(),
            sources: Vec::new(),
        }
    } else {
        // Load research artifact
        let artifact_id = ArtifactId::new(artifact_id_str);

        if let Some(ref notify) = progress {
            notify(
                15.0,
                Some(100.0),
                Some("Loading research artifact...".to_string()),
            )
            .await;
        }

        let research_artifact = artifact_repo
            .get_artifact_by_id(&artifact_id)
            .await
            .map_err(|e| McpError::internal_error(format!("Failed to query artifact: {e}"), None))?
            .ok_or_else(|| {
                McpError::invalid_params(
                    format!(
                        "Research artifact not found: {artifact_id_str}. Use artifact_id from research_blog."
                    ),
                    None,
                )
            })?;

        extract_research_data(&research_artifact)?
    };

    let slug = args
        .get("slug")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: slug", None))?
        .to_string();

    let description = args
        .get("description")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: description", None))?
        .to_string();

    let keywords = extract_string_array(&args, "keywords");

    let instructions = args
        .get("instructions")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            McpError::invalid_params("Missing required parameter: instructions", None)
        })?;

    if let Some(ref notify) = progress {
        notify(20.0, Some(100.0), Some("Building AI prompt...".to_string())).await;
    }

    let system_prompt = if voice_skill.is_empty() {
        content_skill.clone()
    } else {
        format!("{voice_skill}\n\n---\n\n{content_skill}")
    };

    let user_prompt =
        build_user_prompt(&research_data.summary, &research_data.sources, instructions);

    let messages = vec![
        AiMessage::system(&system_prompt),
        AiMessage::user(&user_prompt),
    ];

    if let Some(ref notify) = progress {
        notify(
            30.0,
            Some(100.0),
            Some("Generating blog content with AI...".to_string()),
        )
        .await;
    }

    // Use configured default provider and model from ai/config.yaml
    // Use 4096 max tokens for cross-provider compatibility (OpenAI limit)
    let request = AiRequest::builder(
        messages,
        ai_service.default_provider(),
        ai_service.default_model(),
        4096,
        ctx.clone(),
    )
    .build();

    let response = ai_service.generate(&request).await.map_err(|e| {
        McpError::internal_error(format!("Failed to generate blog content: {e}"), None)
    })?;

    let mut generated_content = response.content;

    if generated_content.starts_with("---") {
        if let Some(end_idx) = generated_content[3..].find("---") {
            generated_content = generated_content[end_idx + 6..].trim().to_string();
        }
    }

    let title = extract_title(&generated_content);
    let word_count = generated_content.split_whitespace().count();

    if let Some(ref notify) = progress {
        notify(
            80.0,
            Some(100.0),
            Some("Saving blog post to database...".to_string()),
        )
        .await;
    }

    // Generate content hash for deduplication
    let mut hasher = Sha256::new();
    hasher.update(generated_content.as_bytes());
    let version_hash = format!("{:x}", hasher.finalize());

    // Convert research sources to links for the references section
    let links: serde_json::Value = serde_json::to_value(
        research_data
            .sources
            .iter()
            .map(|(title, url)| {
                serde_json::json!({
                    "title": title,
                    "url": url
                })
            })
            .collect::<Vec<_>>(),
    )
    .expect("JSON serialization cannot fail for Vec");

    // Create content params for database
    let content_params = CreateContentParams::new(
        slug.clone(),
        title.clone(),
        description.clone(),
        generated_content.clone(),
        "Edward".to_string(), // Author
        Utc::now(),
        SourceId::new("blog".to_string()),
    )
    .with_keywords(keywords.join(", "))
    .with_kind(ContentKind::Blog)
    .with_category(Some(category.clone()))
    .with_version_hash(version_hash)
    .with_links(links);

    // Save to database
    let content = content_repo.create(&content_params).await.map_err(|e| {
        tracing::error!(error = %e, slug = %slug, "Failed to save blog post to database");
        McpError::internal_error(format!("Failed to save blog post to database: {e}"), None)
    })?;

    let blog_content_id = content.id.to_string();

    if let Some(ref notify) = progress {
        notify(
            100.0,
            Some(100.0),
            Some("Blog post saved to database!".to_string()),
        )
        .await;
    }

    tracing::info!(
        title = %title,
        slug = %slug,
        content_id = %blog_content_id,
        word_count = %word_count,
        "Created and saved blog post to database"
    );

    // Use TextArtifact for the blog post content
    let artifact = TextArtifact::new(&generated_content, &ctx)
        .with_title(&title)
        .with_skill(skill_id, skill_name(skill_id));

    let summary = format!(
        "SAVED blog post '{title}' to database\n\n\
         Content ID: {blog_content_id}\n\
         Slug: {slug}\n\
         Category: {category}\n\
         URL: /blog/{slug}\n\
         Word count: {word_count}\n\
         Skill: {} ({})\n\n\
         Content Preview:\n{}\n\n\
         NEXT STEP: Run `infra jobs run publish_pipeline` to publish the content to the live site.",
        skill_name(skill_id),
        skill_id,
        &generated_content.chars().take(500).collect::<String>()
    );

    McpResponseBuilder::new(artifact, "create_blog_post", &ctx, mcp_execution_id)
        .build(summary)
        .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
}

struct ResearchData {
    summary: String,
    sources: Vec<(String, String)>,
}

fn extract_research_data(artifact: &Artifact) -> Result<ResearchData, McpError> {
    for part in &artifact.parts {
        if let Part::Data(DataPart { data }) = part {
            let summary = data
                .get("summary")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string();

            let sources: Vec<(String, String)> = data
                .get("sources")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|s| {
                            let title = s.get("title")?.as_str()?;
                            let uri = s.get("uri")?.as_str()?;
                            Some((title.to_string(), uri.to_string()))
                        })
                        .collect()
                })
                .unwrap_or_else(Vec::new);

            return Ok(ResearchData { summary, sources });
        }
    }

    Err(McpError::invalid_params(
        "Invalid research artifact format. Re-run research_blog to generate a new artifact.",
        None,
    ))
}

fn skill_name(skill_id: &str) -> &str {
    match skill_id {
        "blog_writing" => "Blog Writing",
        "technical_content_writing" => "Technical Content Writing",
        "announcement_writing" => "Announcement Writing",
        "guide_writing" => "Guide Writing",
        _ => skill_id,
    }
}
