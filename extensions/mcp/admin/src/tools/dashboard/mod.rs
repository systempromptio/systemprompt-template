mod models;
pub mod repository;
mod sections;

use rmcp::{model::{CallToolRequestParam, CallToolResult, Content}, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_system::repository::analytics::CoreStatsRepository;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, ExecutionMetadata, LayoutMode, ToolResponse,
};

use repository::DashboardRepository;
use sections::{
    create_agent_usage_section, create_conversations_overview_section, create_daily_trends_section,
    create_realtime_activity_section, create_recent_conversations_section,
    create_tool_usage_section, create_traffic_summary_section,
};

#[must_use] pub fn dashboard_input_schema() -> JsonValue {
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

#[must_use] pub fn dashboard_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_dashboard(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("24h");

    tracing::debug!(time_range = %time_range, "Generating dashboard");

    let days = match time_range {
        "24h" => 1,
        "7d" => 7,
        "30d" => 30,
        _ => 1,
    };

    let stats_repo = CoreStatsRepository::new(pool.clone());
    let dashboard_repo = DashboardRepository::new(pool.clone())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let mut dashboard = DashboardArtifact::new("System Dashboard")
        .with_description(format!("Comprehensive system metrics ({time_range})"))
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(30)
                .with_drill_down(true),
        );

    let overview = stats_repo
        .get_platform_overview()
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

    let metadata = ExecutionMetadata::new().tool("dashboard");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!("System Dashboard ({time_range})"))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
