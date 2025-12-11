mod repository;
mod schema;

pub use schema::{files_input_schema, files_output_schema};

use anyhow::Result;
use repository::FilesRepository;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    Column, ColumnType, ExecutionMetadata, TableArtifact, ToolResponse,
};

pub async fn handle_files(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let limit = args.get("limit").and_then(|v| v.as_i64()).unwrap_or(100);

    let offset = args.get("offset").and_then(|v| v.as_i64()).unwrap_or(0);

    logger
        .debug(
            "files_tool",
            &format!("Listing files | limit={}, offset={}", limit, offset),
        )
        .await
        .ok();

    let repo = FilesRepository::new(pool.clone());

    let files = repo
        .list_all_files(limit, offset)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    logger
        .debug(
            "files_tool",
            &format!("Files listed | count={}", files.len()),
        )
        .await
        .ok();

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

    let metadata = ExecutionMetadata::new().tool("files");
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
