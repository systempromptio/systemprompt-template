//! Latency-bucketed request split: the honest stand-in for "fast/slow
//! request pools", which this platform does not have.
//!
//! The threshold is fixed at 5000 ms — one of the request page's histogram
//! bin edges, so this split and that histogram can never disagree — and never
//! derived from percentiles, so period-over-period comparisons keep their
//! meaning. Untimed requests (NULL latency) are surfaced, not hidden.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

// Why: Boundary between "fast" and "slow", in milliseconds. Matches the 5s bin
// edge in `analytics::request_stats::LATENCY_BIN_EDGES_MS`.
pub const FAST_THRESHOLD_MS: i64 = 5_000;

// Why: `ai_requests.latency_ms` is INT, so the bind must be too; derived from
// the public constant rather than restated, so the two cannot drift.
const FAST_THRESHOLD_MS_I32: i32 = FAST_THRESHOLD_MS as i32;

#[derive(Debug, Default, Clone, Copy)]
pub struct LatencySplit {
    pub fast: i64,
    pub slow: i64,
    pub untimed: i64,
    pub p50_ms: f64,
    pub p95_ms: f64,
}

pub async fn get_latency_split(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<LatencySplit, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE r.latency_ms < $6)::BIGINT AS "fast!",
            COUNT(*) FILTER (WHERE r.latency_ms >= $6)::BIGINT AS "slow!",
            COUNT(*) FILTER (WHERE r.latency_ms IS NULL)::BIGINT AS "untimed!",
            COALESCE(percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms), 0)
                AS "p50!",
            COALESCE(percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms), 0)
                AS "p95!"
        FROM ai_requests r
        LEFT JOIN organization_members m ON m.user_id = r.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE r.created_at >= $1 AND r.created_at < $2
          AND NOT r.synthetic
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR r.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
        scope.user_id_str(),
        FAST_THRESHOLD_MS_I32,
    )
    .fetch_one(pool)
    .await?;

    Ok(LatencySplit {
        fast: row.fast,
        slow: row.slow,
        untimed: row.untimed,
        p50_ms: row.p50,
        p95_ms: row.p95,
    })
}
