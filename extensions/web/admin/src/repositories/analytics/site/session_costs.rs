//! Client-reported session cost snapshots: cache economics and context-window
//! pressure, the two levers `ai_requests` alone cannot show.
//!
//! `session_cost_snapshots` holds one statusline-reported row per session,
//! keyed on `updated_at` (set-not-increment upserts) — so windows select by
//! last update and every figure is labeled "client-reported" on the page.

use sqlx::PgPool;
use systemprompt::identifiers::SessionId;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Default, Clone, Copy)]
pub struct SessionCostStats {
    pub sessions: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub input_tokens: i64,
    // Why: `cache_read / (cache_read + input)` — the share of prompt tokens served
    // from cache. 0 when there are no tokens at all.
    pub cache_hit_pct: f64,
    pub avg_context_window: i64,
    pub max_context_window: i64,
}

pub async fn get_session_cost_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SessionCostStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "sessions!",
            COALESCE(SUM(s.cache_read_input_tokens), 0)::BIGINT AS "cache_read!",
            COALESCE(SUM(s.cache_creation_input_tokens), 0)::BIGINT AS "cache_creation!",
            COALESCE(SUM(s.input_tokens), 0)::BIGINT AS "input!",
            COALESCE(AVG(s.context_window_size), 0)::BIGINT AS "avg_context!",
            COALESCE(MAX(s.context_window_size), 0)::BIGINT AS "max_context!"
        FROM session_cost_snapshots s
        WHERE s.updated_at >= $1 AND s.updated_at < $2
          AND ($3::TEXT[] IS NULL OR s.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR s.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    let prompt_tokens = row.cache_read + row.input;
    let cache_hit_pct = if prompt_tokens > 0 {
        row.cache_read as f64 / prompt_tokens as f64 * 100.0
    } else {
        0.0
    };

    Ok(SessionCostStats {
        sessions: row.sessions,
        cache_read_tokens: row.cache_read,
        cache_creation_tokens: row.cache_creation,
        input_tokens: row.input,
        cache_hit_pct,
        avg_context_window: row.avg_context,
        max_context_window: row.max_context,
    })
}

#[derive(Debug, Clone)]
pub struct UserSessionCostRow {
    pub session_id: SessionId,
    pub model: Option<String>,
    pub total_cost_microdollars: i64,
    pub context_window_size: i64,
    pub cache_read_input_tokens: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}
