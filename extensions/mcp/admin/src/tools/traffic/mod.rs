mod models;
pub mod repository;
mod sections;

use rmcp::{model::{CallToolRequestParam, CallToolResult, Content}, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, ExecutionMetadata, LayoutMode, ToolResponse,
};

use repository::TrafficRepository;
use sections::{
    create_browser_breakdown_section, create_device_breakdown_section,
    create_geographic_breakdown_section, create_os_breakdown_section, create_top_referrers_section,
    create_traffic_summary_section,
};

#[must_use] pub fn traffic_input_schema() -> JsonValue {
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

#[must_use] pub fn traffic_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_traffic(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("30d");

    tracing::debug!(time_range = %time_range, "Generating traffic analytics");

    let days = match time_range {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => 30,
    };

    let repo = TrafficRepository::new(pool.clone())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let mut dashboard = DashboardArtifact::new("Website Traffic Analytics")
        .with_description(format!("Traffic metrics for the last {days} days"))
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

    let top_referrers = repo
        .get_normalized_referrers(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !top_referrers.is_empty() {
        dashboard = dashboard.add_section(create_top_referrers_section(&top_referrers));
    }

    let device_breakdown = repo
        .get_device_breakdown_with_trends(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let geographic_breakdown = repo
        .get_geographic_breakdown(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let browser_breakdown = repo
        .get_browser_breakdown(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let os_breakdown = repo
        .get_os_breakdown(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_device_breakdown_section(&device_breakdown));
    dashboard = dashboard.add_section(create_geographic_breakdown_section(&geographic_breakdown));
    dashboard = dashboard.add_section(create_browser_breakdown_section(&browser_breakdown));
    dashboard = dashboard.add_section(create_os_breakdown_section(&os_breakdown));

    let metadata = ExecutionMetadata::new().tool("traffic");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Website Traffic Analytics ({time_range})"
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
