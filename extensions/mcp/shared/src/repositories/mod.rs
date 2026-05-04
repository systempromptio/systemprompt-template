use sqlx::PgPool;

#[derive(Debug)]
pub struct McpAccessParams<'a> {
    pub user_id: &'a str,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_name: &'a str,
    pub description: &'a str,
    pub metadata: &'a serde_json::Value,
}

pub async fn insert_mcp_access(
    pool: &PgPool,
    params: &McpAccessParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', $2, $3, $4, $5, $6)",
        params.user_id,
        params.action,
        params.entity_type,
        params.entity_name,
        params.description,
        params.metadata,
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn insert_mcp_access_rejection(
    pool: &PgPool,
    server: &str,
    description: &str,
    metadata: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, COALESCE((SELECT id FROM users WHERE email LIKE '%@anonymous.local' LIMIT 1), (SELECT id FROM users LIMIT 1)), 'mcp_access', 'rejected', 'mcp_server', $1, $2, $3)",
        server,
        description,
        metadata,
    )
    .execute(pool)
    .await?;
    Ok(())
}
