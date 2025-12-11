use chrono::Utc;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use sqlx::types::Uuid;
use systemprompt_core_blog::repository::ContentRepository;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_identifiers::McpExecutionId;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, DashboardSection, ExecutionMetadata, LayoutMode,
    LayoutWidth, SectionLayout, SectionType, ToolResponse,
};

pub fn delete_content_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "required": ["uuid"],
        "properties": {
            "uuid": {
                "type": "string",
                "description": "UUID of the resource to delete."
            },
            "resource_type": {
                "type": "string",
                "enum": ["content", "file"],
                "default": "content",
                "description": "Type of resource to delete: 'content' (default) or 'file'"
            }
        }
    })
}

pub fn delete_content_output_schema() -> JsonValue {
    ToolResponse::<DashboardArtifact>::schema()
}

pub async fn handle_delete_content(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
    mcp_execution_id: &McpExecutionId,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let uuid_str = args
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: uuid", None))?;

    let uuid = Uuid::parse_str(uuid_str)
        .map_err(|e| McpError::invalid_params(format!("Invalid UUID: {}", e), None))?;

    let resource_type = args
        .get("resource_type")
        .and_then(|v| v.as_str())
        .unwrap_or("content");

    let pg_pool = pool.pool_arc().expect("Database must be PostgreSQL");

    let (title, message) = match resource_type {
        "file" => {
            logger
                .debug(
                    "delete_content",
                    &format!("Deleting file | uuid={}", uuid_str),
                )
                .await
                .ok();

            let now = Utc::now();
            sqlx::query!("UPDATE files SET deleted_at = $1 WHERE id = $2", now, uuid)
                .execute(&*pg_pool)
                .await
                .map_err(|e| {
                    McpError::internal_error(format!("Failed to delete file: {}", e), None)
                })?;

            logger
                .debug(
                    "delete_content",
                    &format!("File deleted | uuid={}", uuid_str),
                )
                .await
                .ok();

            ("File Deleted", format!("Deleted file: {}", uuid_str))
        }
        _ => {
            logger
                .debug(
                    "delete_content",
                    &format!("Deleting content | uuid={}", uuid_str),
                )
                .await
                .ok();

            let content_repo = ContentRepository::new(pool.clone());
            content_repo.delete(uuid_str).await.map_err(|e| {
                McpError::internal_error(format!("Failed to delete content: {}", e), None)
            })?;

            logger
                .debug(
                    "delete_content",
                    &format!("Content deleted | uuid={}", uuid_str),
                )
                .await
                .ok();

            ("Content Deleted", format!("Deleted content: {}", uuid_str))
        }
    };

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

    let metadata = ExecutionMetadata::new().tool("delete_content");
    let artifact_id = uuid::Uuid::new_v4().to_string();
    let tool_response = ToolResponse::new(
        &artifact_id,
        mcp_execution_id.clone(),
        dashboard,
        metadata.clone(),
    );

    Ok(CallToolResult {
        content: vec![Content::text(message)],
        structured_content: Some(tool_response.to_json()),
        is_error: Some(false),
        meta: metadata.to_meta(),
    })
}
