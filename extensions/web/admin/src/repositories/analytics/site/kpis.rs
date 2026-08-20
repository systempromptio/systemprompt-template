//! Headline figures for the dashboard's KPI strip.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Default, Clone, Copy)]
pub struct SiteKpis {
    pub total_requests: i64,
    pub error_count: i64,
    pub total_cost_microdollars: i64,
    pub total_tokens: i64,
    /// Distinct users with at least one request in the window.
    pub active_users: i64,
    /// Distinct users with at least one request in the trailing 7 days —
    /// computed against `NOW()` regardless of the picked window, and labeled
    /// that way on the page.
    pub weekly_active_users: i64,
}

pub async fn get_site_kpis(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SiteKpis, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "total!",
            COUNT(*) FILTER (WHERE r.created_at >= $1 AND r.created_at < $2
                AND r.status NOT IN ('completed', 'pending', 'streaming'))::BIGINT
                AS "errors!",
            COALESCE(SUM(r.cost_microdollars)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "cost!",
            COALESCE(SUM(r.input_tokens + r.output_tokens)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2), 0)::BIGINT
                AS "tokens!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= $1 AND r.created_at < $2)::BIGINT
                AS "active_users!",
            COUNT(DISTINCT r.user_id)
                FILTER (WHERE r.created_at >= NOW() - INTERVAL '7 days')::BIGINT
                AS "weekly_active_users!"
        FROM ai_requests r
        LEFT JOIN organization_members m ON m.user_id = r.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = r.user_id
        WHERE NOT r.synthetic
          AND (r.created_at >= $1 OR r.created_at >= NOW() - INTERVAL '7 days')
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR r.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(SiteKpis {
        total_requests: row.total,
        error_count: row.errors,
        total_cost_microdollars: row.cost,
        total_tokens: row.tokens,
        active_users: row.active_users,
        weekly_active_users: row.weekly_active_users,
    })
}

#[derive(Debug, Default, Clone, Copy)]
pub struct PermissionGrantStats {
    pub requests: i64,
    /// Permission requests followed by a matching tool use in the same
    /// session within ten minutes — the observable proxy for "granted".
    /// Approximate by construction: a re-prompted identical tool call counts
    /// as the grant of the earlier request too.
    pub granted: i64,
}

pub async fn get_permission_grant_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<PermissionGrantStats, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "requests!",
            COUNT(*) FILTER (WHERE EXISTS (
                SELECT 1 FROM plugin_usage_events g
                WHERE g.session_id = p.session_id
                  AND g.event_type = 'PostToolUse'
                  AND g.tool_name IS NOT DISTINCT FROM p.tool_name
                  AND g.created_at >= p.created_at
                  AND g.created_at < p.created_at + INTERVAL '10 minutes'
            ))::BIGINT AS "granted!"
        FROM plugin_usage_events p
        LEFT JOIN organization_members m ON m.user_id = p.user_id
        LEFT JOIN organizations o ON o.id = m.org_id
        LEFT JOIN user_profile_ext upe ON upe.user_id = p.user_id
        WHERE p.event_type = 'PermissionRequest'
          AND p.created_at >= $1 AND p.created_at < $2
          AND ($3::TEXT IS NULL OR o.slug = $3)
          AND ($4::TEXT IS NULL OR COALESCE(NULLIF(upe.department, ''), 'Default') = $4)
          AND ($5::TEXT IS NULL OR p.user_id = $5)
        "#,
        range.from,
        range.to,
        scope.org_slug.as_deref(),
        scope.department.as_deref(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(PermissionGrantStats {
        requests: row.requests,
        granted: row.granted,
    })
}
