//! The paged governance decision log: one row per evaluated tool call.
//!
//! The rollups in [`super::warnings`] answer "how much"; this answers "which
//! call, and why". Both read `governance_decisions`, and they are separate
//! because a rollup that also had to be paginated would have to choose between
//! being complete and being a page, and the log needs to be both.
//!
//! Sorting is expressed as ordered `CASE` arms rather than as interpolated SQL
//! because `sqlx::query_as!` verifies static text only: a sort key spliced into
//! the string would be unverified and, being caller-supplied, an injection
//! site. The arms cost one comparison per row and keep the whole statement
//! checked at compile time.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{CallId, SessionId, UserId};

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

/// One evaluated call, as the decisions tab renders it.
#[derive(Debug, Clone)]
pub struct DecisionLogRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub decision: String,
    pub policy: String,
    pub tool_name: String,
    pub user_id: UserId,
    pub agent_scope: Option<String>,
    pub reason: String,
    pub trace_id: Option<String>,
    pub session_id: SessionId,
    pub call_id: Option<CallId>,
    pub evidence: String,
}

/// The KPI strip's numbers, over the same window and scope as the rows.
#[derive(Debug, Clone, Copy, Default)]
pub struct DecisionStats {
    pub evaluated: i64,
    // Why: the table counts calls, so the header must too, or the page
    // contradicts itself on screen.
    pub calls: i64,
    pub attention_calls: i64,
    pub allowed: i64,
    pub warned: i64,
    pub denied: i64,
    pub scope_denied: i64,
    pub secret_denied: i64,
    pub blocklist_denied: i64,
    pub rate_denied: i64,
    pub distinct_users: i64,
}

/// What the toolbar narrowed the log to.
#[derive(Debug, Clone, Default)]
pub struct DecisionFilter {
    pub policy: Option<String>,
    pub decision: Option<String>,
    pub search: Option<String>,
    // Why: narrow to calls that *contain* a deny or a warn. Distinct from
    // `decision`, which asks what the call's worst evaluation was — a call can
    // hold a warn and still end up allowed.
    pub attention: bool,
}

/// Column and direction the log is ordered by.
#[derive(Debug, Clone, Copy)]
pub struct DecisionSort {
    pub key: &'static str,
    pub ascending: bool,
}

impl Default for DecisionSort {
    fn default() -> Self {
        Self {
            key: "when",
            ascending: false,
        }
    }
}

pub async fn list_governance_decisions_paged(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
    filter: &DecisionFilter,
    page: super::DecisionPage,
) -> Result<(Vec<DecisionLogRow>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        DecisionLogRow,
        r#"SELECT g.id, g.created_at, g.decision, g.policy, g.tool_name,
                  g.user_id AS "user_id!: UserId", g.agent_scope, g.reason, g.trace_id,
                  g.session_id AS "session_id!: SessionId",
                  g.evaluated_rules ->> 'call_id' AS "call_id: CallId",
                  g.evaluated_rules::TEXT AS "evidence!"
           FROM governance_decisions g
           WHERE g.created_at >= $1 AND g.created_at < $2
             AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR g.policy = $4)
             AND ($5::TEXT IS NULL OR g.decision = $5)
             AND ($6::TEXT IS NULL
                  OR g.tool_name ILIKE '%' || $6 || '%'
                  OR g.user_id ILIKE '%' || $6 || '%'
                  OR g.reason ILIKE '%' || $6 || '%')
             AND (NOT $11::BOOL OR (g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')))
           ORDER BY
             CASE WHEN $7 = 'policy'   AND $8 THEN g.policy END ASC,
             CASE WHEN $7 = 'policy'   AND NOT $8 THEN g.policy END DESC,
             CASE WHEN $7 = 'decision' AND $8 THEN g.decision END ASC,
             CASE WHEN $7 = 'decision' AND NOT $8 THEN g.decision END DESC,
             CASE WHEN $7 = 'tool'     AND $8 THEN g.tool_name END ASC,
             CASE WHEN $7 = 'tool'     AND NOT $8 THEN g.tool_name END DESC,
             CASE WHEN $7 = 'user'     AND $8 THEN g.user_id END ASC,
             CASE WHEN $7 = 'user'     AND NOT $8 THEN g.user_id END DESC,
             CASE WHEN $8 THEN g.created_at END ASC,
             g.created_at DESC
           LIMIT $9 OFFSET $10"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.policy.as_deref(),
        filter.decision.as_deref(),
        filter.search.as_deref(),
        page.sort.key,
        page.sort.ascending,
        page.slice.limit,
        page.slice.offset,
        filter.attention,
    )
    .fetch_all(pool)
    .await?;

    let total = sqlx::query_scalar!(
        r#"SELECT COUNT(*)::BIGINT AS "n!"
           FROM governance_decisions g
           WHERE g.created_at >= $1 AND g.created_at < $2
             AND ($3::TEXT[] IS NULL OR g.user_id = ANY($3))
             AND ($4::TEXT IS NULL OR g.policy = $4)
             AND ($5::TEXT IS NULL OR g.decision = $5)
             AND ($6::TEXT IS NULL
                  OR g.tool_name ILIKE '%' || $6 || '%'
                  OR g.user_id ILIKE '%' || $6 || '%'
                  OR g.reason ILIKE '%' || $6 || '%')
             AND (NOT $7::BOOL OR (g.decision = 'deny' OR (g.decision = 'warn' AND g.reason NOT LIKE 'secret detected: High-entropy token%')))"#,
        range.from,
        range.to,
        scope.as_sql(),
        filter.policy.as_deref(),
        filter.decision.as_deref(),
        filter.search.as_deref(),
        filter.attention,
    )
    .fetch_one(pool)
    .await?;

    Ok((rows, total))
}

