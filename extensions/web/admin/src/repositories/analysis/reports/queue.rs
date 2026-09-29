//! The `analysis_reports` queue: a request from the console, a fenced lease
//! for the job, completion with the model's findings, and retry. Every
//! statement is bound; the job never interpolates.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;

use super::{ReportDigestInputs, ReportFindings};

/// One request as the console makes it.
#[derive(Debug, Clone)]
pub struct NewReport {
    pub scope_kind: String,
    pub scope_id: Option<String>,
    pub scope_label: Option<String>,
    pub window_start: DateTime<Utc>,
    pub window_end: DateTime<Utc>,
    pub requested_by: String,
    pub inputs: ReportDigestInputs,
    // Why: the generation task holds this lease from the insert, so the one
    // completion path (`update_report_completion`) stays fenced.
    pub lease_token: String,
}

/// What the job writes back after the one model call.
#[derive(Debug)]
pub struct ReportCompletion<'a> {
    pub id: &'a str,
    pub lease_token: &'a str,
    pub findings: &'a ReportFindings,
    pub provider: &'a str,
    pub model: &'a str,
    pub ai_request_id: Option<&'a str>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
}

pub async fn insert_report_request(
    pool: &PgPool,
    report: NewReport,
) -> Result<String, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    sqlx::query!(
        r#"INSERT INTO analysis_reports
               (id, scope_kind, scope_id, scope_label, window_start, window_end, requested_by, inputs,
                lease_token, lease_until, attempts)
           VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, clock_timestamp() + interval '10 minutes', 1)"#,
        id,
        report.scope_kind,
        report.scope_id,
        report.scope_label,
        report.window_start,
        report.window_end,
        report.requested_by,
        Json(report.inputs) as _,
        report.lease_token
    )
    .execute(pool)
    .await?;
    Ok(id)
}

// Why: the call's cost is settled by the gateway after the response returns,
// so it is read from ai_requests here; a later run may re-read it.
pub async fn update_report_completion(
    pool: &PgPool,
    done: ReportCompletion<'_>,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!(
        r#"UPDATE analysis_reports
           SET status = 'generated', findings = $3, provider = $4, model = $5, ai_request_id = $6,
               input_tokens = $7, output_tokens = $8,
               cost_microdollars = (SELECT r.cost_microdollars FROM ai_requests r WHERE r.id = $6),
               generated_at = clock_timestamp(), lease_token = NULL, lease_until = NULL,
               last_error = NULL, updated_at = clock_timestamp()
           WHERE id = $1 AND lease_token = $2"#,
        done.id,
        done.lease_token,
        Json(done.findings) as _,
        done.provider,
        done.model,
        done.ai_request_id,
        done.input_tokens,
        done.output_tokens
    )
    .execute(pool)
    .await?;
    Ok(result.rows_affected() > 0)
}

// Why: a report is written once, in the moment it was asked for; a failure
// is recorded with its reason and the page offers Retry, which is a fresh
// request rather than a hidden retry loop.
pub async fn fail_report(
    pool: &PgPool,
    id: &str,
    lease_token: &str,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE analysis_reports
           SET status = 'failed', lease_token = NULL, lease_until = NULL,
               last_error = LEFT($3, 500), updated_at = clock_timestamp()
           WHERE id = $1 AND lease_token = $2"#,
        id,
        lease_token,
        error
    )
    .execute(pool)
    .await?;
    Ok(())
}
