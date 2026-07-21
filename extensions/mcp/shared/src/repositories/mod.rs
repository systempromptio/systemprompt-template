//! Database access shared by the MCP server extensions.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

#[derive(Debug)]
pub(crate) struct McpAccessParams<'a> {
    pub user_id: &'a UserId,
    pub action: &'a str,
    pub entity_type: &'a str,
    pub entity_name: &'a str,
    pub description: &'a str,
    pub metadata: &'a serde_json::Value,
}

pub(crate) async fn insert_mcp_access(
    pool: &PgPool,
    params: &McpAccessParams<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', $2, $3, $4, $5, $6)",
        params.user_id.as_str(),
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

/// Resolve the id of the dedicated anonymous principal, if one exists.
///
/// Rejections must never be attributed to an arbitrary user, so this looks up
/// only the reserved `*@anonymous.local` account and returns `None` when it is
/// absent rather than falling back to whatever user happens to be first.
pub(crate) async fn find_anonymous_user_id(pool: &PgPool) -> Result<Option<UserId>, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT id as "id: UserId" FROM users WHERE email LIKE '%@anonymous.local' ORDER BY created_at LIMIT 1"#
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(|r| r.id))
}

pub(crate) async fn insert_mcp_access_rejection(
    pool: &PgPool,
    user_id: &UserId,
    server: &str,
    description: &str,
    metadata: &serde_json::Value,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, 'mcp_access', 'rejected', 'mcp_server', $2, $3, $4)",
        user_id.as_str(),
        server,
        description,
        metadata,
    )
    .execute(pool)
    .await?;
    Ok(())
}
