
use sqlx::PgPool;
use systemprompt::identifiers::{McpServerId, UserId};

use super::super::types::{CreateUserMcpServerRequest, UpdateUserMcpServerRequest, UserMcpServer};

pub async fn list_user_mcp_servers(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserMcpServer>, sqlx::Error> {
    sqlx::query_as!(
        UserMcpServer,
        r#"SELECT id, user_id as "user_id: UserId", mcp_server_id as "mcp_server_id: McpServerId",
            name, description, "binary", package_name, port, endpoint, enabled,
            oauth_required, COALESCE(oauth_scopes, '{}') as "oauth_scopes!", oauth_audience,
            base_mcp_server_id as "base_mcp_server_id: McpServerId",
            created_at, updated_at
        FROM user_mcp_servers
        WHERE user_id = $1
        ORDER BY created_at DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn create_user_mcp_server(
    pool: &PgPool,
    user_id: &UserId,
    req: &CreateUserMcpServerRequest,
) -> Result<UserMcpServer, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as!(
        UserMcpServer,
        r#"INSERT INTO user_mcp_servers (id, user_id, mcp_server_id, name, description, "binary", package_name, port, endpoint, oauth_required, oauth_scopes, oauth_audience, base_mcp_server_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, user_id as "user_id: UserId", mcp_server_id as "mcp_server_id: McpServerId",
            name, description, "binary", package_name, port, endpoint, enabled,
            oauth_required, COALESCE(oauth_scopes, '{}') as "oauth_scopes!", oauth_audience,
            base_mcp_server_id as "base_mcp_server_id: McpServerId",
            created_at, updated_at"#,
        id,
        user_id.as_str(),
        req.mcp_server_id.as_str(),
        req.name,
        req.description,
        req.binary,
        req.package_name,
        req.port,
        req.endpoint,
        req.oauth_required,
        &req.oauth_scopes as &[String],
        req.oauth_audience,
        req.base_mcp_server_id.as_ref().map(McpServerId::as_str),
    )
    .fetch_one(pool)
    .await
}

pub async fn get_or_create_user_mcp_server(
    pool: &PgPool,
    user_id: &UserId,
    req: &CreateUserMcpServerRequest,
) -> Result<UserMcpServer, sqlx::Error> {
    match create_user_mcp_server(pool, user_id, req).await {
        Ok(server) => Ok(server),
        Err(_) => {
            sqlx::query_as!(
                UserMcpServer,
                r#"SELECT id, user_id as "user_id: UserId", mcp_server_id as "mcp_server_id: McpServerId",
                    name, description, "binary", package_name, port, endpoint, enabled,
                    oauth_required, COALESCE(oauth_scopes, '{}') as "oauth_scopes!", oauth_audience,
                    base_mcp_server_id as "base_mcp_server_id: McpServerId",
                    created_at, updated_at
                FROM user_mcp_servers
                WHERE user_id = $1 AND mcp_server_id = $2"#,
                user_id.as_str(),
                req.mcp_server_id.as_str(),
            )
            .fetch_one(pool)
            .await
        }
    }
}

pub async fn update_user_mcp_server(
    pool: &PgPool,
    user_id: &UserId,
    mcp_server_id: &McpServerId,
    req: &UpdateUserMcpServerRequest,
) -> Result<Option<UserMcpServer>, sqlx::Error> {
    sqlx::query_as!(
        UserMcpServer,
        r#"UPDATE user_mcp_servers SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            "binary" = COALESCE($5, "binary"),
            package_name = COALESCE($6, package_name),
            port = COALESCE($7, port),
            endpoint = COALESCE($8, endpoint),
            enabled = COALESCE($9, enabled),
            oauth_required = COALESCE($10, oauth_required),
            oauth_scopes = COALESCE($11, oauth_scopes),
            oauth_audience = COALESCE($12, oauth_audience),
            updated_at = NOW()
        WHERE user_id = $1 AND mcp_server_id = $2
        RETURNING id, user_id as "user_id: UserId", mcp_server_id as "mcp_server_id: McpServerId",
            name, description, "binary", package_name, port, endpoint, enabled,
            oauth_required, COALESCE(oauth_scopes, '{}') as "oauth_scopes!", oauth_audience,
            base_mcp_server_id as "base_mcp_server_id: McpServerId",
            created_at, updated_at"#,
        user_id.as_str(),
        mcp_server_id.as_str(),
        req.name.as_deref(),
        req.description.as_deref(),
        req.binary.as_deref(),
        req.package_name.as_deref(),
        req.port,
        req.endpoint.as_deref(),
        req.enabled,
        req.oauth_required,
        &req.oauth_scopes as &Option<Vec<String>>,
        req.oauth_audience.as_deref(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_user_mcp_server(
    pool: &PgPool,
    user_id: &UserId,
    mcp_server_id: &McpServerId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM user_mcp_servers WHERE user_id = $1 AND mcp_server_id = $2",
        user_id.as_str(),
        mcp_server_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}
