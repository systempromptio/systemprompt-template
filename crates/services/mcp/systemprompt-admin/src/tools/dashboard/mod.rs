mod models;
mod repository;
mod sections;

use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_core_system::repository::analytics::CoreStatsRepository;
use systemprompt_models::artifacts::{DashboardArtifact, DashboardHints, LayoutMode};

use repository::DashboardRepository;
use sections::{
    create_agent_usage_section, create_conversations_overview_section, create_daily_trends_section,
    create_realtime_activity_section, create_recent_conversations_section,
    create_tool_usage_section, create_traffic_summary_section,
};

pub fn dashboard_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "time_range": {
                "type": "string",
                "enum": ["24h", "7d", "30d"],
                "default": "24h",
                "description": "Time range for metrics"
            }
        }
    })
}

pub fn dashboard_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "Unified dashboard with real-time activity, conversations, traffic, trends, and agent/tool usage",
        "properties": {
            "x-artifact-type": {
                "type": "string",
                "enum": ["dashboard"]
            },
            "title": {"type": "string"},
            "sections": {"type": "array"}
        },
        "required": ["x-artifact-type", "sections"]
    })
}

pub async fn handle_dashboard(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("24h");

    logger
        .info(
            "dashboard",
            &format!("Generating dashboard for: {}", time_range),
        )
        .await
        .ok();

    let days = match time_range {
        "24h" => 1,
        "7d" => 7,
        "30d" => 30,
        _ => 1,
    };

    let stats_repo = CoreStatsRepository::new(pool.clone());
    let dashboard_repo = DashboardRepository::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("System Dashboard")
        .with_description(format!("Comprehensive system metrics ({})", time_range))
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(30)
                .with_drill_down(true),
        );

    let overview = stats_repo
        .get_platform_overview(1)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_realtime_activity_section(&overview));

    let conversation_metrics = dashboard_repo
        .get_conversation_metrics()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_conversations_overview_section(&conversation_metrics));

    let recent_conversations = dashboard_repo
        .get_recent_conversations(10)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !recent_conversations.is_empty() {
        dashboard =
            dashboard.add_section(create_recent_conversations_section(&recent_conversations));
    }

    let traffic_summary = dashboard_repo
        .get_traffic_summary(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_traffic_summary_section(&traffic_summary));

    let trends = dashboard_repo
        .get_conversation_trends(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !trends.is_empty() {
        dashboard = dashboard.add_section(create_daily_trends_section(&trends));
    }

    let tool_usage = dashboard_repo
        .get_tool_usage_data()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !tool_usage.agent_data.is_empty() {
        dashboard = dashboard.add_section(create_agent_usage_section(&tool_usage.agent_data));
    }

    if !tool_usage.tool_data.is_empty() {
        dashboard = dashboard.add_section(create_tool_usage_section(&tool_usage.tool_data));
    }

    let response = dashboard.to_response();

    Ok(CallToolResult {
        content: vec![Content::text(format!("System Dashboard ({})", time_range))],
        structured_content: Some(response),
        is_error: Some(false),
        meta: None,
    })
}
