use anyhow::Result;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AspectRatio, ImageGenerationRequest, ImageResolution, ImageService};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpResponseBuilder;
use systemprompt::models::artifacts::ImageArtifact;
use systemprompt::models::execution::context::RequestContext;

use super::helpers::build_image_prompt;
use crate::server::ProgressCallback;

const MAX_RETRIES: u32 = 2;

#[allow(clippy::too_many_arguments)]
pub async fn handle(
    _db_pool: &DbPool,
    request: CallToolRequestParams,
    ctx: RequestContext,
    image_service: &Arc<ImageService>,
    skill_loader: &SkillService,
    progress: Option<ProgressCallback>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    if let Some(ref notify) = progress {
        notify(
            0.0,
            Some(100.0),
            Some("Starting image generation...".to_string()),
        )
        .await;
    }

    let args = request.arguments.as_ref().ok_or_else(|| {
        McpError::invalid_request("Missing arguments for generate_featured_image tool", None)
    })?;

    let skill_id = args
        .get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: skill_id", None))?;

    if skill_id != "blog_image_generation" {
        return Err(McpError::invalid_params(
            "skill_id must be 'blog_image_generation'",
            None,
        ));
    }

    let skill_content = skill_loader.load_skill(skill_id, &ctx).await.map_err(|e| {
        McpError::internal_error(format!("Failed to load skill '{skill_id}': {e}"), None)
    })?;

    if let Some(ref notify) = progress {
        notify(10.0, Some(100.0), Some("Skill loaded...".to_string())).await;
    }

    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: topic", None))?;

    let title = args
        .get("title")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: title", None))?;

    let summary = args
        .get("summary")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: summary", None))?;

    let style_hints = args.get("style_hints").and_then(|v| v.as_str());

    let aspect_ratio = args
        .get("aspect_ratio")
        .and_then(|v| v.as_str())
        .map(|s| match s {
            "1:1" => AspectRatio::Square,
            "4:3" => AspectRatio::Landscape43,
            _ => AspectRatio::Landscape169,
        })
        .unwrap_or(AspectRatio::Landscape169);

    if let Some(ref notify) = progress {
        notify(
            20.0,
            Some(100.0),
            Some("Building image prompt...".to_string()),
        )
        .await;
    }

    let prompt = build_image_prompt(&skill_content, topic, title, summary, style_hints);

    // Query provider capabilities to select best supported resolution
    let resolution = select_best_resolution(image_service);

    let image_request = ImageGenerationRequest {
        prompt,
        model: None,
        resolution,
        aspect_ratio,
        reference_images: vec![],
        enable_search_grounding: false,
        user_id: Some(ctx.user_id().to_string()),
        session_id: Some(ctx.session_id().to_string()),
        trace_id: Some(ctx.trace_id().to_string()),
        mcp_execution_id: Some(mcp_execution_id.to_string()),
    };

    // Note: Image generation uses the configured default image provider (Gemini or OpenAI)
    // Anthropic does not support image generation
    if let Some(ref notify) = progress {
        notify(
            30.0,
            Some(100.0),
            Some("Generating image with configured provider...".to_string()),
        )
        .await;
    }

    let response = generate_with_retry(image_service, image_request, progress.as_ref()).await?;

    if let Some(ref notify) = progress {
        notify(
            90.0,
            Some(100.0),
            Some("Image generated successfully...".to_string()),
        )
        .await;
    }

    if let Some(ref notify) = progress {
        notify(
            100.0,
            Some(100.0),
            Some("Featured image ready!".to_string()),
        )
        .await;
    }

    tracing::info!(
        image_id = %response.id,
        public_url = ?response.public_url,
        generation_time_ms = %response.generation_time_ms,
        "Generated featured image"
    );

    let public_url = response
        .public_url
        .clone()
        .ok_or_else(|| McpError::internal_error("Image generation returned no public URL", None))?;

    // Use ImageArtifact for the generated image
    let artifact = ImageArtifact::new(&public_url, &ctx)
        .with_alt(format!("Featured image for: {}", title))
        .with_caption(format!("Generated for blog post: {}", title))
        .with_skill("blog_image_generation", "Blog Image Generation");

    let summary = format!(
        "Generated featured image for '{title}'\n\n\
         Image ID: {}\n\
         Public URL: {}\n\
         Resolution: {}\n\
         Aspect Ratio: {}\n\
         Generation Time: {}ms\n\n\
         Use this image_id or public_url in your blog post frontmatter.",
        response.id,
        public_url,
        response.resolution.as_str(),
        response.aspect_ratio.as_str(),
        response.generation_time_ms
    );

    McpResponseBuilder::new(artifact, "generate_featured_image", &ctx, mcp_execution_id)
        .build(summary)
        .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
}

async fn generate_with_retry(
    image_service: &Arc<ImageService>,
    request: ImageGenerationRequest,
    progress: Option<&ProgressCallback>,
) -> Result<systemprompt::ai::ImageGenerationResponse, McpError> {
    let mut last_error = None;

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = 2000 * 2_u64.pow(attempt - 1);

            if let Some(notify) = progress {
                notify(
                    40.0 + (f64::from(attempt) * 15.0),
                    Some(100.0),
                    Some(format!("Retry attempt {} of {MAX_RETRIES}", attempt + 1)),
                )
                .await;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }

        match image_service.generate_image(request.clone()).await {
            Ok(response) => return Ok(response),
            Err(e) => {
                tracing::warn!(attempt = attempt + 1, error = %e, "Image generation attempt failed");
                last_error = Some(e.to_string());
            }
        }
    }

    Err(McpError::internal_error(
        format!(
            "Image generation failed after {MAX_RETRIES} attempts: {}",
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ),
        None,
    ))
}

/// Select the best resolution supported by the default image provider.
/// Prefers higher resolutions (4K > 2K > 1K) when available.
fn select_best_resolution(image_service: &Arc<ImageService>) -> ImageResolution {
    image_service
        .default_provider_capabilities()
        .and_then(|caps| {
            // Prefer highest resolution available
            [
                ImageResolution::FourK,
                ImageResolution::TwoK,
                ImageResolution::OneK,
            ]
            .into_iter()
            .find(|r| caps.supported_resolutions.contains(r))
        })
        .unwrap_or(ImageResolution::OneK)
}
