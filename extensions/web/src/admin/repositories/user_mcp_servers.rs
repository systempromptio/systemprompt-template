use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{CreateUserMcpServerRequest, UpdateUserMcpServerRequest, UserMcpServer};

pub async fn list_user_mcp_servers(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserMcpServer>, sqlx::Error> {
    sqlx::query_as::<_, UserMcpServer>(
        r"
        SELECT id, user_id, mcp_server_id, name, description, binary, package_name, port, endpoint, enabled, oauth_required, oauth_scopes, oauth_audience, base_mcp_server_id, created_at, updated_at
        FROM user_mcp_servers
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn create_user_mcp_server(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &CreateUserMcpServerRequest,
) -> Result<UserMcpServer, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, UserMcpServer>(
        r"
        INSERT INTO user_mcp_servers (id, user_id, mcp_server_id, name, description, binary, package_name, port, endpoint, oauth_required, oauth_scopes, oauth_audience, base_mcp_server_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, user_id, mcp_server_id, name, description, binary, package_name, port, endpoint, enabled, oauth_required, oauth_scopes, oauth_audience, base_mcp_server_id, created_at, updated_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&req.mcp_server_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.binary)
    .bind(&req.package_name)
    .bind(req.port)
    .bind(&req.endpoint)
    .bind(req.oauth_required)
    .bind(&req.oauth_scopes)
    .bind(&req.oauth_audience)
    .bind(&req.base_mcp_server_id)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn update_user_mcp_server(
    pool: &Arc<PgPool>,
    user_id: &str,
    mcp_server_id: &str,
    req: &UpdateUserMcpServerRequest,
) -> Result<Option<UserMcpServer>, sqlx::Error> {
    sqlx::query_as::<_, UserMcpServer>(
        r"
        UPDATE user_mcp_servers SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            binary = COALESCE($5, binary),
            package_name = COALESCE($6, package_name),
            port = COALESCE($7, port),
            endpoint = COALESCE($8, endpoint),
            enabled = COALESCE($9, enabled),
            oauth_required = COALESCE($10, oauth_required),
            oauth_scopes = COALESCE($11, oauth_scopes),
            oauth_audience = COALESCE($12, oauth_audience),
            updated_at = NOW()
        WHERE user_id = $1 AND mcp_server_id = $2
        RETURNING id, user_id, mcp_server_id, name, description, binary, package_name, port, endpoint, enabled, oauth_required, oauth_scopes, oauth_audience, base_mcp_server_id, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(mcp_server_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.binary)
    .bind(&req.package_name)
    .bind(req.port)
    .bind(&req.endpoint)
    .bind(req.enabled)
    .bind(req.oauth_required)
    .bind(&req.oauth_scopes)
    .bind(&req.oauth_audience)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn delete_user_mcp_server(
    pool: &Arc<PgPool>,
    user_id: &str,
    mcp_server_id: &str,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query("DELETE FROM user_mcp_servers WHERE user_id = $1 AND mcp_server_id = $2")
            .bind(user_id)
            .bind(mcp_server_id)
            .execute(pool.as_ref())
            .await?;
    Ok(result.rows_affected() > 0)
}
