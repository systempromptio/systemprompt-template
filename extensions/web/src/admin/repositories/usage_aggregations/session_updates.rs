use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

pub async fn update_session_metadata(
    pool: &PgPool,
    session_id: &SessionId,
    source: &str,
    model: &str,
    permission_mode: &str,
) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET client_source = CASE WHEN $2 != '' THEN $2 ELSE client_source END,
              model = CASE WHEN $3 != '' THEN $3 ELSE model END,
              permission_mode = CASE WHEN $4 != '' THEN $4 ELSE permission_mode END,
              updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        source,
        model,
        permission_mode,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session metadata");
    }
}

pub async fn update_session_permission_mode(
    pool: &PgPool,
    session_id: &SessionId,
    permission_mode: &str,
) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET permission_mode = $2, updated_at = NOW()
          WHERE session_id = $1",
        session_id.as_str(),
        permission_mode,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session permission mode");
    }
}

pub async fn update_session_title_if_empty(pool: &PgPool, session_id: &SessionId, title: &str) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries
          SET ai_title = $2, updated_at = NOW()
          WHERE session_id = $1 AND (ai_title IS NULL OR ai_title = '')",
        session_id.as_str(),
        title,
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update session title");
    }
}

pub async fn update_unique_files_touched(pool: &PgPool, session_id: &SessionId, _file_path: &str) {
    let result = sqlx::query!(
        r"UPDATE plugin_session_summaries SET unique_files_touched = sub.cnt
          FROM (
            SELECT COUNT(DISTINCT f.path) AS cnt
            FROM plugin_usage_events e,
                 LATERAL (SELECT e.metadata->'tool_input'->>'file_path' AS path) f
            WHERE e.session_id = $1 AND f.path IS NOT NULL AND f.path != ''
          ) sub
          WHERE plugin_session_summaries.session_id = $1",
        session_id.as_str(),
    )
    .execute(pool)
    .await;

    if let Err(e) = result {
        tracing::warn!(error = %e, "Failed to update unique_files_touched");
    }
}
