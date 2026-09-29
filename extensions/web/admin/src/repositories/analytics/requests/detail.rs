//! Per-request evidence the audit-detail page shows beneath the policy chain.
//!
//! Two planes, read separately because they are written separately: the
//! governance chain is request-time and already carried by the chain
//! envelope, while the gateway's safety scanners run in both directions and
//! land in `ai_safety_findings` whether or not they blocked anything. The
//! tool-call list is the third: what the model actually asked to run.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::ArtifactId;

/// One scanner hit against a request, in either direction.
#[derive(Debug, Clone)]
pub struct SafetyFindingRow {
    pub phase: String,
    pub severity: String,
    pub category: String,
    pub scanner: String,
    pub excerpt: Option<String>,
    pub blocked: bool,
    pub created_at: DateTime<Utc>,
}

/// One tool the model asked for, in the order it asked, with the execution
/// and artifact the ledger joined to it by the client's `tool_use_id`.
#[derive(Debug, Clone)]
pub struct RequestToolCallRow {
    pub tool_name: String,
    pub sequence_number: i32,
    pub mcp_execution_id: Option<String>,
    pub artifact_id: Option<ArtifactId>,
    pub state: String,
    pub execution_status: Option<String>,
    pub is_structured: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn list_request_safety_findings(
    pool: &PgPool,
    ai_request_ids: &[String],
) -> Result<Vec<SafetyFindingRow>, sqlx::Error> {
    sqlx::query_as!(
        SafetyFindingRow,
        r#"SELECT phase AS "phase!", severity AS "severity!", category AS "category!",
                  scanner AS "scanner!", excerpt, blocked AS "blocked!",
                  created_at AS "created_at!"
           FROM ai_safety_findings
           WHERE ai_request_id = ANY($1)
           ORDER BY created_at ASC
           LIMIT 200"#,
        ai_request_ids
    )
    .fetch_all(pool)
    .await
}

pub async fn list_request_tool_calls(
    pool: &PgPool,
    ai_request_ids: &[String],
) -> Result<Vec<RequestToolCallRow>, sqlx::Error> {
    sqlx::query_as!(
        RequestToolCallRow,
        r#"SELECT t.tool_name AS "tool_name!", t.sequence_number AS "sequence_number!",
                  l.mcp_execution_id,
                  l.artifact_id AS "artifact_id: ArtifactId",
                  COALESCE(l.state, 'intended') AS "state!",
                  l.execution_status,
                  COALESCE(l.is_structured, FALSE) AS "is_structured!",
                  t.created_at AS "created_at!"
           FROM ai_request_tool_calls t
           LEFT JOIN tool_call_ledger l ON l.intent_id = t.id
           WHERE t.request_id = ANY($1)
           ORDER BY t.created_at ASC, t.sequence_number ASC
           LIMIT 200"#,
        ai_request_ids
    )
    .fetch_all(pool)
    .await
}
