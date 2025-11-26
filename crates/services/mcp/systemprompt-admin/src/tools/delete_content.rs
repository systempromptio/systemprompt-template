use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_blog::repository::ContentRepository;
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;
use systemprompt_models::artifacts::{
    DashboardArtifact, DashboardHints, DashboardSection, LayoutMode, LayoutWidth, SectionLayout,
    SectionType,
};

pub fn delete_content_input_schema() -> JsonValue {
    json!({
        "type": "object",
        "required": ["uuid"],
        "properties": {
            "uuid": {
                "type": "string",
                "description": "UUID of the content to delete. Cascade deletes all related data (tags, revisions, child content)."
            }
        }
    })
}

pub fn delete_content_output_schema() -> JsonValue {
    json!({
        "type": "object",
        "description": "Deletion confirmation dashboard",
        "properties": {
            "x-artifact-type": {
                "type": "string",
                "enum": ["dashboard"]
            },
            "title": {"type": "string"},
            "sections": {"type": "array"},
            "mcp_execution_id": {"type": "string"}
        },
        "required": ["title", "sections", "mcp_execution_id"],
        "x-artifact-type": "dashboard"
    })
}

pub async fn handle_delete_content(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();

    let uuid = args
        .get("uuid")
        .and_then(|v| v.as_str())
        .ok_or_else(|| McpError::invalid_params("Missing required parameter: uuid", None))?;

    logger
        .info("delete_content", &format!("Deleting content: {}", uuid))
        .await
        .ok();

    let content_repo = ContentRepository::new(pool.clone());

    content_repo
        .delete(uuid)
        .await
        .map_err(|e| McpError::internal_error(format!("Failed to delete content: {}", e), None))?;

    logger
        .info(
            "delete_content",
            &format!("Successfully deleted content: {}", uuid),
        )
        .await
        .ok();

    let dashboard = DashboardArtifact::new("Content Deleted")
        .with_hints(DashboardHints::new().with_layout(LayoutMode::Vertical))
        .add_section(
            DashboardSection::new("status", "Status", SectionType::MetricsCards)
                .with_data(json!({
                    "cards": [{
                        "title": "Content Deleted",
                        "value": &uuid[..8.min(uuid.len())],
                        "icon": "trash-2",
                        "status": "success"
                    }]
                }))
                .with_layout(SectionLayout {
                    width: LayoutWidth::Full,
                    order: 1,
                }),
        );

    Ok(CallToolResult {
        content: vec![Content::text(format!("Deleted content: {}", uuid))],
        structured_content: Some(dashboard.to_response()),
        is_error: Some(false),
        meta: None,
    })
}
