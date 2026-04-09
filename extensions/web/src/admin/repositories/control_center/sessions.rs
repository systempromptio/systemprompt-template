use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::admin::types::control_center::RecentSession;

pub async fn fetch_recent_sessions_filtered(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
    status: &str,
) -> Result<Vec<RecentSession>, sqlx::Error> {
    match status {
        "active" => fetch_sessions_active(pool, user_id, limit).await,
        _ => fetch_sessions_all(pool, user_id, limit).await,
    }
}

async fn fetch_sessions_active(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<RecentSession>, sqlx::Error> {
    sqlx::query_as!(
        RecentSession,
        r#"SELECT
            s.session_id, s.started_at, s.ended_at,
            COALESCE(s.total_events, 0)::BIGINT AS "total_events!",
            COALESCE(s.tool_uses, 0)::BIGINT AS "tool_uses!",
            COALESCE(s.prompts, 0)::BIGINT AS "prompts!",
            COALESCE(s.errors, 0)::BIGINT AS "errors!",
            COALESCE(s.content_input_bytes, 0)::BIGINT AS "content_input_bytes!",
            COALESCE(s.content_output_bytes, 0)::BIGINT AS "content_output_bytes!",
            COALESCE(s.subagent_spawns, 0)::BIGINT AS "subagent_spawns!",
            COALESCE(s.status, 'active') AS "status!",
            s.updated_at AS "updated_at!",
            COALESCE(s.client_source, '') AS "client_source!",
            COALESCE(s.permission_mode, '') AS "permission_mode!",
            COALESCE(s.user_prompts, 0)::INT AS "user_prompts!",
            COALESCE(s.automated_actions, 0)::INT AS "automated_actions!",
            COALESCE(s.model, '') AS "model!"
        FROM plugin_session_summaries s
        WHERE s.user_id = $1
          AND s.ended_at IS NULL
          AND COALESCE(s.status, 'active') = 'active'
          AND (COALESCE(s.prompts, 0) > 0 OR COALESCE(s.tool_uses, 0) > 0 OR COALESCE(s.total_events, 0) > 1)
        ORDER BY s.updated_at DESC
        LIMIT $2"#,
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await
}

async fn fetch_sessions_all(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<RecentSession>, sqlx::Error> {
    sqlx::query_as!(
        RecentSession,
        r#"SELECT
            s.session_id, s.started_at, s.ended_at,
            COALESCE(s.total_events, 0)::BIGINT AS "total_events!",
            COALESCE(s.tool_uses, 0)::BIGINT AS "tool_uses!",
            COALESCE(s.prompts, 0)::BIGINT AS "prompts!",
            COALESCE(s.errors, 0)::BIGINT AS "errors!",
            COALESCE(s.content_input_bytes, 0)::BIGINT AS "content_input_bytes!",
            COALESCE(s.content_output_bytes, 0)::BIGINT AS "content_output_bytes!",
            COALESCE(s.subagent_spawns, 0)::BIGINT AS "subagent_spawns!",
            COALESCE(s.status, 'active') AS "status!",
            s.updated_at AS "updated_at!",
            COALESCE(s.client_source, '') AS "client_source!",
            COALESCE(s.permission_mode, '') AS "permission_mode!",
            COALESCE(s.user_prompts, 0)::INT AS "user_prompts!",
            COALESCE(s.automated_actions, 0)::INT AS "automated_actions!",
            COALESCE(s.model, '') AS "model!"
        FROM plugin_session_summaries s
        WHERE s.user_id = $1
          AND COALESCE(s.status, 'active') != 'deleted'
          AND (COALESCE(s.prompts, 0) > 0 OR COALESCE(s.tool_uses, 0) > 0 OR COALESCE(s.total_events, 0) > 1)
        ORDER BY s.updated_at DESC
        LIMIT $2"#,
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await
}

pub async fn update_session_status(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &str,
    status: &str,
) -> Result<(), sqlx::Error> {
    if status == "deleted" {
        sqlx::query!(
            "DELETE FROM session_analyses WHERE session_id = $1 AND user_id = $2",
            session_id,
            user_id.as_str(),
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "DELETE FROM session_entity_links WHERE session_id = $1 AND user_id = $2",
            session_id,
            user_id.as_str(),
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "DELETE FROM session_ratings WHERE session_id = $1 AND user_id = $2",
            session_id,
            user_id.as_str(),
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "DELETE FROM plugin_usage_events WHERE session_id = $1 AND user_id = $2",
            session_id,
            user_id.as_str(),
        )
        .execute(pool)
        .await?;

        sqlx::query!(
            "DELETE FROM plugin_session_summaries WHERE session_id = $1 AND user_id = $2",
            session_id,
            user_id.as_str(),
        )
        .execute(pool)
        .await?;
    } else {
        sqlx::query!(
            r"UPDATE plugin_session_summaries
              SET status = $1, updated_at = NOW()
              WHERE user_id = $2 AND session_id = $3",
            status,
            user_id.as_str(),
            session_id,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}
