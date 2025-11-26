mod models;
mod repository;
mod schema;

pub use schema::{users_input_schema, users_output_schema};

use anyhow::Result;
use repository::UsersRepository;
use rmcp::{model::*, service::RequestContext, ErrorData as McpError, RoleServer};
use serde_json::{json, Value as JsonValue};
use systemprompt_core_database::DbPool;
use systemprompt_core_logging::LogService;

pub async fn handle_users(
    pool: &DbPool,
    request: CallToolRequestParam,
    _ctx: RequestContext<RoleServer>,
    logger: LogService,
) -> Result<CallToolResult, McpError> {
    let args = request.arguments.unwrap_or_default();
    let user_id = args.get("user_id").and_then(|v| v.as_str());

    let repo = UsersRepository::new(pool.clone());

    if let Some(id) = user_id {
        logger
            .info(
                "users_tool",
                &format!("Listing users filtered by user_id: {}", id),
            )
            .await
            .ok();
    } else {
        logger.info("users_tool", "Listing all users").await.ok();
    }

    let users = repo
        .list_users(user_id)
        .await
        .map_err(|e| McpError::internal_error(e.to_string(), None))?;

    logger
        .info("users_tool", &format!("Found {} users", users.len()))
        .await
        .ok();

    let items: Vec<JsonValue> = users.iter().map(|u| json!(u)).collect();

    Ok(CallToolResult {
        content: vec![Content::text(format!(
            "Found {} users\n\n{}",
            users.len(),
            serde_json::to_string_pretty(&items).unwrap_or_default()
        ))],
        structured_content: Some(json!({
            "items": items,
            "count": users.len()
        })),
        is_error: Some(false),
        meta: None,
    })
}
