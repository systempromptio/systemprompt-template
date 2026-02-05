use anyhow::Result;
use rmcp::model::{CallToolRequestParams, CallToolResult, Meta, Tool};
use rmcp::ErrorData as McpError;
use std::sync::Arc;

pub const SERVER_NAME: &str = "content-manager";

fn create_ui_meta() -> Meta {
    Meta(tool_ui_meta(SERVER_NAME, &default_tool_visibility()))
}
use systemprompt::agent::repository::content::ArtifactRepository;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiService, ImageService};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::{default_tool_visibility, tool_ui_meta, McpResponseBuilder};
use systemprompt::models::execution::context::RequestContext;

pub mod create_blog_post;
pub mod generate_featured_image;
pub mod research_blog;
pub mod shared;

use crate::server::ProgressCallback;

#[must_use]
pub fn list_tools() -> Vec<Tool> {
    vec![
        create_tool(
            "research_blog",
            "Research Blog Topic",
            "Research a topic using Google Search with AI-powered analysis. Returns an artifact_id \
             that must be passed to create_blog_post. skill_id must be 'research_blog'. \
             Call once per topic - do not call again unless user explicitly requests additional research.",
            &research_blog::input_schema(),
            &research_blog::output_schema(),
        ),
        create_tool(
            "create_blog_post",
            "Create Blog Post",
            "Create a blog post using AI generation from research. Requires artifact_id (UUID from \
             research_blog), skill_id ('blog_writing' or 'technical_content_writing'), slug, \
             description, keywords, and instructions. Returns blog content with content_id (UUID).",
            &create_blog_post::input_schema(),
            &create_blog_post::output_schema(),
        ),
        create_tool(
            "generate_featured_image",
            "Generate Featured Image",
            "Generate a striking featured image for a blog post. Requires skill_id 'blog_image_generation', \
             topic, title, and summary. Optional: style_hints, aspect_ratio (1:1, 16:9, 4:3). \
             Returns image_id and public_url for use in blog frontmatter.",
            &generate_featured_image::input_schema(),
            &generate_featured_image::output_schema(),
        ),
    ]
}

fn create_tool(
    name: &str,
    title: &str,
    description: &str,
    input_schema: &serde_json::Value,
    output_schema: &serde_json::Value,
) -> Tool {
    let input_obj = input_schema.as_object().cloned().expect("schema is object");
    let output_obj = output_schema
        .as_object()
        .cloned()
        .expect("schema is object");

    Tool {
        name: name.to_string().into(),
        title: Some(title.to_string()),
        description: Some(description.to_string().into()),
        input_schema: Arc::new(input_obj),
        output_schema: Some(Arc::new(output_obj)),
        annotations: None,
        icons: None,
        meta: Some(create_ui_meta()),
    }
}

#[allow(clippy::too_many_arguments)]
pub async fn handle_tool_call(
    name: &str,
    request: CallToolRequestParams,
    ctx: RequestContext,
    db_pool: &DbPool,
    ai_service: &Arc<AiService>,
    image_service: &Arc<ImageService>,
    skill_loader: &SkillService,
    artifact_repo: &ArtifactRepository,
    progress: Option<ProgressCallback>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    match name {
        "research_blog" => {
            research_blog::handle(
                db_pool,
                request,
                ctx,
                ai_service,
                skill_loader,
                progress,
                mcp_execution_id,
            )
            .await
        }
        "create_blog_post" => {
            create_blog_post::handle(
                db_pool,
                request,
                ctx,
                ai_service,
                skill_loader,
                artifact_repo,
                progress,
                mcp_execution_id,
            )
            .await
        }
        "generate_featured_image" => {
            generate_featured_image::handle(
                db_pool,
                request,
                ctx,
                image_service,
                skill_loader,
                progress,
                mcp_execution_id,
            )
            .await
        }
        _ => Ok(McpResponseBuilder::<()>::build_error(format!(
            "Unknown tool: '{name}'\n\n\
            Available tools: research_blog, create_blog_post, generate_featured_image"
        ))),
    }
}
