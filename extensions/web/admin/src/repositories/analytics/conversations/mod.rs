//! Conversations & Transcripts page data layer.
//!
//! Parses the JSONB `session_transcripts.transcript` into a flat
//! `Vec<TranscriptTurn>`, enriches each turn with any matching
//! `governance_decisions` row, and exposes both a redacted (default) and an
//! optional raw text body.
//!
//! The `/admin/history` listing lives in `unified`, which reads both places a
//! conversation is recorded: these transcripts, and the gateway's own
//! `ai_requests` rows grouped by context. It replaced a transcript-only query
//! that showed nothing at all to a user who never runs Claude Code.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{PluginId, SessionId, TraceId, UserId};

mod detail;
mod gateway_text;
mod redact;
pub mod scope;
mod store;
mod transcript;
mod unified;

pub use detail::find_raw_turns;
pub use gateway_text::strip_gateway_markers;
pub use redact::redact_text;
pub use scope::{HistoryScope, has_full_history_view, history_scope_for, resolve_history_scope};
pub use store::upsert_session_transcript;
pub use unified::{HistoryItem, HistorySource, list_history_items};

#[derive(Debug, Clone, Serialize)]
pub struct ConversationListItem {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub plugin_id: Option<PluginId>,
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
    pub plugin_id: Option<PluginId>,
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
    pub content_redacted: Option<String>,
    pub redactions_applied: u32,
    pub content: Option<String>,
    pub tool_calls: Vec<ToolCall>,
    pub governance: Option<TurnGovernance>,
    pub anomaly_count: i32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCall {
    pub id: Option<String>,
    pub name: String,
    // JSON: arbitrary tool arguments lifted out of the third-party transcript.
    pub args_json: serde_json::Value,
    // JSON: arbitrary tool result lifted out of the third-party transcript.
    pub result_json: Option<serde_json::Value>,
    pub duration_ms: Option<i32>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TurnGovernance {
    pub decision: String,
    pub trace_id: Option<TraceId>,
    pub rule_count: i32,
    pub redactions_applied: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationDetail {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub plugin_id: Option<PluginId>,
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
