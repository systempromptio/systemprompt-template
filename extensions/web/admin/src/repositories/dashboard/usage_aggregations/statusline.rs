//! Statusline snapshots: client-reported session cost and context-window usage.
//!
//! The statusline reports cumulative totals, so every write replaces the
//! previous snapshot (set, not increment) — both here and on the
//! `plugin_session_summaries` token columns.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

#[derive(Debug, Clone, Copy)]
pub struct SessionCostSnapshot<'a> {
    pub session_id: &'a SessionId,
    pub user_id: &'a UserId,
    pub model: Option<&'a str>,
    pub total_cost_microdollars: Option<i64>,
    pub context_window_size: Option<i64>,
    pub input_tokens: Option<i64>,
    pub output_tokens: Option<i64>,
    pub cache_creation_input_tokens: Option<i64>,
    pub cache_read_input_tokens: Option<i64>,
}

pub async fn upsert_session_cost_snapshot(
    pool: &PgPool,
    params: &SessionCostSnapshot<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "INSERT INTO session_cost_snapshots
            (session_id, user_id, model, total_cost_microdollars,
             context_window_size, input_tokens, output_tokens,
             cache_creation_input_tokens, cache_read_input_tokens, updated_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
         ON CONFLICT (session_id) DO UPDATE SET
            user_id = EXCLUDED.user_id,
            model = COALESCE(EXCLUDED.model, session_cost_snapshots.model),
            total_cost_microdollars =
                COALESCE(EXCLUDED.total_cost_microdollars, session_cost_snapshots.total_cost_microdollars),
            context_window_size =
                COALESCE(EXCLUDED.context_window_size, session_cost_snapshots.context_window_size),
            input_tokens = COALESCE(EXCLUDED.input_tokens, session_cost_snapshots.input_tokens),
            output_tokens = COALESCE(EXCLUDED.output_tokens, session_cost_snapshots.output_tokens),
            cache_creation_input_tokens =
                COALESCE(EXCLUDED.cache_creation_input_tokens, session_cost_snapshots.cache_creation_input_tokens),
            cache_read_input_tokens =
                COALESCE(EXCLUDED.cache_read_input_tokens, session_cost_snapshots.cache_read_input_tokens),
            updated_at = NOW()",
        params.session_id.as_str(),
        params.user_id.as_str(),
        params.model,
        params.total_cost_microdollars,
        params.context_window_size,
        params.input_tokens,
        params.output_tokens,
        params.cache_creation_input_tokens,
        params.cache_read_input_tokens,
    )
    .execute(pool)
    .await?;
    Ok(())
}

/// Backfill the session summary's never-populated token columns from the
/// statusline's cumulative report. No-op when the summary row doesn't exist
/// yet — the hook path owns row creation.
pub async fn set_session_summary_tokens(
    pool: &PgPool,
    session_id: &SessionId,
    input_tokens: Option<i64>,
    output_tokens: Option<i64>,
    model: Option<&str>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        "UPDATE plugin_session_summaries SET
            total_input_tokens = COALESCE($2, total_input_tokens),
            total_output_tokens = COALESCE($3, total_output_tokens),
            model = COALESCE(NULLIF($4, ''), model),
            updated_at = NOW()
         WHERE session_id = $1",
        session_id.as_str(),
        input_tokens,
        output_tokens,
        model,
    )
    .execute(pool)
    .await?;
    Ok(())
}
