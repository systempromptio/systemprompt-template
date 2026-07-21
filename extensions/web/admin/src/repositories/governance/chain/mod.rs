//! Decision chain assembly.
//!
//! `governance_decisions` and `plugin_usage_events` do not carry a `trace_id`
//! column today, so the chain is anchored on `session_id` (shared by all four
//! tables) and surfaces the `trace_id` from `ai_requests` when available.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentId, AiRequestId, PluginId, SessionId, TraceId, UserId};

mod assemble;
mod fetch;
mod resolve;

pub use assemble::find_decision_chain;

#[derive(Debug, Clone, Serialize)]
pub struct ChainIdentity {
    pub user_id: UserId,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DecisionStage {
    pub id: String,
    pub policy: String,
    pub decision: String,
    pub reason: String,
    pub tool_name: String,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub evaluated_rules: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AiRequestSummary {
    pub id: String,
    pub request_id: AiRequestId,
    pub trace_id: Option<TraceId>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainUsageEvent {
    pub id: String,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub description: Option<String>,
    pub prompt_preview: Option<String>,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TranscriptEnvelope {
    pub id: String,
    pub model: Option<String>,
    pub entries_counted: Option<i32>,
    pub total_input_tokens: Option<i64>,
    pub total_output_tokens: Option<i64>,
    pub captured_at: DateTime<Utc>,
    pub transcript: serde_json::Value,
}

#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    pub ai_title: Option<String>,
    pub ai_summary: Option<String>,
    pub ai_tags: Option<String>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub ended_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct ChainTotals {
    pub decision_count: i64,
    pub deny_count: i64,
    pub event_count: i64,
    pub request_count: i64,
    pub total_cost_microdollars: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChainEnvelope {
    pub trace_id: Option<TraceId>,
    pub session_id: SessionId,
    pub identity: ChainIdentity,
    pub decisions: Vec<DecisionStage>,
    pub requests: Vec<AiRequestSummary>,
    pub events: Vec<ChainUsageEvent>,
    pub transcript: Option<TranscriptEnvelope>,
    pub summary: Option<SessionSummary>,
    pub totals: ChainTotals,
}
