//! The paged governance log, one row per governed *call* rather than per policy
//! evaluation.
//!
//! `governance_decisions` is written by three independent producers — the
//! gateway prompt gate, the authz webhook in this fork, and core's authz audit
//! sink — and all three record against the same `trace_id`. Read a row at a
//! time, one request is three near-identical lines and the log is unreadable;
//! read a trace at a time, it is one line that says what was asked, what each
//! producer decided, and whether anything objected. Grouping is therefore the
//! page's job, not a nicety.
//!
//! Rows written before core carried `trace_id` group on their own id instead,
//! so they appear as honest calls of one evaluation. `session_id` is
//! deliberately *not* a fallback key: the gateway gate and the authz webhook do
//! not agree on what a session is, which is the overloading `trace_id` was
//! added to end.
//!
//! [`super::decision_log`] keeps the flat, row-per-evaluation query. It is not
//! dead: the CSV export serves it, because an evidence export wants every
//! evaluation while the console wants the summary.

use chrono::{DateTime, Utc};
use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

use crate::repositories::scope::SubjectScope;
use crate::util::time_range::TimeRange;

use super::decision_log::DecisionFilter;

/// One policy evaluation within a call, as `jsonb_agg` emits it.
#[derive(Debug, Clone, Deserialize)]
pub struct ChainEvaluation {
    pub id: String,
    pub policy: String,
    pub decision: String,
    pub reason: String,
    pub tool_name: String,
    pub entity_type: Option<String>,
}

/// One governed call: every evaluation that shared its trace, as a single row.
#[derive(Debug, Clone)]
pub struct DecisionCallRow {
    pub call_key: String,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub eval_count: i64,
    pub deny_count: i64,
    pub warn_count: i64,
    // Why: `MIN(user_id)` over the group. All three producers write the calling
    // identity, so a trace has one caller; if that ever stopped being true the
    // group would show one of them and hide the rest.
    pub user_id: UserId,
    pub user_label: String,
    pub trace_id: Option<String>,
    pub worst_id: String,
    pub worst_decision: String,
    pub worst_policy: String,
    pub worst_reason: String,
    pub worst_tool: String,
    pub worst_entity_type: Option<String>,
    pub agent_scope: Option<String>,
    pub chain: Json<Vec<ChainEvaluation>>,
}

// Why: calls in the window, newest first, and how many the filter selected.
// The filter selects a call that *contains* a matching evaluation and still
// returns the whole chain, so narrowing to one policy shows the call that
// policy ran in rather than a fragment of it.
pub async fn list_decision_calls_paged(
    pool: &PgPool,
    range: TimeRange,
    scope: &SubjectScope,
    filter: &DecisionFilter,
    page: super::DecisionPage,
) -> Result<(Vec<DecisionCallRow>, i64), sqlx::Error> {
    let rows = sqlx::query_file_as!(
        DecisionCallRow,
        "src/repositories/governance/decision_calls/page.sql",
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

    let total = sqlx::query_file_scalar!(
        "src/repositories/governance/decision_calls/count.sql",
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
