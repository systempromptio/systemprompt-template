use sqlx::PgPool;

use crate::types::{
    GovernanceDecisionRow, GovernanceEvent, IncidentGroup, TopActor, TopPolicy, WindowedCounts,
};

pub async fn list_governance_decisions(
    pool: &PgPool,
    search: Option<&str>,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    let search_pattern = search.filter(|s| !s.is_empty()).map(|s| format!("%{s}%"));

    if let Some(ref pattern) = search_pattern {
        sqlx::query_as::<_, GovernanceDecisionRow>(
            "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
             FROM governance_decisions \
             WHERE user_id ILIKE $1 OR tool_name ILIKE $1 OR decision ILIKE $1 \
                OR reason ILIKE $1 OR policy ILIKE $1 OR agent_scope ILIKE $1 \
             ORDER BY created_at DESC \
             LIMIT 200",
        )
        .bind(pattern)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, GovernanceDecisionRow>(
            "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
             FROM governance_decisions \
             ORDER BY created_at DESC \
             LIMIT 200",
        )
        .fetch_all(pool)
        .await
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernanceCounts {
    pub total: i64,
    pub allowed: i64,
    pub denied: i64,
    pub secret_breaches: i64,
}

pub async fn fetch_governance_counts(pool: &PgPool) -> Result<GovernanceCounts, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r"SELECT
            COUNT(*)::bigint,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            COUNT(*) FILTER (WHERE reason ILIKE '%secret%')::bigint
        FROM governance_decisions",
    )
    .fetch_one(pool)
    .await?;
    Ok(GovernanceCounts {
        total: row.0,
        allowed: row.1,
        denied: row.2,
        secret_breaches: row.3,
    })
}

/// Lifetime totals scoped to a sliding window ending at `now()`.
pub async fn fetch_governance_counts_windowed(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<GovernanceCounts, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r"SELECT
            COUNT(*)::bigint,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            COUNT(*) FILTER (WHERE reason ILIKE '%secret%')::bigint
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)",
    )
    .bind(window_seconds)
    .fetch_one(pool)
    .await?;
    Ok(GovernanceCounts {
        total: row.0,
        allowed: row.1,
        denied: row.2,
        secret_breaches: row.3,
    })
}

#[derive(Debug, Clone)]
pub struct PerPolicyCounts {
    pub policy: String,
    pub allowed: i64,
    pub denied: i64,
    pub last_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// One row per `policy` value seen in `governance_decisions`. Used by the
/// Policies dashboard to show recent activity next to each registered policy.
pub async fn fetch_per_policy_counts(pool: &PgPool) -> Result<Vec<PerPolicyCounts>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64, i64, Option<chrono::DateTime<chrono::Utc>>)>(
        r"SELECT
            policy,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            MAX(created_at)
        FROM governance_decisions
        GROUP BY policy",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(policy, allowed, denied, last_at)| PerPolicyCounts {
            policy,
            allowed,
            denied,
            last_at,
        })
        .collect())
}

/// Per-policy counts within a sliding window ending at `now()`. Used by the
/// Policies dashboard's "Enforcement (last 24h)" panel.
pub async fn fetch_per_policy_counts_windowed(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<Vec<PerPolicyCounts>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (String, i64, i64, Option<chrono::DateTime<chrono::Utc>>)>(
        r"SELECT
            policy,
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            MAX(created_at)
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY policy",
    )
    .bind(window_seconds)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|(policy, allowed, denied, last_at)| PerPolicyCounts {
            policy,
            allowed,
            denied,
            last_at,
        })
        .collect())
}

