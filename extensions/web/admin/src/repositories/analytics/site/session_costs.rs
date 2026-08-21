//! Client-reported session cost snapshots: cache economics and context-window
//! pressure, the two levers `ai_requests` alone cannot show.
//!
//! `session_cost_snapshots` holds one statusline-reported row per session,
//! keyed on `updated_at` (set-not-increment upserts) — so windows select by
//! last update and every figure is labeled "client-reported" on the page.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Default, Clone, Copy)]
pub struct SessionCostStats {
    pub sessions: i64,
    pub cache_read_tokens: i64,
    pub cache_creation_tokens: i64,
    pub input_tokens: i64,
    /// `cache_read / (cache_read + input)` — the share of prompt tokens served
    /// from cache. 0 when there are no tokens at all.
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
        LEFT JOIN organization_members m ON m.user_id = s.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = s.user_id
        WHERE s.updated_at >= $1 AND s.updated_at < $2
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR s.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
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

/// The user's most recently updated sessions, for the drill-down page's
/// cost-history table.
pub async fn list_user_session_costs(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<UserSessionCostRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT session_id AS "session_id!: SessionId", model,
               COALESCE(total_cost_microdollars, 0)::BIGINT AS "cost!",
               COALESCE(context_window_size, 0)::BIGINT AS "context!",
               COALESCE(cache_read_input_tokens, 0)::BIGINT AS "cache_read!",
               COALESCE(input_tokens, 0)::BIGINT AS "input!",
               COALESCE(output_tokens, 0)::BIGINT AS "output!",
               updated_at AS "updated_at!"
        FROM session_cost_snapshots
        WHERE user_id = $1
        ORDER BY updated_at DESC
        LIMIT $2
        "#,
        user_id.as_str(),
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UserSessionCostRow {
            session_id: r.session_id,
            model: r.model,
            total_cost_microdollars: r.cost,
            context_window_size: r.context,
            cache_read_input_tokens: r.cache_read,
            input_tokens: r.input,
            output_tokens: r.output,
            updated_at: r.updated_at,
        })
        .collect())
}
