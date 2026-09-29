//! One artifact, read for `/admin/artifacts/{artifact_id}` and its preview.
//!
//! The list pages read `repositories::analysis::tools` (the `tool_activity`
//! view); this module reads the one row `mcp_artifacts` holds for an
//! artifact with its stored body, the execution and ledger rows of the same
//! call, the one artifact rule's verdict on it, the scanner findings and the
//! governance decision keyed to the call.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ArtifactId, ContextId, McpExecutionId, SessionId, UserId};

/// One scanner finding raised when the artifact was ingested.
#[derive(Debug, Clone)]
pub struct ArtifactFindingRow {
    pub phase: String,
    pub severity: String,
    pub category: String,
    pub scanner: String,
    pub path: Option<String>,
    pub excerpt: Option<String>,
    pub redacted: bool,
    pub created_at: DateTime<Utc>,
}

/// The artifact row with everything the detail page shows beside it.
#[derive(Debug, Clone)]
pub struct ArtifactDetail {
    pub artifact_id: ArtifactId,
    pub mcp_execution_id: McpExecutionId,
    pub ai_tool_call_id: Option<String>,
    pub request_id: Option<String>,
    pub user_id: Option<UserId>,
    pub user_label: Option<String>,
    pub session_id: Option<SessionId>,
    pub context_id: Option<ContextId>,
    pub trace_id: Option<String>,
    pub tool_name: Option<String>,
    pub server_name: String,
    pub artifact_type: String,
    pub artifact_title: Option<String>,
    pub source: String,
    pub last_seen_source: Option<String>,
    pub correlation: Option<String>,
    pub client_kind: Option<String>,
    pub is_structured: bool,
    pub has_ui_resource: bool,
    pub is_error: bool,
    pub payload_bytes: Option<i32>,
    pub payload_sha256: Option<String>,
    pub secret_redactions: i32,
    // JSON: the stored artifact body, typed per `artifact_type`.
    pub body: Option<serde_json::Value>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i32>,
    pub error_message: Option<String>,
    pub intended_at: Option<DateTime<Utc>>,
    pub artifact_kind: Option<String>,
    pub input_summary: Option<String>,
    pub is_builtin: bool,
    pub governance_decision_id: Option<String>,
    pub governance_decision: Option<String>,
    pub skill: Option<String>,
    pub created_at: DateTime<Utc>,
    pub findings: Vec<ArtifactFindingRow>,
}

// Why: the statement's own row; `body` arrives as `Json` and is unwrapped
// into the detail so callers see one plain struct.
struct DetailRecord {
    artifact_id: ArtifactId,
    mcp_execution_id: McpExecutionId,
    ai_tool_call_id: Option<String>,
    request_id: Option<String>,
    user_id: Option<UserId>,
    user_label: Option<String>,
    session_id: Option<SessionId>,
    context_id: Option<ContextId>,
    trace_id: Option<String>,
    tool_name: Option<String>,
    server_name: String,
    artifact_type: String,
    artifact_title: Option<String>,
    source: String,
    last_seen_source: Option<String>,
    correlation: Option<String>,
    client_kind: Option<String>,
    is_structured: bool,
    has_ui_resource: bool,
    is_error: bool,
    payload_bytes: Option<i32>,
    payload_sha256: Option<String>,
    secret_redactions: i32,
    // JSON: the stored artifact body as the driver hands it over.
    body: Option<Json<serde_json::Value>>,
    execution_status: Option<String>,
    execution_time_ms: Option<i32>,
    error_message: Option<String>,
    intended_at: Option<DateTime<Utc>>,
    artifact_kind: Option<String>,
    input_summary: Option<String>,
    is_builtin: bool,
    governance_decision_id: Option<String>,
    governance_decision: Option<String>,
    skill: Option<String>,
    created_at: DateTime<Utc>,
}

pub async fn find_artifact(
    pool: &PgPool,
    artifact_id: &ArtifactId,
) -> Result<Option<ArtifactDetail>, sqlx::Error> {
    let Some(r) = sqlx::query_file_as!(
        DetailRecord,
        "src/repositories/analytics/artifacts/detail.sql",
        artifact_id.as_str()
    )
    .fetch_optional(pool)
    .await?
    else {
        return Ok(None);
    };
    let findings = sqlx::query_as!(
        ArtifactFindingRow,
        r#"SELECT phase, severity, category, scanner, path, excerpt, redacted, created_at
           FROM mcp_artifact_findings WHERE artifact_id = $1 ORDER BY created_at"#,
        artifact_id.as_str()
    )
    .fetch_all(pool)
    .await?;
    Ok(Some(ArtifactDetail {
        artifact_id: r.artifact_id,
        mcp_execution_id: r.mcp_execution_id,
        ai_tool_call_id: r.ai_tool_call_id,
        request_id: r.request_id,
        user_id: r.user_id,
        user_label: r.user_label,
        session_id: r.session_id,
        context_id: r.context_id,
        trace_id: r.trace_id,
        tool_name: r.tool_name,
        server_name: r.server_name,
        artifact_type: r.artifact_type,
        artifact_title: r.artifact_title,
        source: r.source,
        last_seen_source: r.last_seen_source,
        correlation: r.correlation,
        client_kind: r.client_kind,
        is_structured: r.is_structured,
        has_ui_resource: r.has_ui_resource,
        is_error: r.is_error,
        payload_bytes: r.payload_bytes,
        payload_sha256: r.payload_sha256,
        secret_redactions: r.secret_redactions,
        body: r.body.map(|b| b.0),
        execution_status: r.execution_status,
        execution_time_ms: r.execution_time_ms,
        error_message: r.error_message,
        intended_at: r.intended_at,
        artifact_kind: r.artifact_kind,
        input_summary: r.input_summary,
        is_builtin: r.is_builtin,
        governance_decision_id: r.governance_decision_id,
        governance_decision: r.governance_decision,
        skill: r.skill,
        created_at: r.created_at,
        findings,
    }))
}
