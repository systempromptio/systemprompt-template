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

use repository::ConversationsRepository;
use sections::{
    create_agent_breakdown_section, create_conversation_trends_section,
    create_conversations_table_section, create_summary_cards_section,
};

pub fn conversations_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "time_range": {
                "type": "string",
                "enum": ["7d", "30d", "90d"],
                "default": "30d",
                "description": "Time range for metrics: 7d, 30d, or 90d"
            },
            "page": {
                "type": "integer",
                "default": 1,
                "minimum": 1,
                "description": "Page number for conversation table pagination"
            },
            "per_page": {
                "type": "integer",
                "default": 500,
                "minimum": 1,
                "maximum": 500,
                "description": "Number of conversations per page"
            }
        }
    })
}

pub fn conversations_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_conversations(
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

    let page = args.get("page").and_then(|v| v.as_i64()).unwrap_or(1) as i32;

    let per_page = args.get("per_page").and_then(|v| v.as_i64()).unwrap_or(10) as i32;

    logger
        .debug(
            "conversations_tool",
            &format!(
                "Generating analytics | type=conversations, period={}, page={}, per_page={}",
                time_range, page, per_page
            ),
        )
        .await
        .ok();

    let days = match time_range {
        "7d" => 7,
        "30d" => 30,
        "90d" => 90,
        _ => 30,
    };

    let repo = ConversationsRepository::new(pool.clone());

    let mut dashboard = DashboardArtifact::new("Conversation Analytics")
        .with_description(format!(
            "Conversation metrics for the last {} days with detailed recent conversations",
            days
        ))
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(60)
                .with_drill_down(true),
        );

    let summary = repo
        .get_conversation_summary(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let evaluation_stats = repo
        .get_evaluation_stats(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let offset = (page - 1) * per_page;
    let recent_conversations = repo
        .get_recent_conversations_paginated(days, per_page, offset)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !recent_conversations.is_empty() {
        dashboard =
            dashboard.add_section(create_conversations_table_section(&recent_conversations));
    }

    dashboard = dashboard.add_section(create_summary_cards_section(&summary, &evaluation_stats));

    let conversation_trends = repo
        .get_conversation_trends()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !conversation_trends.is_empty() {
        dashboard = dashboard.add_section(create_conversation_trends_section(&conversation_trends));
    }

    let agent_breakdown = repo
        .get_conversations_by_agent(days)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !agent_breakdown.is_empty() {
        dashboard = dashboard.add_section(create_agent_breakdown_section(&agent_breakdown));
    }

    let metadata = ExecutionMetadata::new().tool("conversations");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Conversation Analytics ({}) - Page {}, {} per page",
            time_range, page, per_page
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
