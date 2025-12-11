mod models;
mod repository;
mod sections;

use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, ExecutionMetadata, LayoutMode, ToolResponse,
};

use repository::LogsRepository;
use sections::{create_logs_table_section, create_stats_section};

pub fn logs_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "page": {
                "type": "integer",
                "default": 0,
                "description": "Page number for pagination (0-indexed)"
            },
            "limit": {
                "type": "integer",
                "default": 1000,
                "description": "Number of log entries per page (default 1000)"
            },
            "level": {
                "type": "string",
                "description": "Filter logs by level (INFO, WARN, ERROR, DEBUG)",
                "enum": ["INFO", "WARN", "ERROR", "DEBUG"]
            }
        }
    })
}

pub fn logs_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_logs(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let page = args.get("page").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(1000) as i32;
    let level = args.get("level").and_then(|v| v.as_str()).map(String::from);

    logger
        .debug(
            "logs_tool",
            &format!(
                "Fetching logs - page: {}, limit: {}, level: {:?}",
                page, limit, level
            ),
        )
        .await
        .ok();

    let repo = LogsRepository::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("System Logs")
        .with_description("Latest system logs with pagination and server-side level filtering")
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(30),
        );

    let logs = repo
        .fetch_recent_logs(page, limit, level.as_deref())
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let stats = repo
        .fetch_log_stats()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_stats_section(&stats));
    dashboard = dashboard.add_section(create_logs_table_section(&logs, page));

    let metadata = ExecutionMetadata::new().tool("logs");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "System Logs (Page {} of {}, {} per page)",
            page,
            (stats.total_logs as f64 / limit as f64).ceil() as i32,
            limit
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
