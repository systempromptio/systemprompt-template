//! The retention block on `/admin/configuration`: every window the
//! `database_cleanup` job enforces, read from the profile, beside what the
//! last run deleted and when the next one is due. Read-only — the profile is
//! the source; the manual trigger is `systemprompt infra jobs run
//! database_cleanup`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::manifest::profile::RetentionConfig;

use crate::error::AdminResult;
use crate::repositories::jobs::find_job_run;

const JOB_NAME: &str = "database_cleanup";

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RetentionRowView {
    pub table: &'static str,
    pub purpose: &'static str,
    pub days: u32,
    pub days_source: &'static str,
    // Why: None until a run has reported; the job's `+` suffix survives to say
    // it stopped at its time budget with more to do.
    pub deleted_last_run: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RetentionView {
    pub job_name: &'static str,
    pub rows: Vec<RetentionRowView>,
    pub last_run: Option<DateTime<Utc>>,
    pub next_run: Option<DateTime<Utc>>,
    pub last_status: Option<String>,
    pub last_status_tone: &'static str,
    pub last_error: Option<String>,
    pub orphaned_logs_last_run: Option<String>,
    pub never_ran: bool,
}

pub(crate) async fn retention_view(
    pool: &PgPool,
    retention: &RetentionConfig,
    ai_history_days: u32,
) -> AdminResult<RetentionView> {
    let run = find_job_run(pool, JOB_NAME).await?;
    let counts: Vec<(String, String)> = run
        .as_ref()
        .and_then(|r| r.last_message.as_deref())
        .map(parse_counts)
        .unwrap_or_default();
    let rows = retention_rows(retention, ai_history_days, &counts);
    Ok(RetentionView {
        job_name: JOB_NAME,
        rows,
        last_run: run.as_ref().and_then(|r| r.last_run),
        next_run: run.as_ref().and_then(|r| r.next_run),
        last_status: run.as_ref().and_then(|r| r.last_status.clone()),
        last_status_tone: match run.as_ref().and_then(|r| r.last_status.as_deref()) {
            Some("success") => "ok",
            Some(_) => "warn",
            None => "muted",
        },
        last_error: run.as_ref().and_then(|r| r.last_error.clone()),
        orphaned_logs_last_run: deleted_count(&counts, "orphaned_logs"),
        never_ran: run.as_ref().is_none_or(|r| r.last_run.is_none()),
    })
}

// Why: the row list is the bulk of the page and is pure formatting over the
// config and the last run's counts, so it is built apart from the query that
// fetches them.
fn retention_rows(
    retention: &RetentionConfig,
    ai_history_days: u32,
    counts: &[(String, String)],
) -> Vec<RetentionRowView> {
    let deleted = |table: &str| deleted_count(counts, table);
    let (messages_days, messages_source) = retention.ai_request_messages_days.map_or(
        (ai_history_days, "services ai.history.retention_days"),
        |days| (days, "profile retention.ai_request_messages_days"),
    );
    vec![
        RetentionRowView {
            table: "logs",
            purpose: "server log lines",
            days: retention.logs_days,
            days_source: "profile retention.logs_days",
            deleted_last_run: deleted("logs"),
        },
        RetentionRowView {
            table: "analytics_events",
            purpose: "traffic and behaviour events",
            days: retention.analytics_events_days,
            days_source: "profile retention.analytics_events_days",
            deleted_last_run: deleted("analytics_events"),
        },
        RetentionRowView {
            table: "ai_request_messages",
            purpose: "stored prompt and completion bodies",
            days: messages_days,
            days_source: messages_source,
            deleted_last_run: deleted("ai_request_messages"),
        },
        RetentionRowView {
            table: "mcp_tool_executions",
            purpose: "tool call records",
            days: retention.mcp_tool_executions_days,
            days_source: "profile retention.mcp_tool_executions_days",
            deleted_last_run: deleted("mcp_tool_executions"),
        },
        RetentionRowView {
            table: "event_outbox",
            purpose: "processed durable events",
            days: retention.outbox_processed_days,
            days_source: "profile retention.outbox_processed_days",
            deleted_last_run: deleted("event_outbox"),
        },
        RetentionRowView {
            table: "ai_request_payloads",
            purpose: "raw request/response bodies released; excerpts and hashes stay",
            days: retention.ai_request_payload_raw_days,
            days_source: "profile retention.ai_request_payload_raw_days",
            deleted_last_run: deleted("ai_request_payloads"),
        },
        RetentionRowView {
            table: "governance_decisions",
            purpose: "policy decisions per call",
            days: retention.governance_decisions_days,
            days_source: "profile retention.governance_decisions_days",
            deleted_last_run: deleted("governance_decisions"),
        },
        RetentionRowView {
            table: "ai_quota_buckets",
            purpose: "spent quota windows",
            days: 62,
            days_source: "fixed: longest quota window plus carry-forward",
            deleted_last_run: deleted("ai_quota_buckets"),
        },
    ]
}

// Why: what the last run deleted from one table, as the job reported it.
fn deleted_count(counts: &[(String, String)], table: &str) -> Option<String> {
    counts
        .iter()
        .find(|(name, _)| name == table)
        .map(|(_, value)| value.clone())
}

// Why: `table=count` pairs, the shape of the job's success message.
fn parse_counts(message: &str) -> Vec<(String, String)> {
    message
        .split_whitespace()
        .filter_map(|part| part.split_once('='))
        .map(|(name, value)| (name.to_owned(), value.to_owned()))
        .collect()
}
