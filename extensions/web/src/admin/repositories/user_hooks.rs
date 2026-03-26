use std::collections::HashMap;
use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{CreateUserHookRequest, UpdateUserHookRequest, UserHook};

pub async fn get_hook_overrides_enabled_map(
    pool: &Arc<PgPool>,
) -> Result<HashMap<String, bool>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, bool)>("SELECT hook_id, enabled FROM hook_overrides")
        .fetch_all(pool.as_ref())
        .await?;
    Ok(rows.into_iter().collect())
}

pub async fn upsert_hook_override_enabled(
    pool: &Arc<PgPool>,
    hook_id: &str,
    enabled: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r"
        INSERT INTO hook_overrides (hook_id, enabled)
        VALUES ($1, $2)
        ON CONFLICT (hook_id) DO UPDATE SET enabled = $2, updated_at = NOW()
        ",
    )
    .bind(hook_id)
    .bind(enabled)
    .execute(pool.as_ref())
    .await?;
    Ok(())
}

pub async fn list_user_hooks(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<UserHook>, sqlx::Error> {
    sqlx::query_as::<_, UserHook>(
        r"
        SELECT id, user_id, hook_id, name, description, event, matcher, command, is_async, enabled, base_hook_id, created_at, updated_at
        FROM user_hooks
        WHERE user_id = $1
        ORDER BY created_at DESC
        ",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn create_user_hook(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &CreateUserHookRequest,
) -> Result<UserHook, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as::<_, UserHook>(
        r"
        INSERT INTO user_hooks (id, user_id, hook_id, name, description, event, matcher, command, is_async, base_hook_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, user_id, hook_id, name, description, event, matcher, command, is_async, enabled, base_hook_id, created_at, updated_at
        ",
    )
    .bind(&id)
    .bind(user_id)
    .bind(&req.hook_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.event)
    .bind(&req.matcher)
    .bind(&req.command)
    .bind(req.is_async)
    .bind(&req.base_hook_id)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn delete_user_hook(
    pool: &Arc<PgPool>,
    user_id: &str,
    hook_id: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM user_hooks WHERE user_id = $1 AND hook_id = $2")
        .bind(user_id)
        .bind(hook_id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn update_user_hook(
    pool: &Arc<PgPool>,
    user_id: &str,
    hook_id: &str,
    req: &UpdateUserHookRequest,
) -> Result<Option<UserHook>, sqlx::Error> {
    sqlx::query_as::<_, UserHook>(
        r"
        UPDATE user_hooks SET
            name = COALESCE($3, name),
            description = COALESCE($4, description),
            event = COALESCE($5, event),
            matcher = COALESCE($6, matcher),
            command = COALESCE($7, command),
            is_async = COALESCE($8, is_async),
            enabled = COALESCE($9, enabled),
            updated_at = NOW()
        WHERE user_id = $1 AND hook_id = $2
        RETURNING id, user_id, hook_id, name, description, event, matcher, command, is_async, enabled, base_hook_id, created_at, updated_at
        ",
    )
    .bind(user_id)
    .bind(hook_id)
    .bind(&req.name)
    .bind(&req.description)
    .bind(&req.event)
    .bind(&req.matcher)
    .bind(&req.command)
    .bind(req.is_async)
    .bind(req.enabled)
    .fetch_optional(pool.as_ref())
    .await
}
