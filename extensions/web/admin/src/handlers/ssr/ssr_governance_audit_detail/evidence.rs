//! The two evidence planes the audit chain does not carry.
//!
//! The governance chain is request-time and already in the envelope. The
//! gateway's safety scanners are a second, independent plane that runs in both
//! directions and records every finding whether or not it blocked; the
//! tool-call list is what the model actually asked to run. Both are keyed on
//! the `ai_requests.id` values in the chain, so one page load reads them for
//! every request in the session at once.

use serde::Serialize;
use sqlx::PgPool;

use crate::repositories::analytics::requests::{
    list_request_safety_findings, list_request_tool_calls,
};

#[derive(Debug, Default, Serialize)]
pub(super) struct EvidenceView {
    pub findings: Vec<FindingView>,
    pub has_findings: bool,
    pub finding_count: usize,
    pub blocked_count: usize,
    pub tool_calls: Vec<ToolCallView>,
    pub has_tool_calls: bool,
    pub tool_call_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct FindingView {
    pub phase: String,
    pub severity: String,
    pub category: String,
    pub scanner: String,
    pub excerpt: String,
    pub blocked: bool,
    // Why: `warn` mode audits a finding without acting on it, and the two are
    // read very differently — the badge has to say which one this row is.
    pub outcome: &'static str,
    pub tone: &'static str,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ToolCallView {
    pub tool_name: String,
    pub sequence_number: i32,
    pub mcp_execution_id: Option<String>,
    pub result_label: &'static str,
    pub created_at: String,
}

// Why: a failed evidence read degrades to an empty section rather than taking
// the audit page with it — the chain above it is the part an operator came for.
pub(super) async fn load(pool: &PgPool, request_ids: &[String]) -> EvidenceView {
    if request_ids.is_empty() {
        return EvidenceView::default();
    }
    let (findings_res, calls_res) = tokio::join!(
        list_request_safety_findings(pool, request_ids),
        list_request_tool_calls(pool, request_ids),
    );

    let findings: Vec<FindingView> = findings_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_request_safety_findings failed");
            Vec::new()
        })
        .into_iter()
        .map(|f| FindingView {
            phase: f.phase,
            severity: f.severity,
            category: f.category,
            scanner: f.scanner,
            excerpt: f.excerpt.unwrap_or_default(),
            outcome: if f.blocked { "BLOCKED" } else { "AUDITED" },
            tone: if f.blocked { "err" } else { "warn" },
            blocked: f.blocked,
            created_at: local_time(f.created_at),
        })
        .collect();

    let tool_calls: Vec<ToolCallView> = calls_res
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "list_request_tool_calls failed");
            Vec::new()
        })
        .into_iter()
        .map(|t| ToolCallView {
            tool_name: t.tool_name,
            sequence_number: t.sequence_number,
            mcp_execution_id: t.mcp_execution_id,
            result_label: if t.has_result { "recorded" } else { "none" },
            created_at: local_time(t.created_at),
        })
        .collect();

    EvidenceView {
        blocked_count: findings.iter().filter(|f| f.blocked).count(),
        finding_count: findings.len(),
        has_findings: !findings.is_empty(),
        findings,
        tool_call_count: tool_calls.len(),
        has_tool_calls: !tool_calls.is_empty(),
        tool_calls,
    }
}

fn local_time(at: chrono::DateTime<chrono::Utc>) -> String {
    at.with_timezone(&chrono::Local)
        .format("%Y-%m-%d %H:%M:%S")
        .to_string()
}
