use systemprompt::database::DbPool;

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
        "used" => format!("Executed '{tool}' on {server}"),
        _ => format!("{action} on {server}"),
    };
    let entity_type = if action == "used" {
        "tool"
    } else {
        "mcp_server"
    };
    let entity_name = if action == "used" { tool } else { server };
    let metadata = serde_json::json!({ "tool_name": tool, "server": server });

    if let Err(e) = sqlx::query(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', $2, $3, $4, $5, $6)",
    )
    .bind(user_id)
    .bind(action)
    .bind(entity_type)
    .bind(entity_name)
    .bind(&description)
    .bind(&metadata)
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

    if let Err(e) = sqlx::query(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, COALESCE((SELECT id FROM users WHERE email LIKE '%@anonymous.local' LIMIT 1), (SELECT id FROM users LIMIT 1)), 'mcp_access', 'rejected', 'mcp_server', $1, $2, $3)",
    )
    .bind(server)
    .bind(&description)
    .bind(&metadata)
    .execute(pg_pool.as_ref())
    .await
    {
        tracing::warn!(error = %e, "Failed to record MCP access rejection (non-fatal)");
    }
}
