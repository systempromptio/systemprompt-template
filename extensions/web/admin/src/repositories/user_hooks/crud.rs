use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::super::types::{CreateUserHookRequest, UpdateUserHookRequest, UserHook};

const DEFAULT_HOOK_EVENTS: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "UserPromptSubmit",
    "Stop",
    "SubagentStop",
    "TaskCompleted",
    "SessionStart",
    "SessionEnd",
    "SubagentStart",
    "Notification",
    "TeammateIdle",
];

pub async fn ensure_default_hooks(
    pool: &PgPool,
    user_id: &UserId,
    platform_url: &str,
) -> Result<(), sqlx::Error> {
    if platform_url.is_empty() {
        return Ok(());
    }

    let existing: Vec<String> = sqlx::query_scalar!(
        "SELECT event_type FROM user_hooks WHERE user_id = $1 AND is_default = true",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    let track_url = format!("{platform_url}/api/public/hooks/track");

    for event in DEFAULT_HOOK_EVENTS {
        if existing.iter().any(|e| e == *event) {
            continue;
        }
        let id = uuid::Uuid::new_v4().to_string();
        let hook_name = format!("Platform: {event}");
        let description = format!("Default platform hook for {event} events");
        sqlx::query!(
            r"INSERT INTO user_hooks
                (id, user_id, hook_name, description, event_type, matcher,
                 hook_type, url, command, headers, timeout, is_async, enabled, is_default)
            VALUES ($1, $2, $3, $4, $5, '*', 'http', $6, '', '{}', 30, false, true, true)",
            id,
            user_id.as_str(),
            hook_name,
            description,
            *event,
            track_url,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn list_user_hooks(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UserHook>, sqlx::Error> {
    sqlx::query_as!(
        UserHook,
        r#"SELECT id, user_id, plugin_id, hook_name, description, event_type, matcher,
            hook_type, url, command, headers, timeout, is_async, enabled, is_default,
            created_at, updated_at
        FROM user_hooks
        WHERE user_id = $1
        ORDER BY event_type, hook_name"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn find_user_hook(
    pool: &PgPool,
    hook_id: &str,
    user_id: &UserId,
) -> Result<Option<UserHook>, sqlx::Error> {
    sqlx::query_as!(
        UserHook,
        r#"SELECT id, user_id, plugin_id, hook_name, description, event_type, matcher,
            hook_type, url, command, headers, timeout, is_async, enabled, is_default,
            created_at, updated_at
        FROM user_hooks
        WHERE id = $1 AND user_id = $2"#,
        hook_id,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
}

pub async fn create_user_hook(
    pool: &PgPool,
    user_id: &UserId,
    req: &CreateUserHookRequest,
) -> Result<UserHook, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query_as!(
        UserHook,
        r#"INSERT INTO user_hooks (id, user_id, plugin_id, hook_name, description, event_type,
            matcher, hook_type, url, command, headers, timeout, is_async)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, user_id, plugin_id, hook_name, description, event_type, matcher,
            hook_type, url, command, headers, timeout, is_async, enabled, is_default,
            created_at, updated_at"#,
        id,
        user_id.as_str(),
        req.plugin_id.as_deref(),
        req.hook_name,
        req.description,
        req.event_type,
        req.matcher,
        req.hook_type,
        req.url,
        req.command,
        req.headers,
        req.timeout,
        req.is_async,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_user_hook(
    pool: &PgPool,
    hook_id: &str,
    user_id: &UserId,
    req: &UpdateUserHookRequest,
) -> Result<Option<UserHook>, sqlx::Error> {
    sqlx::query_as!(
        UserHook,
        r#"UPDATE user_hooks SET
            hook_name = COALESCE($3, hook_name),
            description = COALESCE($4, description),
            event_type = COALESCE($5, event_type),
            matcher = COALESCE($6, matcher),
            url = COALESCE($7, url),
            command = COALESCE($8, command),
            headers = COALESCE($9, headers),
            timeout = COALESCE($10, timeout),
            is_async = COALESCE($11, is_async),
            enabled = COALESCE($12, enabled),
            updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, plugin_id, hook_name, description, event_type, matcher,
            hook_type, url, command, headers, timeout, is_async, enabled, is_default,
            created_at, updated_at"#,
        hook_id,
        user_id.as_str(),
        req.hook_name.as_deref(),
        req.description.as_deref(),
        req.event_type.as_deref(),
        req.matcher.as_deref(),
        req.url.as_deref(),
        req.command.as_deref(),
        req.headers.clone(),
        req.timeout,
        req.is_async,
        req.enabled,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_user_hook(
    pool: &PgPool,
    hook_id: &str,
    user_id: &UserId,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        "DELETE FROM user_hooks WHERE id = $1 AND user_id = $2 AND is_default = false",
        hook_id,
        user_id.as_str(),
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn toggle_user_hook(
    pool: &PgPool,
    hook_id: &str,
    user_id: &UserId,
) -> Result<Option<UserHook>, sqlx::Error> {
    sqlx::query_as!(
        UserHook,
        r#"UPDATE user_hooks SET enabled = NOT enabled, updated_at = NOW()
        WHERE id = $1 AND user_id = $2
        RETURNING id, user_id, plugin_id, hook_name, description, event_type, matcher,
            hook_type, url, command, headers, timeout, is_async, enabled, is_default,
            created_at, updated_at"#,
        hook_id,
        user_id.as_str(),
    )
    .fetch_optional(pool)
    .await
}
