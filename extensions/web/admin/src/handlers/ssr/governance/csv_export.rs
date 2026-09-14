//! The governance CSV export.
//!
//! Split from the page so the handler module stays inside its size ceiling, and
//! because the export answers a different question from the screen: it stays
//! one row per policy evaluation while the console folds them into calls.
//! Evidence wants everything that was decided; a console wants the summary.

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use sqlx::PgPool;
use systemprompt::identifiers::{CallId, SessionId};

use crate::error::AdminResult;
use crate::handlers::ssr::csv::CsvBuilder;
use crate::repositories::governance::decision_log::DecisionLogRow;
use crate::repositories::governance::findings::SafetyFindingLogRow;
use crate::repositories::governance::{DecisionPage, PageSlice};
use crate::types::UserContext;

use super::{
    GovernanceQuery, decision_filter, finding_filter, range_of, require_console, resolve_scope,
    sort_from, view,
};

// Why: an export is a bounded read, not a stream. Five thousand evaluations is
// several times the largest window an operator opens and still a file a
// spreadsheet will accept.
const CSV_LIMIT: i64 = 5_000;

pub(crate) async fn governance_csv(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<GovernanceQuery>,
) -> AdminResult<Response> {
    require_console(&user_ctx)?;

    let range = range_of(&query);
    let (_, scope) = resolve_scope(&pool, &user_ctx, &query).await?;

    let (decisions, decision_total) =
        crate::repositories::governance::decision_log::list_governance_decisions_paged(
            &pool,
            range,
            &scope,
            &decision_filter(&query),
            DecisionPage {
                sort: sort_from(&query),
                slice: PageSlice::first(CSV_LIMIT),
            },
        )
        .await?;
    let (findings, finding_total) =
        crate::repositories::governance::findings::list_safety_findings_paged(
            &pool,
            range,
            &scope,
            &finding_filter(&query),
            PageSlice::first(CSV_LIMIT),
        )
        .await?;

    // Why: the export stays one row per evaluation while the console folds them
    // into calls. An evidence export wants everything that was decided, not the
    // summary the screen shows, and `trace_id` is here so the grouping the
    // console performs is reproducible from the file.
    let mut csv = CsvBuilder::new(&[
        "plane",
        "at",
        "trace_id",
        "outcome",
        "policy_or_category",
        "stage_or_scanner",
        "tool_or_model",
        "user",
        "scope",
        "reason",
        "record_id",
        "session_id",
        "call_id",
        "request_id",
        "phase",
        "evaluated_rules",
        "export_truncated",
        "matching_decisions",
        "matching_findings",
    ]);
    let totals = ExportTotals {
        truncated: (decision_total > CSV_LIMIT || finding_total > CSV_LIMIT).to_string(),
        decisions: decision_total.to_string(),
        findings: finding_total.to_string(),
    };
    for row in &decisions {
        push_decision_row(&mut csv, row, &totals);
    }
    for row in &findings {
        push_finding_row(&mut csv, row, &totals);
    }

    Ok(csv.into_response(&format!("governance-{}.csv", range.from.format("%Y%m%d"))))
}

// Why: repeat completeness totals so each exported line can be interpreted
// independently.
struct ExportTotals {
    truncated: String,
    decisions: String,
    findings: String,
}

fn push_decision_row(csv: &mut CsvBuilder, row: &DecisionLogRow, totals: &ExportTotals) {
    csv.row(&[
        "chain",
        &row.created_at.to_rfc3339(),
        row.trace_id.as_deref().unwrap_or(""),
        &row.decision,
        &row.policy,
        &view::plane_of(&row.policy),
        &row.tool_name,
        row.user_id.as_str(),
        row.agent_scope.as_deref().unwrap_or(""),
        &row.reason,
        &row.id,
        row.session_id.as_str(),
        row.call_id.as_ref().map_or("", CallId::as_str),
        "",
        "request",
        &row.evidence,
        &totals.truncated,
        &totals.decisions,
        &totals.findings,
    ]);
}

fn push_finding_row(csv: &mut CsvBuilder, row: &SafetyFindingLogRow, totals: &ExportTotals) {
    csv.row(&[
        "safety",
        &row.created_at.to_rfc3339(),
        row.trace_id.as_deref().unwrap_or(""),
        if row.blocked { "blocked" } else { "audited" },
        &row.category,
        &row.scanner,
        row.model.as_deref().unwrap_or(""),
        row.user_id.as_ref().map_or("", |u| u.as_str()),
        "",
        row.excerpt.as_deref().unwrap_or(""),
        &row.id,
        row.session_id.as_ref().map_or("", SessionId::as_str),
        "",
        &row.ai_request_id,
        &row.phase,
        "",
        &totals.truncated,
        &totals.decisions,
        &totals.findings,
    ]);
}
