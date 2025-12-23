mod models;
pub mod repository;
mod schema;

pub use schema::{users_input_schema, users_output_schema};

use anyhow::Result;
use repository::UsersRepository;
use rmcp::{model::{CallToolRequestParam, CallToolResult, Content}, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    Column, ColumnType, ExecutionMetadata, TableArtifact, ToolResponse,
};

pub async fn handle_users(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let user_id = args.get("user_id").and_then(|v| v.as_str());

    let repo = UsersRepository::new(pool.clone())
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    tracing::debug!(user_id = ?user_id, "Listing users");

    let users = repo
        .list_users(user_id)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    tracing::debug!(count = users.len(), "Users listed");

    let items: Vec<JsonValue> = users.iter().map(|u| json!(u)).collect();

    let columns = vec![
        Column::new("id", ColumnType::String).with_label("ID"),
        Column::new("name", ColumnType::String).with_label("Name"),
        Column::new("email", ColumnType::String).with_label("Email"),
        Column::new("display_name", ColumnType::String).with_label("Display Name"),
        Column::new("status", ColumnType::String).with_label("Status"),
        Column::new("roles", ColumnType::String).with_label("Roles"),
        Column::new("total_sessions", ColumnType::Integer).with_label("Sessions"),
        Column::new("created_at", ColumnType::Date).with_label("Created"),
    ];

    let metadata = ExecutionMetadata::new().tool("users");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let artifact = TableArtifact::new(columns)
        .with_rows(items.clone())
        .with_metadata(metadata.clone());
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        artifact,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Found {} users\n\n{}",
            users.len(),
            serde_json::to_string_pretty(&items).unwrap_or_default()
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