// Why: the four per-policy counts are the four synchronous chain stages, named
// as the policy writes them. They are counted here rather than derived from the
// page of rows because a page is fifty rows and the deny rate is a property of
// the window.
pub async fn get_decision_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<DecisionStats, sqlx::Error> {
    sqlx::query_as!(
        DecisionStats,
        r#"SELECT
             COUNT(*)::BIGINT AS "evaluated!",
             COUNT(*) FILTER (WHERE decision = 'allow')::BIGINT AS "allowed!",
             COUNT(*) FILTER (WHERE decision = 'warn')::BIGINT AS "warned!",
             COUNT(*) FILTER (WHERE decision = 'deny')::BIGINT AS "denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'agent_scope')::BIGINT
               AS "scope_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'secret_scan')::BIGINT
               AS "secret_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'tool_blocklist')::BIGINT
               AS "blocklist_denied!",
             COUNT(*) FILTER (WHERE decision = 'deny' AND policy = 'rate_limit')::BIGINT
               AS "rate_denied!",
             COUNT(DISTINCT user_id)::BIGINT AS "distinct_users!",
             COUNT(DISTINCT COALESCE(NULLIF(trace_id, ''), id))::BIGINT AS "calls!",
             COUNT(DISTINCT 'review:' || md5(user_id || ':' || session_id || ':' || policy || ':' || COALESCE(substring(reason from 'fingerprint:([0-9a-f]{64})'), reason)))
               FILTER (WHERE (decision = 'deny' OR (decision = 'warn' AND reason NOT LIKE 'secret detected: High-entropy token%')))::BIGINT
               AS "attention_calls!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
             AND ($3::TEXT[] IS NULL OR user_id = ANY($3))"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_one(pool)
    .await
}

/// One policy's showing in the window, for the chip strip.
#[derive(Debug, Clone)]
pub struct PolicyCount {
    pub policy: String,
    pub total: i64,
    pub denied: i64,
    pub warned: i64,
}

// Why: the chip strip used to name the four synchronous chain stages and count
// only those, so on an instance whose traffic is authz and gateway decisions it
// showed four zeroes beside a full log. Counting what the window actually holds
// means a chip appears because a policy fired, and a producer nobody knew about
// announces itself instead of hiding behind an em-dash.
pub async fn list_decision_policy_counts(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<PolicyCount>, sqlx::Error> {
    sqlx::query_as!(
        PolicyCount,
        r#"SELECT policy AS "policy!",
                  COUNT(*)::BIGINT AS "total!",
                  COUNT(*) FILTER (WHERE decision = 'deny')::BIGINT AS "denied!",
                  COUNT(*) FILTER (WHERE decision = 'warn')::BIGINT AS "warned!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
             AND ($3::TEXT[] IS NULL OR user_id = ANY($3))
           GROUP BY policy
           ORDER BY COUNT(*) FILTER (WHERE decision IN ('deny', 'warn')) DESC,
                    COUNT(*) DESC, policy"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}

// Why: built from the window rather than from a fixed list, so a policy
// that stops firing stops being offered as a filter.
pub async fn list_decision_policies(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r#"SELECT policy AS "policy!"
           FROM governance_decisions
           WHERE created_at >= $1 AND created_at < $2
             AND ($3::TEXT[] IS NULL OR user_id = ANY($3))
           GROUP BY policy
           ORDER BY COUNT(*) DESC, policy"#,
        range.from,
        range.to,
        scope.as_sql(),
    )
    .fetch_all(pool)
    .await
}
