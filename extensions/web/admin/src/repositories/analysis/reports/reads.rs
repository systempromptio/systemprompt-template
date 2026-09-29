//! Reading `analysis_reports`: the history list, one report, and the latest
//! generated report for a scope (what the banner on the Conversations and
//! Skills pages shows). One record shape, one column list, so every read
//! decodes the same way.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;

use super::{AnalysisReportRow, ReportDigestInputs, ReportFindings};

// Why: the table row as SQLx decodes it; folded into `AnalysisReportRow` so
// the jsonb columns are typed once, here.
pub(super) struct ReportRecord {
    pub(super) id: String,
    pub(super) scope_kind: String,
    pub(super) scope_id: Option<String>,
    pub(super) scope_label: Option<String>,
    pub(super) window_start: DateTime<Utc>,
    pub(super) window_end: DateTime<Utc>,
    pub(super) status: String,
    pub(super) requested_by: String,
    pub(super) provider: Option<String>,
    pub(super) model: Option<String>,
    pub(super) ai_request_id: Option<String>,
    pub(super) input_tokens: Option<i32>,
    pub(super) output_tokens: Option<i32>,
    pub(super) cost_microdollars: Option<i64>,
    pub(super) inputs: Json<ReportDigestInputs>,
    pub(super) findings: Option<Json<ReportFindings>>,
    pub(super) attempts: i32,
    pub(super) lease_token: Option<String>,
    pub(super) last_error: Option<String>,
    pub(super) generated_at: Option<DateTime<Utc>>,
    pub(super) created_at: DateTime<Utc>,
    pub(super) updated_at: DateTime<Utc>,
}

impl From<ReportRecord> for AnalysisReportRow {
    fn from(r: ReportRecord) -> Self {
        Self {
            id: r.id,
            scope_kind: r.scope_kind,
            scope_id: r.scope_id,
            scope_label: r.scope_label,
            window_start: r.window_start,
            window_end: r.window_end,
            status: r.status,
            requested_by: r.requested_by,
            provider: r.provider,
            model: r.model,
            ai_request_id: r.ai_request_id,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cost_microdollars: r.cost_microdollars,
            inputs: r.inputs.0,
            findings: r.findings.map(|f| f.0),
            attempts: r.attempts,
            lease_token: r.lease_token,
            last_error: r.last_error,
            generated_at: r.generated_at,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}

pub async fn list_reports(
    pool: &PgPool,
    limit: i64,
) -> Result<Vec<AnalysisReportRow>, sqlx::Error> {
    let rows = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, scope_label, window_start, window_end, status,
                  requested_by, provider, model, ai_request_id, input_tokens, output_tokens,
                  cost_microdollars, inputs AS "inputs: Json<ReportDigestInputs>",
                  findings AS "findings?: Json<ReportFindings>", attempts, lease_token,
                  last_error, generated_at, created_at, updated_at
           FROM analysis_reports
           ORDER BY created_at DESC
           LIMIT $1"#,
        limit
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(AnalysisReportRow::from).collect())
}

pub async fn find_report(
    pool: &PgPool,
    id: &str,
) -> Result<Option<AnalysisReportRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, scope_label, window_start, window_end, status,
                  requested_by, provider, model, ai_request_id, input_tokens, output_tokens,
                  cost_microdollars, inputs AS "inputs: Json<ReportDigestInputs>",
                  findings AS "findings?: Json<ReportFindings>", attempts, lease_token,
                  last_error, generated_at, created_at, updated_at
           FROM analysis_reports
           WHERE id = $1"#,
        id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AnalysisReportRow::from))
}

// Why: the newest generated report for a scope, or the newest pending one
// when nothing has been generated yet, so a banner can say "queued" rather
// than "none".
pub async fn find_latest_generated_report(
    pool: &PgPool,
    scope_kind: &str,
    scope_id: Option<&str>,
) -> Result<Option<AnalysisReportRow>, sqlx::Error> {
    let row = sqlx::query_as!(
        ReportRecord,
        r#"SELECT id, scope_kind, scope_id, scope_label, window_start, window_end, status,
                  requested_by, provider, model, ai_request_id, input_tokens, output_tokens,
                  cost_microdollars, inputs AS "inputs: Json<ReportDigestInputs>",
                  findings AS "findings?: Json<ReportFindings>", attempts, lease_token,
                  last_error, generated_at, created_at, updated_at
           FROM analysis_reports
           WHERE scope_kind = $1 AND scope_id IS NOT DISTINCT FROM $2
             AND status IN ('generated', 'pending')
           ORDER BY (status = 'generated') DESC, created_at DESC
           LIMIT 1"#,
        scope_kind,
        scope_id
    )
    .fetch_optional(pool)
    .await?;
    Ok(row.map(AnalysisReportRow::from))
}
