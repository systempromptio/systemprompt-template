mod models;
mod repository;
mod sections;

use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_models::artifacts::{DashboardArtifact, DashboardHints, LayoutMode};

use repository::TrafficRepository;
use sections::{create_traffic_summary_section, create_traffic_table_section};

pub fn traffic_input_schema() -> JsonValue {
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

pub fn traffic_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "Website traffic analytics: requests, visitors, devices, geolocation, and clients",
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

pub async fn handle_traffic(
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
            "traffic_tool",
            &format!("Generating traffic analytics for: {}", time_range),
        )
        .await
        .ok();

    let days = match time_range {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => 30,
    };

    let repo = TrafficRepository::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("Website Traffic Analytics")
        .with_description(format!("Traffic metrics for the last {} days", days))
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(60)
                .with_drill_down(true),
        );

    let traffic_summary = repo
        .get_traffic_summary(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_traffic_summary_section(&traffic_summary));

    let device_breakdown = repo
        .get_device_breakdown(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let traffic_sources = repo
        .get_traffic_sources(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let landing_pages = repo
        .get_landing_pages(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_traffic_table_section(
        &traffic_sources,
        &landing_pages,
        &device_breakdown,
    ));

    let response = dashboard.to_response();

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Website Traffic Analytics ({})\n\n{}",
            time_range,
            serde_json::to_string_pretty(&response).unwrap_or_default()
        ))],
        structured_content: Some(response),
        is_error: Some(false),
        meta: None,
    })
}