/// Decisions filtered to a single policy id. Used by the per-policy detail
/// page so admins can see exactly which calls a given policy has fired on.
pub async fn list_decisions_for_policy(
    pool: &PgPool,
    policy_id: &str,
    limit: i64,
) -> Result<Vec<GovernanceDecisionRow>, sqlx::Error> {
    sqlx::query_as::<_, GovernanceDecisionRow>(
        "SELECT id, user_id, tool_name, agent_id, agent_scope, decision, policy, reason, created_at \
         FROM governance_decisions \
         WHERE policy = $1 \
         ORDER BY created_at DESC \
         LIMIT $2",
    )
    .bind(policy_id)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn fetch_governance_events(pool: &PgPool) -> Result<Vec<GovernanceEvent>, sqlx::Error> {
    sqlx::query_as::<_, GovernanceEvent>(
        r"SELECT id, user_id, tool_name, agent_id, decision, reason, created_at
        FROM governance_decisions
        ORDER BY created_at DESC
        LIMIT 50",
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_windowed_counts(
    pool: &PgPool,
    window_seconds: i64,
) -> Result<WindowedCounts, sqlx::Error> {
    let row = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r"SELECT
            COUNT(*)::bigint,
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint,
            COUNT(*) FILTER (
                WHERE decision = 'deny'
                  AND (policy = 'secret_scan' OR reason ILIKE '%secret%')
            )::bigint,
            COUNT(DISTINCT user_id)::bigint
        FROM governance_decisions
        WHERE created_at > now() - make_interval(secs => $1::double precision)",
    )
    .bind(window_seconds)
    .fetch_one(pool)
    .await?;
    Ok(WindowedCounts {
        decisions: row.0,
        denied: row.1,
        secret_blocks: row.2,
        distinct_actors: row.3,
    })
}

/// Returns one row per matching window across the last `lookback_days`,
/// excluding the live window. Caller computes mean/stddev for σ-deviation.
pub async fn fetch_baseline_window_samples(
    pool: &PgPool,
    window_seconds: i64,
    lookback_days: i64,
) -> Result<Vec<WindowedCounts>, sqlx::Error> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, i64)>(
        r"WITH live_start AS (
            SELECT now() - make_interval(secs => $1::double precision) AS ts
        ),
        buckets AS (
            SELECT generate_series(
                date_trunc('minute', now() - make_interval(days => $2::int)),
                (SELECT ts FROM live_start) - make_interval(secs => $1::double precision),
                make_interval(secs => $1::double precision)
            ) AS bucket_start
        )
        SELECT
            COALESCE(COUNT(g.id), 0)::bigint,
            COALESCE(COUNT(g.id) FILTER (WHERE g.decision = 'deny'), 0)::bigint,
            COALESCE(COUNT(g.id) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'secret_scan' OR g.reason ILIKE '%secret%')
            ), 0)::bigint,
            COALESCE(COUNT(DISTINCT g.user_id), 0)::bigint
        FROM buckets b
        LEFT JOIN governance_decisions g
            ON g.created_at >= b.bucket_start
           AND g.created_at <  b.bucket_start + make_interval(secs => $1::double precision)
        GROUP BY b.bucket_start",
    )
    .bind(window_seconds)
    .bind(lookback_days)
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| WindowedCounts {
            decisions: r.0,
            denied: r.1,
            secret_blocks: r.2,
            distinct_actors: r.3,
        })
        .collect())
}

pub async fn fetch_top_actors(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<TopActor>, sqlx::Error> {
    sqlx::query_as::<_, TopActor>(
        r"SELECT
            g.user_id::TEXT AS user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, g.user_id) AS display_name,
            u.email::TEXT AS email,
            COUNT(*) FILTER (WHERE g.decision = 'deny')::bigint AS deny_count,
            COUNT(*) FILTER (
                WHERE g.decision = 'deny'
                  AND (g.policy = 'secret_scan' OR g.reason ILIKE '%secret%')
            )::bigint AS secret_count,
            COUNT(*)::bigint AS total
        FROM governance_decisions g
        LEFT JOIN users u ON u.id = g.user_id
        WHERE g.created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY g.user_id, u.display_name, u.full_name, u.name, u.email
        ORDER BY deny_count DESC, total DESC
        LIMIT $2",
    )
    .bind(window_seconds)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn fetch_top_policies(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<TopPolicy>, sqlx::Error> {
    sqlx::query_as::<_, TopPolicy>(
        r"SELECT
            policy,
            tool_name,
            COUNT(*)::bigint AS hits,
            COUNT(DISTINCT user_id)::bigint AS distinct_actors
        FROM governance_decisions
        WHERE decision = 'deny'
          AND created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY policy, tool_name
        ORDER BY hits DESC
        LIMIT $2",
    )
    .bind(window_seconds)
    .bind(limit)
    .fetch_all(pool)
    .await
}

pub async fn fetch_grouped_incidents(
    pool: &PgPool,
    window_seconds: i64,
    limit: i64,
) -> Result<Vec<IncidentGroup>, sqlx::Error> {
    sqlx::query_as::<_, IncidentGroup>(
        r"SELECT
            g.agent_id,
            g.user_id::TEXT AS user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email) AS display_name,
            g.policy,
            g.tool_name,
            COUNT(*)::bigint AS attempts,
            MIN(g.created_at) AS first_seen,
            MAX(g.created_at) AS last_seen,
            COALESCE((ARRAY_AGG(g.reason ORDER BY g.created_at DESC))[1], '') AS sample_reason
        FROM governance_decisions g
        LEFT JOIN users u ON u.id = g.user_id
        WHERE g.decision = 'deny'
          AND g.created_at > now() - make_interval(secs => $1::double precision)
        GROUP BY g.agent_id, g.user_id, u.display_name, u.full_name, u.name, u.email,
                 g.policy, g.tool_name
        ORDER BY attempts DESC, last_seen DESC
        LIMIT $2",
    )
    .bind(window_seconds)
    .bind(limit)
    .fetch_all(pool)
    .await
}
