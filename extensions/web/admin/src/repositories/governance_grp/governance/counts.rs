//! Allow/deny/secret rollups: overall and per-policy, lifetime and windowed.

use sqlx::PgPool;

use super::{GovernanceCounts, PerPolicyCounts};

pub async fn fetch_governance_counts(pool: &PgPool) -> Result<GovernanceCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            COUNT(*) FILTER (WHERE reason ILIKE '%secret%')::bigint AS "secret_breaches!"
        FROM governance_decisions"#,
    )
    .fetch_one(pool)
    .await?;
    Ok(GovernanceCounts {
        total: row.total,
        allowed: row.allowed,
        denied: row.denied,
        secret_breaches: row.secret_breaches,
    })
}

/// Lifetime totals scoped to a sliding window ending at `now()`.
pub async fn fetch_governance_counts_windowed(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<GovernanceCounts, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            COUNT(*) FILTER (WHERE reason ILIKE '%secret%')::bigint AS "secret_breaches!"
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)"#,
        window_seconds as f64,
    )
    .fetch_one(pool)
    .await?;
    Ok(GovernanceCounts {
        total: row.total,
        allowed: row.allowed,
        denied: row.denied,
        secret_breaches: row.secret_breaches,
    })
}

/// One row per `policy` value seen in `governance_decisions`. Used by the
/// Policies dashboard to show recent activity next to each registered policy.
pub async fn fetch_per_policy_counts(pool: &PgPool) -> Result<Vec<PerPolicyCounts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            policy,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            MAX(created_at) AS last_at
        FROM governance_decisions
        GROUP BY policy"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| PerPolicyCounts {
            policy: r.policy,
            allowed: r.allowed,
            denied: r.denied,
            last_at: r.last_at,
        })
        .collect())
}

/// Per-policy counts within a sliding window ending at `now()`. Used by the
/// Policies dashboard's "Enforcement (last 24h)" panel.
pub async fn fetch_per_policy_counts_windowed(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<Vec<PerPolicyCounts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            policy,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            MAX(created_at) AS last_at
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY policy"#,
        window_seconds as f64,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| PerPolicyCounts {
            policy: r.policy,
            allowed: r.allowed,
            denied: r.denied,
            last_at: r.last_at,
        })
        .collect())
}
