use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{
    CreateUserPluginRequest, UpdateUserPluginRequest, UserPlugin,
};

pub async fn list_user_plugins(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserPlugin>, sqlx::Error> {
    sqlx::query_as::<_, UserPlugin>(
        r"
        SELECT id, user_id, plugin_id, name, description, version, enabled, category, keywords, author_name, base_plugin_id, created_at, updated_at
        FROM user_plugins
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_user_plugin(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<Option<UserPlugin>, sqlx::Error> {
    sqlx::query_as::<_, UserPlugin>(
        r"
        SELECT id, user_id, plugin_id, name, description, version, enabled, category, keywords, author_name, base_plugin_id, created_at, updated_at
        FROM user_plugins
        WHERE user_id = $1 AND plugin_id = $2
        ",
    )
    .bind(user_id)
    .bind(plugin_id)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn create_user_plugin(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &CreateUserPluginRequest,
) -> Result<UserPlugin, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, UserPlugin>(
        r"
        INSERT INTO user_plugins (id, user_id, plugin_id, name, description, version, category, keywords, author_name, base_plugin_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, user_id, plugin_id, name, description, version, enabled, category, keywords, author_name, base_plugin_id, created_at, updated_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&req.plugin_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.version)
    .bind(&req.category)
    .bind(&req.keywords)
    .bind(&req.author_name)
    .bind(&req.base_plugin_id)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn update_user_plugin(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
    req: &UpdateUserPluginRequest,
) -> Result<Option<UserPlugin>, sqlx::Error> {
    sqlx::query_as::<_, UserPlugin>(
        r"
        UPDATE user_plugins SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            version = COALESCE($5, version),
            enabled = COALESCE($6, enabled),
            category = COALESCE($7, category),
            keywords = COALESCE($8, keywords),
            author_name = COALESCE($9, author_name),
            updated_at = NOW()
        WHERE user_id = $1 AND plugin_id = $2
        RETURNING id, user_id, plugin_id, name, description, version, enabled, category, keywords, author_name, base_plugin_id, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(plugin_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.version)
    .bind(req.enabled)
    .bind(&req.category)
    .bind(&req.keywords)
    .bind(&req.author_name)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn delete_user_plugin(
    pool: &Arc<PgPool>,
    user_id: &str,
    plugin_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_plugins WHERE user_id = $1 AND plugin_id = $2")
        .bind(user_id)
        .bind(plugin_id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}
