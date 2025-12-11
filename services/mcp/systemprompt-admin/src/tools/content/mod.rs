mod models;
mod repository;
mod sections;

use anyhow::Result;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_blog::models::{LinkType, UtmParams};
use systemprompt_core_blog::services::LinkGenerationService;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, ExecutionMetadata, LayoutMode, ToolResponse,
};

use models::ContentPerformance;
use repository::ContentRepository;
use sections::{
    create_daily_views_chart, create_top_content_section, create_top_referrers_section,
    create_traffic_summary_cards,
};

const REDIRECT_BASE_URL: &str = "https://tyingshoelaces.com";

pub fn content_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "time_range": {
                "type": "string",
                "enum": ["7d", "30d", "90d"],
                "default": "30d",
                "description": "Time range for metrics: 7d, 30d, or 90d"
            }
        }
    })
}

pub fn content_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_content(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("30d");

    logger
        .debug(
            "content_tool",
            &format!("Generating analytics | type=content, period={}", time_range),
        )
        .await
        .ok();

    let days = match time_range {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => 30,
    };

    let repo = ContentRepository::new(pool.clone());
    let link_service = LinkGenerationService::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("Content Analytics").with_hints(
        DashboardHints::new()
            .with_layout(LayoutMode::Vertical)
            .with_refreshable(true)
            .with_refresh_interval(300)
            .with_drill_down(true),
    );

    let traffic_summary = repo
        .get_traffic_summary()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_traffic_summary_cards(&traffic_summary));

    let daily_views = repo
        .get_daily_views_per_content(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !daily_views.is_empty() {
        dashboard = dashboard.add_section(create_daily_views_chart(&daily_views));
    }

    let mut top_content = repo
        .get_top_content_by_7d(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    populate_trackable_links(&link_service, &mut top_content).await;

    if !top_content.is_empty() {
        dashboard = dashboard.add_section(create_top_content_section(&top_content));
    }

    let top_referrers = repo
        .get_normalized_referrers(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !top_referrers.is_empty() {
        dashboard = dashboard.add_section(create_top_referrers_section(&top_referrers));
    }

    let metadata = ExecutionMetadata::new().tool("content");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );
    let text_summary = format_summary(&top_content, time_range);

    Ok(CallToolResult {
        content: vec![Content::text(text_summary)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

fn format_summary(top_content: &[ContentPerformance], time_range: &str) -> String {
    let mut summary = format!("Content Analytics ({})\n\n", time_range);

    if top_content.is_empty() {
        summary.push_str("No content data available for the selected time range.");
        return summary;
    }

    summary.push_str("Top Performing Content\n");
    summary.push_str(&"─".repeat(50));
    summary.push('\n');

    for (idx, item) in top_content.iter().take(5).enumerate() {
        summary.push_str(&format!(
            "{}. {} ({} views, {} visitors - 1d: {}, 7d: {}, 30d: {})\n   Trackable: {}\n\n",
            idx + 1,
            item.title,
            item.total_views,
            item.visitors_all_time,
            item.visitors_1d,
            item.visitors_7d,
            item.visitors_30d,
            item.trackable_url
        ));
    }

    if top_content.len() > 5 {
        summary.push_str(&format!(
            "   ... and {} more items\n\n",
            top_content.len() - 5
        ));
    }

    summary
}

async fn populate_trackable_links(
    link_service: &LinkGenerationService,
    content_items: &mut [ContentPerformance],
) {
    for item in content_items {
        if let Ok(link) = generate_trackable_link(link_service, item).await {
            item.trackable_url = link;
        } else {
            item.trackable_url = item.preview_url.clone();
        }
    }
}

async fn generate_trackable_link(
    link_service: &LinkGenerationService,
    item: &ContentPerformance,
) -> Result<String> {
    let utm_params = UtmParams {
        source: Some("admin_dashboard".to_string()),
        medium: Some("content_tool".to_string()),
        campaign: Some("content_analytics".to_string()),
        term: None,
        content: Some(item.content_id.clone()),
    };

    let link = link_service
        .generate_link(
            &item.preview_url,
            LinkType::Both,
            Some("admin_content_preview".to_string()),
            Some("Admin Content Preview".to_string()),
            Some(item.content_id.clone()),
            Some("top_content".to_string()),
            Some(utm_params),
            None,
            Some("preview".to_string()),
            None,
        )
        .await?;

    Ok(LinkGenerationService::build_trackable_url(
        &link,
        REDIRECT_BASE_URL,
    ))
}
