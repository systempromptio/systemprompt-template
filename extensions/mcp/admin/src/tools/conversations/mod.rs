mod models;
pub mod repository;
mod sections;

use rmcp::{model::{CallToolRequestParam, CallToolResult, Content}, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_agent::{repository::TaskRepository, Part};
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, ExecutionMetadata, LayoutMode, ToolResponse,
};

use repository::ConversationsRepository;
use sections::{
    create_conversation_trends_section, create_conversations_table_section,
    create_summary_cards_section,
};

#[must_use] pub fn conversations_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "context_id": {
                "type": "string",
                "description": "Optional. When provided, returns full conversation details (messages) instead of analytics"
            },
            "time_range": {
                "type": "string",
                "enum": ["1h", "24h", "7d", "30d"],
                "default": "30d",
                "description": "Time range: 1h (last hour), 24h (last day), 7d, or 30d"
            },
            "agent_name": {
                "type": "string",
                "description": "Filter by agent name. Use 'non-anonymous' to exclude anonymous agents."
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

#[must_use] pub fn conversations_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_conversations(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    if let Some(context_id) = args.get("context_id").and_then(|v| v.as_str()) {
        return handle_conversation_details(pool, context_id, mcp_execution_id).await;
    }

    let time_range = args
        .get("time_range")
        .and_then(|v| v.as_str())
        .unwrap_or("30d");

    let agent_name = args.get("agent_name").and_then(|v| v.as_str());

    let page = args.get("page").and_then(serde_json::Value::as_i64).unwrap_or(1) as i32;

    let per_page = args.get("per_page").and_then(serde_json::Value::as_i64).unwrap_or(10) as i32;

    tracing::debug!(
        time_range = %time_range,
        agent_filter = ?agent_name,
        page = page,
        per_page = per_page,
        "Generating conversation analytics"
    );

    let interval = match time_range {
        "1h" => "1 hour",
        "24h" => "1 day",
        "7d" => "7 days",
        "30d" => "30 days",
        _ => "30 days",
    };

    let repo = ConversationsRepository::new(pool.clone())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let description = match agent_name {
        Some("non-anonymous") => format!(
            "Conversation metrics for the last {time_range} (excluding anonymous agents)"
        ),
        Some(name) => format!(
            "Conversation metrics for the last {time_range} (agent: {name})"
        ),
        None => format!(
            "Conversation metrics for the last {time_range} with detailed recent conversations"
        ),
    };

    let mut dashboard = DashboardArtifact::new("Conversation Analytics")
        .with_description(description)
        .with_hints(
            DashboardHints::new()
                .with_layout(LayoutMode::Vertical)
                .with_refreshable(true)
                .with_refresh_interval(60)
                .with_drill_down(true),
        );

    let summary = repo
        .get_conversation_summary(interval)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let offset = (page - 1) * per_page;
    let recent_conversations = repo
        .get_recent_conversations_paginated(interval, per_page, offset, agent_name)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !recent_conversations.is_empty() {
        dashboard =
            dashboard.add_section(create_conversations_table_section(&recent_conversations));
    }

    dashboard = dashboard.add_section(create_summary_cards_section(&summary));

    let conversation_trends = repo
        .get_conversation_trends()
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    if !conversation_trends.is_empty() {
        dashboard = dashboard.add_section(create_conversation_trends_section(&conversation_trends));
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
            "Conversation Analytics ({}) - Page {}, {} per page{}",
            time_range,
            page,
            per_page,
            agent_name.map_or(String::new(), |n| format!(", agent: {n}"))
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

async fn handle_conversation_details(
    pool: &DbPool,
    context_id: &str,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    tracing::debug!(context_id = %context_id, "Retrieving messages");

    let task_repo = TaskRepository::new(pool.clone());

    let tasks = task_repo
        .list_tasks_by_context(context_id)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let mut messages = Vec::new();

    for task in tasks {
        if let Some(history) = task.history {
            for msg in history {
                let content = msg
                    .parts
                    .iter()
                    .filter_map(|part| match part {
                        Part::Text(text) => Some(text.text.clone()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                messages.push(json!({
                    "id": msg.message_id,
                    "role": msg.role,
                    "content": content,
                }));
            }
        }
    }

    let artifact = json!({
        "context_id": context_id,
        "messages": messages,
    });

    tracing::debug!(count = messages.len(), "Messages retrieved");

    let metadata = ExecutionMetadata::new().tool("conversations");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        artifact.clone(),
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Conversation Details ({})\n\nShowing {} messages\n\n{}",
            context_id,
            messages.len(),
            serde_json::to_string_pretty(&artifact).unwrap_or_default()
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
