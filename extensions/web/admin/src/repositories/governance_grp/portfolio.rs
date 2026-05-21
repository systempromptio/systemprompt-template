//! Aggregations for the Policy Decisions portfolio page.
//!
//! Three families of helpers are exposed:
//!
//! * [`fetch_governance_counts_in_range`] — total / allow / deny + per-policy
//!   counts for the supplied window.
//! * [`fetch_decision_buckets`] — equal-width time buckets across the window
//!   for the stacked-area chart and KPI sparklines, optionally filtered to a
//!   policy family.
//! * [`fetch_top_denies`] — top-N denies grouped by `tool_name` or `agent_scope`.

use serde::Serialize;
use sqlx::PgPool;

use super::time_range::TimeRange;

/// Counts within a single window, broken down by policy family.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct GovernanceCountsByPolicy {
    pub total: i64,
    pub allowed: i64,
    pub denied: i64,
    pub secret_scan: i64,
    pub blocklist: i64,
    pub rate_limit: i64,
}

/// Aggregate decision counts inside `range`, classifying by policy family.
pub async fn fetch_governance_counts_in_range(
    pool: &PgPool,
    range: TimeRange,
) -> Result<GovernanceCountsByPolicy, sqlx::Error> {
    let row = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "total!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            COUNT(*) FILTER (
                WHERE policy = 'secret_scan'
                   OR reason ILIKE '%secret%'
            )::bigint AS "secret_scan!",
            COUNT(*) FILTER (
                WHERE policy IN ('tool_blocklist', 'blocklist')
            )::bigint AS "blocklist!",
            COUNT(*) FILTER (
                WHERE policy = 'rate_limit'
            )::bigint AS "rate_limit!"
        FROM governance_decisions
        WHERE created_at >= $1 AND created_at < $2"#,
        range.from,
        range.to,
    )
    .fetch_one(pool)
    .await?;

    Ok(GovernanceCountsByPolicy {
        total: row.total,
        allowed: row.allowed,
        denied: row.denied,
        secret_scan: row.secret_scan,
        blocklist: row.blocklist,
        rate_limit: row.rate_limit,
    })
}

/// Filter for [`fetch_decision_buckets`].
#[derive(Debug, Clone, Copy, Default)]
pub struct BucketFilter {
    /// Restrict to rows whose `policy` is in this set (when non-empty).
    pub policies: BucketPolicyFilter,
    /// When true, additionally include rows whose reason matches the secret
    /// regex (used by the Secret-scan KPI sparkline).
    pub include_secret_reason: bool,
}

#[derive(Debug, Clone, Copy, Default)]
pub enum BucketPolicyFilter {
    #[default]
    All,
    SecretScan,
    Blocklist,
    RateLimit,
}

/// One time bucket with allow / deny counts.
#[derive(Debug, Clone, Copy, Serialize)]
pub struct DecisionBucket {
    pub bucket_index: i32,
    pub allow: i64,
    pub deny: i64,
}

/// Split `range` into `n_buckets` equal-width time buckets and count decisions.
///
/// Each returned row contains the allow / deny totals inside its bucket. Empty
/// buckets are returned with zero counts so the caller always gets exactly
/// `n_buckets` rows in index order.
pub async fn fetch_decision_buckets(
    pool: &PgPool,
    range: TimeRange,
    n_buckets: i32,
    filter: BucketFilter,
) -> Result<Vec<DecisionBucket>, sqlx::Error> {
    let buckets = n_buckets.max(1);

    let rows = sqlx::query!(
        r#"WITH params AS (
            SELECT $1::timestamptz AS lo,
                   $2::timestamptz AS hi,
                   $3::int          AS n
        ),
        series AS (
            SELECT generate_series(0, (SELECT n - 1 FROM params))::int AS idx
        ),
        decisions AS (
            SELECT
                LEAST(
                    width_bucket(
                        EXTRACT(EPOCH FROM g.created_at),
                        EXTRACT(EPOCH FROM (SELECT lo FROM params)),
                        EXTRACT(EPOCH FROM (SELECT hi FROM params)),
                        (SELECT n FROM params)
                    ) - 1,
                    (SELECT n - 1 FROM params)
                ) AS bucket_index,
                g.decision
            FROM governance_decisions g
            WHERE g.created_at >= (SELECT lo FROM params)
              AND g.created_at <  (SELECT hi FROM params)
              AND (
                    $4::text = 'all'
                 OR (
                       $4 = 'secret'
                       AND (
                            g.policy = 'secret_scan'
                            OR ($5::bool AND g.reason ILIKE '%secret%')
                       )
                    )
                 OR ($4 = 'blocklist'  AND g.policy IN ('tool_blocklist', 'blocklist'))
                 OR ($4 = 'rate_limit' AND g.policy = 'rate_limit')
              )
        ),
        agg AS (
            SELECT
                bucket_index,
                COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS allow_count,
                COUNT(*) FILTER (WHERE decision = 'deny')::bigint  AS deny_count
            FROM decisions
            GROUP BY bucket_index
        )
        SELECT
            s.idx                                   AS "bucket_index!",
            COALESCE(a.allow_count, 0)::bigint      AS "allow!",
            COALESCE(a.deny_count, 0)::bigint       AS "deny!"
        FROM series s
        LEFT JOIN agg a ON a.bucket_index = s.idx
        ORDER BY s.idx"#,
        range.from,
        range.to,
        buckets,
        bucket_filter_tag(filter.policies),
        filter.include_secret_reason,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| DecisionBucket {
            bucket_index: r.bucket_index,
            allow: r.allow,
            deny: r.deny,
        })
        .collect())
}

const fn bucket_filter_tag(p: BucketPolicyFilter) -> &'static str {
    match p {
        BucketPolicyFilter::All => "all",
        BucketPolicyFilter::SecretScan => "secret",
        BucketPolicyFilter::Blocklist => "blocklist",
        BucketPolicyFilter::RateLimit => "rate_limit",
    }
}

/// Group dimension for [`fetch_top_denies`].
#[derive(Debug, Clone, Copy)]
pub enum TopDenyGroup {
    Tool,
    AgentScope,
}

#[derive(Debug, Clone, Serialize)]
pub struct TopDeny {
    pub key: String,
    pub label: String,
    pub deny_count: i64,
}

/// Top-N denies grouped by tool or agent scope inside the window. NULL group
/// values are excluded.
pub async fn fetch_top_denies(
    pool: &PgPool,
    range: TimeRange,
    group_by: TopDenyGroup,
    limit: i64,
) -> Result<Vec<TopDeny>, sqlx::Error> {
    let group_expr = match group_by {
        TopDenyGroup::Tool => "g.tool_name",
        TopDenyGroup::AgentScope => "g.agent_scope",
    };

    let sql = format!(
        r"SELECT
            {group_expr} AS key,
            COUNT(*)::bigint AS deny_count
        FROM governance_decisions g
        WHERE g.decision = 'deny'
          AND g.created_at >= $1 AND g.created_at < $2
          AND {group_expr} IS NOT NULL
          AND {group_expr} <> ''
        GROUP BY {group_expr}
        ORDER BY deny_count DESC
        LIMIT $3",
    );

    let rows: Vec<(String, i64)> = sqlx::query_as(&sql)
        .bind(range.from)
        .bind(range.to)
        .bind(limit)
        .fetch_all(pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|(key, deny_count)| TopDeny {
            label: key.clone(),
            key,
            deny_count,
        })
        .collect())
}
