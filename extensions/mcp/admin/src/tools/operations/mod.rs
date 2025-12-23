mod schema;
mod validation;

pub use schema::{operations_input_schema, operations_output_schema};
pub use validation::{handle_validate_agents, handle_validate_config, handle_validate_skills};

use chrono::Utc;
use rmcp::{model::{CallToolRequestParam, CallToolResult, Content}, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use sqlx::types::Uuid;
use systemprompt_core_blog::repository::ContentRepository;
use systemprompt_core_database::DbPool;
use systemprompt_core_files::File;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    Column, ColumnType, DashboardArtifact, DashboardHints, DashboardSection, ExecutionMetadata,
    LayoutMode, LayoutWidth, SectionLayout, SectionType, TableArtifact, ToolResponse,
};

pub async fn handle_operations(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let action = args
        .get("action")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("action is required", None))?;

    match action {
        "list_files" => handle_list_files(pool, &args, mcp_execution_id).await,
        "delete_file" => handle_delete_file(pool, &args, mcp_execution_id).await,
        "delete_content" => handle_delete_content(pool, &args, mcp_execution_id).await,
        "validate_skills" => handle_validate_skills(&args, mcp_execution_id).await,
        "validate_agents" => handle_validate_agents(&args, mcp_execution_id).await,
        "validate_config" => handle_validate_config(&args, mcp_execution_id).await,
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown action: {action}. Valid actions: list_files, delete_file, delete_content, validate_skills, validate_agents, validate_config"
            ),
            None,
        )),
    }
}

async fn handle_list_files(
    pool: &DbPool,
    args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let limit = args.get("limit").and_then(serde_json::Value::as_i64).unwrap_or(100);
    let offset = args.get("offset").and_then(serde_json::Value::as_i64).unwrap_or(0);

    tracing::debug!(limit = limit, offset = offset, "Listing files");

    let pg_pool = pool
        .pool_arc()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let files: Vec<File> = sqlx::query_as!(
        File,
        r#"
        SELECT
            id,
            file_path,
            public_url,
            mime_type,
            file_size_bytes,
            ai_content,
            metadata,
            user_id,
            session_id,
            trace_id,
            created_at,
            updated_at,
            deleted_at
        FROM files
        WHERE deleted_at IS NULL
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
        limit,
        offset
    )
    .fetch_all(&*pg_pool)
    .await
    .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    tracing::debug!(count = files.len(), "Files listed");

    let items: Vec<JsonValue> = files
        .iter()
        .map(|f| {
            json!({
                "id": f.id.to_string(),
                "thumbnail": f.public_url,
                "file_path": f.file_path,
                "public_url": f.public_url,
                "mime_type": f.mime_type,
                "file_size_bytes": f.file_size_bytes,
                "ai_content": f.ai_content,
                "created_at": f.created_at.to_rfc3339()
            })
        })
        .collect();

    let columns = vec![
        Column::new("id", ColumnType::String).with_label("ID"),
        Column::new("thumbnail", ColumnType::Link).with_label("Thumbnail"),
        Column::new("file_path", ColumnType::String).with_label("Path"),
        Column::new("public_url", ColumnType::Link).with_label("URL"),
        Column::new("mime_type", ColumnType::String).with_label("Type"),
        Column::new("file_size_bytes", ColumnType::Integer).with_label("Size"),
        Column::new("ai_content", ColumnType::Boolean).with_label("AI"),
        Column::new("created_at", ColumnType::Date).with_label("Created"),
    ];

    let metadata = ExecutionMetadata::new().tool("operations");
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
            "Found {} files\n\n{}",
            files.len(),
            serde_json::to_string_pretty(&items).unwrap_or_default()
        ))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}

async fn handle_delete_file(
    pool: &DbPool,
    args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let uuid_str = args
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("uuid is required for delete_file action", None))?;

    let uuid = Uuid::parse_str(uuid_str)
        .map_err(|e| McpError::invalid_params(format!("Invalid UUID: {e}"), None))?;

    tracing::debug!(uuid = %uuid_str, "Deleting file");

    let pg_pool = pool
        .pool_arc()
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    let now = Utc::now();
    sqlx::query!("UPDATE files SET deleted_at = $1 WHERE id = $2", now, uuid)
        .execute(&*pg_pool)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to delete file: {e}"), None))?;

    tracing::debug!(uuid = %uuid_str, "File deleted");

    build_delete_response("File Deleted", uuid_str, mcp_execution_id)
}

async fn handle_delete_content(
    pool: &DbPool,
    args: &serde_json::Map<String, JsonValue>,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let uuid_str = args.get("uuid").and_then(|v| v.as_str()).ok_or_else(|| {
        McpError::invalid_params("uuid is required for delete_content action", None)
    })?;

    tracing::debug!(uuid = %uuid_str, "Deleting content");

    let content_repo = ContentRepository::new(pool.clone());
    content_repo
        .delete(uuid_str)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to delete content: {e}"), None))?;

    tracing::debug!(uuid = %uuid_str, "Content deleted");

    build_delete_response("Content Deleted", uuid_str, mcp_execution_id)
}

fn build_delete_response(
    title: &str,
    uuid_str: &str,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let dashboard = DashboardArtifact::new(title)
        .with_hints(DashboardHints::new().with_layout(LayoutMode::Vertical))
        .add_section(
            DashboardSection::new("status", "Status", SectionType::MetricsCards)
                .with_data(json!({
                    "cards": [{
                        "title": title,
                        "value": &uuid_str[..8.min(uuid_str.len())],
                        "icon": "trash-2",
                        "status": "success"
                    }]
                }))
                .with_layout(SectionLayout {
                    width: LayoutWidth::Full,
                    order: 1,
                }),
        );

    let metadata = ExecutionMetadata::new().tool("operations");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(format!("{title}: {uuid_str}"))],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
