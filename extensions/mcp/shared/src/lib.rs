use systemprompt::database::DbPool;

mod repositories;

use repositories::{insert_mcp_access, insert_mcp_access_rejection, McpAccessParams};

const ACTION_USED: &str = "used";

pub async fn record_mcp_access(
    pool: &DbPool,
    user_id: &str,
    server: &str,
    tool: &str,
    action: &str,
) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access event");
        return;
    };
    let description = match action {
        "authenticated" => format!("Authenticated to {server} for '{tool}'"),
        ACTION_USED => format!("Executed '{tool}' on {server}"),
        _ => format!("{action} on {server}"),
    };
    let entity_type = if action == ACTION_USED {
        "tool"
    } else {
        "mcp_server"
    };
    let entity_name = if action == ACTION_USED { tool } else { server };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server });

    let params = McpAccessParams {
        user_id,
        action,
        entity_type,
        entity_name,
        description: &description,
        metadata: &metadata,
    };

    if let Err(e) = insert_mcp_access(pg_pool.as_ref(), &params).await {
        tracing::warn!(error = %e, "Failed to record MCP access event (non-fatal)");
    }
}

pub async fn record_mcp_access_rejected(pool: &DbPool, server: &str, tool: &str, reason: &str) {
    let Some(pg_pool) = pool.pool() else {
        tracing::warn!("No PgPool available to record MCP access rejection");
        return;
    };
    let description = if reason.len() > 120 {
        format!("Access rejected on {server}: {}...", &reason[..117])
    } else {
        format!("Access rejected on {server}: {reason}")
    };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server, "reason": reason });

    if let Err(e) =
        insert_mcp_access_rejection(pg_pool.as_ref(), server, &description, &metadata).await
    {
        tracing::warn!(error = %e, "Failed to record MCP access rejection (non-fatal)");
    }
}
