mod models;
mod repository;
mod sections;

use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_models::artifacts::{DashboardArtifact, DashboardHints, LayoutMode};

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
                "default": 50,
                "description": "Number of log entries per page (default 50)"
            }
        }
    })
}

pub fn logs_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "System logs with pagination",
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

pub async fn handle_logs(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let page = args.get("page").and_then(|v| v.as_i64()).unwrap_or(0) as i32;
    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(50) as i32;

    logger
        .info(
            "logs_tool",
            &format!("Fetching logs - page: {}, limit: {}", page, limit),
        )
        .await
        .ok();

    let repo = LogsRepository::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("System Logs")
        .with_description("Latest system logs with pagination and client-side filtering")
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(30),
        );

    let logs = repo
        .fetch_recent_logs(page, limit)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let stats = repo
        .fetch_log_stats()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    dashboard = dashboard.add_section(create_stats_section(&stats));
    dashboard = dashboard.add_section(create_logs_table_section(&logs, page));

    let response = dashboard.to_response();

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "System Logs (Page {} of {}, {} per page)",
            page,
            (stats.total_logs as f64 / limit as f64).ceil() as i32,
            limit
        ))],
        structured_content: Some(response),
        is_error: Some(false),
        meta: None,
    })
}
