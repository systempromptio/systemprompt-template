//! Conversations & Transcripts page data layer.
//!
//! `fetch_conversation_list` powers the left pane (sessions filtered by
//! time-range / identity / free-text). `fetch_conversation_detail` parses the
//! JSONB `session_transcripts.transcript` into a flat `Vec<TranscriptTurn>`,
//! enriches each turn with any matching `governance_decisions` row, and
//! exposes both a redacted (default) and an optional raw text body.
//!
//! Free-text search relies on the `idx_session_transcripts_jsonb` GIN index
//! (`jsonb_path_ops`). Pure substring searches use `ILIKE` against the JSONB
//! cast to text — that is unindexed and capped at 200 rows by the SQL filter
//! upstream so the cost is bounded.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

mod detail;
mod list;
mod redact;
mod transcript;

pub use detail::{fetch_conversation_detail, fetch_raw_turns};
pub use list::fetch_conversation_list;
pub use redact::redact_text;

#[derive(Debug, Clone, Serialize)]
pub struct ConversationListItem {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub plugin_id: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub ai_title: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub governance_intervention_count: i64,
    pub deny_count: i64,
}

#[derive(Debug, Clone, Default)]
pub struct ConversationListFilter {
    pub user_id: Option<UserId>,
    pub plugin_id: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub limit: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptTurn {
    pub id: String,
    pub session_id: SessionId,
    pub ordinal: i32,
    pub role: String,
    pub ts: Option<DateTime<Utc>>,
    pub model: Option<String>,
    pub latency_ms: Option<i32>,
    /// Always populated. PII-bearing substrings are replaced with sentinels.
    pub content_redacted: Option<String>,
    pub redactions_applied: u32,
    /// Only populated when the caller holds `transcript:view_pii`.
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub governance: Option<TurnGovernance>,
    pub anomaly_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    pub args_json: serde_json::Value,
    pub result_json: Option<serde_json::Value>,
    pub duration_ms: Option<i32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnGovernance {
    pub decision: String,
    pub trace_id: Option<String>,
    pub rule_count: i32,
    pub redactions_applied: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationDetail {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub plugin_id: Option<String>,
    pub ai_title: Option<String>,
    pub ai_summary: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
    pub turns: Vec<TranscriptTurn>,
}

/// Just the raw turn bodies, keyed by ordinal — the capability-gated endpoint
/// returns this when the viewer holds `transcript:view_pii`.
#[derive(Debug, Clone, Serialize)]
pub struct RawTurnBody {
    pub ordinal: i32,
    pub content: String,
}
