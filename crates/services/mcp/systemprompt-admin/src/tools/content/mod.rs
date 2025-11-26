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
use systemprompt_models::artifacts::{DashboardArtifact, DashboardHints, LayoutMode};

use models::ContentPerformance;
use repository::ContentRepository;
use sections::{
    create_daily_views_chart, create_top_content_section, create_top_referrers_section,
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
    json!({
        "type": "object",
        "description": "Content performance analytics: top content, categories, trends",
        "properties": {
            "title": {"type": "string"},
            "description": {"type": "string"},
            "sections": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "section_id": {"type": "string"},
                        "title": {"type": "string"},
                        "section_type": {
                            "type": "string",
                            "enum": ["metrics_cards", "table", "chart", "list"]
                        },
                        "data": {"type": "object"},
                        "layout": {
                            "type": "object",
                            "properties": {
                                "width": {"type": "string"},
                                "order": {"type": "integer"}
                            }
                        }
                    }
                }
            },
            "mcp_execution_id": {"type": "string"}
        },
        "required": ["title", "sections", "mcp_execution_id"],
        "x-artifact-type": "dashboard"
    })
}

pub async fn handle_content(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("30d");

    logger
        .info(
            "content_tool",
            &format!("Generating content analytics for: {}", time_range),
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

    let daily_views = repo
        .get_daily_views_per_content(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !daily_views.is_empty() {
        dashboard = dashboard.add_section(create_daily_views_chart(&daily_views));
    }

    let mut top_content = repo
        .get_top_content(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    populate_trackable_links(&link_service, &mut top_content).await;

    if !top_content.is_empty() {
        dashboard = dashboard.add_section(create_top_content_section(&top_content));
    }

    let top_referrers = repo
        .get_top_referrers(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !top_referrers.is_empty() {
        dashboard = dashboard.add_section(create_top_referrers_section(&top_referrers));
    }

    let response = dashboard.to_response();
    let text_summary = format_summary(&top_content, time_range);

    Ok(CallToolResult {
        content: vec![Content::text(text_summary)],
        structured_content: Some(response),
        is_error: Some(false),
        meta: None,
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
            "{}. {} ({} views, {} visitors)\n   Trackable: {}\n\n",
            idx + 1,
            item.title,
            item.total_views,
            item.unique_visitors,
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

    Ok(LinkGenerationService::build_trackable_url(&link, REDIRECT_BASE_URL))
}
