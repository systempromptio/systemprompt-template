use systemprompt::database::DbPool;

const ACTION_USED: &str = "used";

#[allow(clippy::cognitive_complexity)]
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

    if let Err(e) = sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', $2, $3, $4, $5, $6)",
        user_id,
        action,
        entity_type,
        entity_name,
        description,
        metadata,
    )
    .execute(pg_pool.as_ref())
    .await
    {
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

    if let Err(e) = sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, COALESCE((SELECT id FROM users WHERE email LIKE '%@anonymous.local' LIMIT 1), (SELECT id FROM users LIMIT 1)), 'mcp_access', 'rejected', 'mcp_server', $1, $2, $3)",
        server,
        description,
        metadata,
    )
    .execute(pg_pool.as_ref())
    .await
    {
        tracing::warn!(error = %e, "Failed to record MCP access rejection (non-fatal)");
    }
}
