use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_agent::{repository::TaskRepository, Part};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{ExecutionMetadata, ToolResponse};

pub fn conversation_details_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "properties": {
            "context_id": {
                "type": "string",
                "description": "The context ID of the conversation to retrieve"
            }
        },
        "required": ["context_id"]
    })
}

pub fn conversation_details_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "Conversation messages displayed as JSON",
        "properties": {
            "context_id": {"type": "string"},
            "messages": {
                "type": "array",
                "items": {"type": "object"}
            },
            "mcp_execution_id": {"type": "string"}
        },
        "required": ["context_id", "messages", "mcp_execution_id"],
        "x-artifact-type": "json"
    })
}

pub async fn handle_conversation_details(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let context_id = args
        .get("context_id")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("context_id is required", None))?;

    logger
        .debug(
            "conversation_details",
            &format!("Retrieving messages | context_id={}", context_id),
        )
        .await
        .ok();

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

    logger
        .debug(
            "conversation_details",
            &format!("Messages retrieved | count={}", messages.len()),
        )
        .await
        .ok();

    let metadata = ExecutionMetadata::new().tool("conversation_details");
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
