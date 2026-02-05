use anyhow::Result;
use rmcp::model::{CallToolRequestParams, CallToolResult};
use rmcp::ErrorData as McpError;
use std::sync::Arc;
use systemprompt::agent::services::SkillService;
use systemprompt::ai::{AiMessage, AiService, GoogleSearchParams};
use systemprompt::database::DbPool;
use systemprompt::identifiers::McpExecutionId;
use systemprompt::mcp::McpResponseBuilder;
use systemprompt::models::artifacts::{
    CardSection, PresentationCardResponse, ResearchArtifact, SourceCitation,
};
use systemprompt::models::execution::context::RequestContext;

use super::helpers::extract_string_array;
use crate::server::ProgressCallback;

const MAX_RETRIES: u32 = 3;

#[allow(clippy::too_many_lines)]
pub async fn handle(
    _db_pool: &DbPool,
    request: CallToolRequestParams,
    ctx: RequestContext,
    ai_service: &Arc<AiService>,
    skill_loader: &SkillService,
    progress: Option<ProgressCallback>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    if let Some(ref notify) = progress {
        notify(0.0, Some(100.0), Some("Starting research...".to_string())).await;
    }

    let args = request.arguments.as_ref().ok_or_else(|| {
        McpError::invalid_request("Missing arguments for research_blog tool", None)
    })?;

    let skill_id = args
        .get("skill_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: skill_id", None))?;

    if skill_id != "research_blog" {
        return Err(McpError::invalid_params(
            "skill_id must be 'research_blog'",
            None,
        ));
    }

    let skill_content = skill_loader.load_skill(skill_id, &ctx).await.map_err(|e| {
        McpError::internal_error(format!("Failed to load skill '{skill_id}': {e}"), None)
    })?;

    let topic = args
        .get("topic")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: topic", None))?;

    let focus_areas = extract_string_array(args, "focus_areas");

    // NOTE: Google Search grounding is a Gemini-only feature.
    // This tool requires Gemini regardless of the configured default_provider.
    if let Some(ref notify) = progress {
        notify(
            10.0,
            Some(100.0),
            Some(
                "Querying Gemini with Google Search (Gemini required for search grounding)..."
                    .to_string(),
            ),
        )
        .await;
    }

    let research_prompt = build_research_prompt(topic, &focus_areas);

    let messages = vec![
        AiMessage::system(&skill_content),
        AiMessage::user(&research_prompt),
    ];

    // Use default Gemini model (configured in ai/config.yaml under providers.gemini.default_model)
    // model: None lets the provider use its configured default
    let search_params = GoogleSearchParams {
        messages,
        sampling: None,
        max_output_tokens: 8192,
        model: None,
        urls: None,
        response_schema: None,
    };

    let search_response = call_with_retry(ai_service, search_params, progress.as_ref()).await?;

    if let Some(ref notify) = progress {
        notify(
            70.0,
            Some(100.0),
            Some("Processing research results...".to_string()),
        )
        .await;
    }

    // Build typed SourceCitation list
    let sources: Vec<SourceCitation> = search_response
        .sources
        .iter()
        .map(|s| SourceCitation::new(&s.title, &s.uri, s.relevance))
        .collect();

    let source_count = sources.len();
    let query_count = search_response.web_search_queries.len();

    // Build PresentationCardResponse for the research artifact
    let card = PresentationCardResponse {
        artifact_type: "presentation_card".to_string(),
        title: format!("Research: {topic}"),
        subtitle: Some(format!("{source_count} sources found")),
        sections: vec![CardSection {
            heading: "Summary".to_string(),
            content: search_response.content.clone(),
            icon: None,
        }],
        ctas: vec![],
        theme: "gradient".to_string(),
        execution_id: Some(mcp_execution_id.to_string()),
        skill_id: Some(skill_id.to_string()),
        skill_name: Some("Blog Research".to_string()),
    };

    let research_artifact =
        ResearchArtifact::new(topic, card, sources.clone()).with_query_count(query_count as u32);

    if let Some(ref notify) = progress {
        notify(100.0, Some(100.0), Some("Research complete".to_string())).await;
    }

    tracing::info!(
        topic = %topic,
        source_count = %source_count,
        "Research completed"
    );

    let summary = format!(
        "Research complete for '{topic}'. Found {source_count} sources and used {query_count} search queries.\n\n\
         Use create_blog_post with the artifact_id from the response metadata to load research findings."
    );

    McpResponseBuilder::new(research_artifact, "research_blog", &ctx, mcp_execution_id)
        .build(summary)
        .map_err(|e| McpError::internal_error(format!("Failed to build response: {e}"), None))
}

fn build_research_prompt(topic: &str, focus_areas: &[String]) -> String {
    let focus_section = if focus_areas.is_empty() {
        String::new()
    } else {
        format!(
            "\n\n**Focus Areas:**\n{}",
            focus_areas
                .iter()
                .map(|a| format!("- {a}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    format!(
        "Research the following topic thoroughly using Google Search:\n\n\
         **Topic:** {topic}{focus_section}\n\n\
         Provide a comprehensive research summary with key insights, recent developments, \
         and important context. Focus on authoritative sources and current information."
    )
}

async fn call_with_retry(
    ai_service: &Arc<AiService>,
    params: GoogleSearchParams<'_>,
    progress: Option<&ProgressCallback>,
) -> Result<systemprompt::ai::SearchGroundedResponse, McpError> {
    let mut last_error = None;

    for attempt in 0..MAX_RETRIES {
        if attempt > 0 {
            let delay_ms = 1000 * 2_u64.pow(attempt - 1);

            if let Some(notify) = progress {
                notify(
                    20.0 + (f64::from(attempt) * 10.0),
                    Some(100.0),
                    Some(format!("Retry attempt {} of {MAX_RETRIES}", attempt + 1)),
                )
                .await;
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms)).await;
        }

        let retry_params = GoogleSearchParams {
            messages: params.messages.clone(),
            sampling: params.sampling.clone(),
            max_output_tokens: params.max_output_tokens,
            model: params.model,
            urls: params.urls.clone(),
            response_schema: params.response_schema.clone(),
        };

        match ai_service.generate_with_google_search(retry_params).await {
            Ok(response) => {
                if response.content.trim().is_empty() {
                    if let Some(reason) = &response.finish_reason {
                        if reason == "SAFETY" || reason == "RECITATION" {
                            return Ok(response);
                        }
                    }
                    last_error = Some("Empty response from Gemini".to_string());
                    continue;
                }

                let has_search_results =
                    !response.sources.is_empty() || !response.web_search_queries.is_empty();

                if !has_search_results {
                    if let Some(reason) = &response.finish_reason {
                        if reason == "SAFETY" || reason == "RECITATION" {
                            return Ok(response);
                        }
                    }
                    last_error = Some("Gemini did not use search grounding".to_string());
                    continue;
                }

                return Ok(response);
            }
            Err(e) => {
                tracing::warn!(attempt = attempt + 1, error = %e, "Gemini attempt failed");
                last_error = Some(e.to_string());
            }
        }
    }

    Err(McpError::internal_error(
        format!(
            "All retry attempts failed: {}",
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ),
        None,
    ))
}
