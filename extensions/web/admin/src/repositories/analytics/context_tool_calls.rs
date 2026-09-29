//! Tool calls of a context's transcript requests, joined to the artifact the
//! ingest stored for each through the tool-call ledger.
//!
//! Split from `context_detail` so each file stays readable; the caller names
//! the request ids, exactly as it does for messages.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{AiRequestId, AiToolCallId, ArtifactId};

#[derive(Debug, Clone)]
pub struct ContextToolCallRow {
    pub request_id: AiRequestId,
    pub tool_name: String,
    pub sequence_number: i32,
    pub ai_tool_call_id: Option<AiToolCallId>,
    // JSON: arbitrary MCP tool arguments, shaped by each tool's own schema.
    pub tool_input: serde_json::Value,
    // JSON: arbitrary MCP tool result payload, shaped by each tool's own schema.
    pub tool_result_payload: Option<serde_json::Value>,
    // Why: the stored artifact of this call, joined by the client tool_use_id.
    pub artifact_id: Option<ArtifactId>,
    pub artifact_structured: bool,
    pub created_at: DateTime<Utc>,
}

pub async fn list_tool_calls_for_requests(
    pool: &PgPool,
    request_ids: &[String],
) -> Result<Vec<ContextToolCallRow>, sqlx::Error> {
    if request_ids.is_empty() {
        return Ok(Vec::new());
    }
    // JSON: per-tool payload columns — see ContextToolCallRow above.
    sqlx::query_as!(
        ContextToolCallRow,
        r#"
        SELECT
            t.request_id          AS "request_id!: AiRequestId",
            t.tool_name           AS "tool_name!",
            t.sequence_number     AS "sequence_number!",
            t.ai_tool_call_id     AS "ai_tool_call_id?: AiToolCallId",
            -- `tool_input` is a TEXT column holding a JSON document. Selecting
            -- it raw decoded to JSON null in every row, because sqlx handed
            -- the text bytes to `serde_json::Value`'s Postgres decoder, which
            -- expects the JSON wire format. The cast makes the column what the
            -- Rust type already claimed it was.
            t.tool_input::jsonb   AS "tool_input!: serde_json::Value",
            t.tool_result_payload AS "tool_result_payload?: serde_json::Value",
            l.artifact_id         AS "artifact_id: ArtifactId",
            COALESCE(l.is_structured, FALSE) AS "artifact_structured!",
            r.created_at          AS "created_at!"
        FROM ai_request_tool_calls t
        JOIN ai_requests r ON r.id = t.request_id
        LEFT JOIN tool_call_ledger l ON l.intent_id = t.id
        WHERE t.request_id = ANY($1)
        ORDER BY r.created_at ASC, t.sequence_number ASC
        "#,
        request_ids
    )
    .fetch_all(pool)
    .await
}
