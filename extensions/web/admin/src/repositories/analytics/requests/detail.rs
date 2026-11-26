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

// Why: a call the ledger has no row for is one the model asked for and nothing
// executed — `intended`. The ledger is the MCP member's table, so a build
// without that member reads every call that way.
pub async fn list_request_tool_calls(
    pool: &PgPool,
    ai_request_ids: &[String],
) -> Result<Vec<RequestToolCallRow>, sqlx::Error> {
    let intents = sqlx::query!(
        r#"SELECT t.id AS "id!", t.tool_name AS "tool_name!",
                  t.sequence_number AS "sequence_number!", t.created_at AS "created_at!"
           FROM ai_request_tool_calls t
           WHERE t.request_id = ANY($1)
           ORDER BY t.created_at ASC, t.sequence_number ASC
           LIMIT 200"#,
        ai_request_ids
    )
    .fetch_all(pool)
    .await?;
    #[cfg(feature = "mcp")]
    let ledger = get_ledger_entries(
        pool,
        &intents.iter().map(|i| i.id.clone()).collect::<Vec<_>>(),
    )
    .await?;
    Ok(intents
        .into_iter()
        .map(|i| {
            let row = RequestToolCallRow {
                tool_name: i.tool_name,
                sequence_number: i.sequence_number,
                mcp_execution_id: None,
                artifact_id: None,
                state: "intended".to_owned(),
                execution_status: None,
                is_structured: false,
                created_at: i.created_at,
            };
            #[cfg(feature = "mcp")]
            let row = match ledger.get(&i.id) {
                Some(entry) => RequestToolCallRow {
                    mcp_execution_id: entry.mcp_execution_id.clone(),
                    artifact_id: entry.artifact_id.clone(),
                    state: entry.state.clone(),
                    execution_status: entry.execution_status.clone(),
                    is_structured: entry.is_structured,
                    ..row
                },
                None => row,
            };
            row
        })
        .collect())
}

#[cfg(feature = "mcp")]
struct LedgerEntry {
    mcp_execution_id: Option<String>,
    artifact_id: Option<ArtifactId>,
    state: String,
    execution_status: Option<String>,
    is_structured: bool,
}

#[cfg(feature = "mcp")]
async fn get_ledger_entries(
    pool: &PgPool,
    intent_ids: &[String],
) -> Result<std::collections::HashMap<String, LedgerEntry>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT l.intent_id AS "intent_id!", l.mcp_execution_id,
                  l.artifact_id AS "artifact_id: ArtifactId",
                  l.state AS "state!", l.execution_status,
                  COALESCE(l.is_structured, FALSE) AS "is_structured!"
           FROM tool_call_ledger l
           WHERE l.intent_id = ANY($1)"#,
        intent_ids
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.intent_id,
                LedgerEntry {
                    mcp_execution_id: r.mcp_execution_id,
                    artifact_id: r.artifact_id,
                    state: r.state,
                    execution_status: r.execution_status,
                    is_structured: r.is_structured,
                },
            )
        })
        .collect())
}
