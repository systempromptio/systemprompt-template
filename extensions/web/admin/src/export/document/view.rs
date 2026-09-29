//! The document rows of a conversation bundle and how each repository row
//! becomes one. Identifiers leave as plain strings and payloads leave as the
//! JSON they were stored as, so the file needs no knowledge of the console's
//! newtypes to be read.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{ArtifactId, MarketplaceId, PluginId};

use crate::repositories::analysis::conversations::hook_events::ConversationHookEventRow;
use crate::repositories::analysis::conversations::planes::{
    ConversationDecisionRow, ConversationSafetyRow, ConversationSkillRow, ConversationToolCallRow,
    ConversationTurnRow,
};
use crate::repositories::analytics::context_detail::{ContextMessageRow, ContextRequestRow};
use crate::repositories::analytics::context_tool_calls::ContextToolCallRow;

#[derive(Debug, Serialize)]
pub(crate) struct RequestDoc {
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub kind: String,
    pub trace_id: Option<String>,
    pub gateway_conversation_id: Option<String>,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub requested_model: Option<String>,
    pub route_match: Option<String>,
    pub status: String,
    pub finish_reason: Option<String>,
    pub error_message: Option<String>,
    pub is_streaming: Option<bool>,
    pub message_count: i64,
    pub max_tokens: Option<i32>,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cache_read_tokens: Option<i32>,
    pub cache_creation_tokens: Option<i32>,
    pub reasoning_tokens: Option<i64>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub upstream_latency_ms: Option<i32>,
    pub tool_calls: Option<i64>,
    pub tool_names: Vec<String>,
    pub safety_findings: Option<i64>,
    pub safety_blocked: Option<i64>,
}

pub(crate) fn request_doc(r: &ContextRequestRow, turn: Option<&ConversationTurnRow>) -> RequestDoc {
    RequestDoc {
        request_id: r.id.as_str().to_owned(),
        created_at: r.created_at,
        kind: r.effective_kind.clone(),
        trace_id: r.trace_id.as_ref().map(|t| t.as_str().to_owned()),
        gateway_conversation_id: r
            .gateway_conversation_id
            .as_ref()
            .map(|g| g.as_str().to_owned()),
        provider: turn.and_then(|t| t.provider.clone()),
        model: r.model.clone(),
        requested_model: turn.and_then(|t| t.requested_model.clone()),
        route_match: turn.and_then(|t| t.route_match.clone()),
        status: r.status.clone(),
        finish_reason: turn.and_then(|t| t.finish_reason.clone()),
        error_message: turn.and_then(|t| t.error_message.clone()),
        is_streaming: turn.map(|t| t.is_streaming),
        message_count: r.message_count,
        max_tokens: r.max_tokens,
        input_tokens: r.input_tokens,
        output_tokens: r.output_tokens,
        cache_read_tokens: r.cache_read_tokens,
        cache_creation_tokens: r.cache_creation_tokens,
        reasoning_tokens: turn.map(|t| t.reasoning_tokens),
        cost_microdollars: r.cost_microdollars,
        latency_ms: r.latency_ms,
        upstream_latency_ms: turn.and_then(|t| t.upstream_latency_ms),
        tool_calls: turn.map(|t| t.tool_calls),
        tool_names: turn.map(|t| t.tool_names.clone()).unwrap_or_default(),
        safety_findings: turn.map(|t| t.safety_findings),
        safety_blocked: turn.map(|t| t.safety_blocked),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct MessageDoc {
    pub request_id: String,
    pub sequence_number: i32,
    pub role: String,
    pub content: String,
    pub tool_call_id: Option<String>,
    pub created_at: DateTime<Utc>,
}

pub(crate) fn message_doc(m: &ContextMessageRow, body: impl Fn(&str) -> String) -> MessageDoc {
    MessageDoc {
        request_id: m.request_id.as_str().to_owned(),
        sequence_number: m.sequence_number,
        role: m.role.clone(),
        content: body(&m.content),
        tool_call_id: m.tool_call_id.as_ref().map(|id| id.as_str().to_owned()),
        created_at: m.created_at,
    }
}

// JSON: tool arguments and results are shaped by each tool's own schema and
// leave the console exactly as the gateway stored them.
#[derive(Debug, Serialize)]
pub(crate) struct ToolCallDoc {
    pub request_id: String,
    pub sequence_number: i32,
    pub tool_name: String,
    pub tool_use_id: Option<String>,
    pub tool_input: serde_json::Value,
    pub tool_result: Option<serde_json::Value>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_structured: bool,
    pub created_at: DateTime<Utc>,
}

pub(crate) fn tool_call_doc(t: &ContextToolCallRow) -> ToolCallDoc {
    ToolCallDoc {
        request_id: t.request_id.as_str().to_owned(),
        sequence_number: t.sequence_number,
        tool_name: t.tool_name.clone(),
        tool_use_id: t.ai_tool_call_id.as_ref().map(|id| id.as_str().to_owned()),
        tool_input: t.tool_input.clone(),
        tool_result: t.tool_result_payload.clone(),
        artifact_id: t.artifact_id.clone(),
        artifact_structured: t.artifact_structured,
        created_at: t.created_at,
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolLedgerDoc {
    pub tool_name: Option<String>,
    pub server_name: Option<String>,
    pub state: String,
    pub source: Option<String>,
    pub execution_status: Option<String>,
    pub execution_time_ms: Option<i64>,
    pub occurred_at: Option<DateTime<Utc>>,
    pub request_id: Option<String>,
    pub artifact_id: Option<ArtifactId>,
    pub artifact_type: Option<String>,
    pub artifact_title: Option<String>,
    pub artifact_kind: Option<String>,
    pub input_summary: Option<String>,
    pub is_builtin: bool,
    pub is_structured: bool,
    pub payload_bytes: Option<i32>,
    pub is_error: Option<bool>,
    pub error_message: Option<String>,
}

pub(crate) fn tool_ledger_doc(t: &ConversationToolCallRow) -> ToolLedgerDoc {
    ToolLedgerDoc {
        tool_name: t.tool_name.clone(),
        server_name: t.server_name.clone(),
        state: t.state.clone(),
        source: t.source.clone(),
        execution_status: t.execution_status.clone(),
        execution_time_ms: t.execution_time_ms,
        occurred_at: t.occurred_at,
        request_id: t.request_id.clone(),
        artifact_id: t.artifact_id.clone(),
        artifact_type: t.artifact_type.clone(),
        artifact_title: t.artifact_title.clone(),
        artifact_kind: t.artifact_kind.clone(),
        input_summary: t.input_summary.clone(),
        is_builtin: t.is_builtin,
        is_structured: t.is_structured,
        payload_bytes: t.payload_bytes,
        is_error: t.is_error,
        error_message: t.error_message.clone(),
    }
}

// JSON: the chain's per-rule evaluations, shaped by each policy stage.
#[derive(Debug, Serialize)]
pub(crate) struct DecisionDoc {
    pub created_at: DateTime<Utc>,
    pub tool_name: String,
    pub decision: String,
    pub policy: String,
    pub reason: String,
    pub plugin_id: Option<PluginId>,
    pub tool_use_id: Option<String>,
    pub trace_id: Option<String>,
    pub evaluated_rules: Option<serde_json::Value>,
}

pub(crate) fn decision_doc(d: &ConversationDecisionRow) -> DecisionDoc {
    DecisionDoc {
        created_at: d.created_at,
        tool_name: d.tool_name.clone(),
        decision: d.decision.clone(),
        policy: d.policy.clone(),
        reason: d.reason.clone(),
        plugin_id: d.plugin_id.clone(),
        tool_use_id: d.tool_use_id.clone(),
        trace_id: d.trace_id.clone(),
        evaluated_rules: d.evaluated_rules.clone(),
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SkillDoc {
    pub skill: String,
    pub plugin_id: Option<PluginId>,
    pub marketplace_id: Option<MarketplaceId>,
    pub marketplace_hash: Option<String>,
    pub invocations: i64,
    pub first_invoked_at: DateTime<Utc>,
}

pub(crate) fn skill_doc(s: &ConversationSkillRow) -> SkillDoc {
    SkillDoc {
        skill: s.skill.clone(),
        plugin_id: s.plugin_id.clone(),
        marketplace_id: s.marketplace_id.clone(),
        marketplace_hash: s.marketplace_hash.clone(),
        invocations: s.invocations,
        first_invoked_at: s.first_invoked_at,
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct SafetyDoc {
    pub created_at: DateTime<Utc>,
    pub request_id: String,
    pub phase: String,
    pub category: String,
    pub severity: String,
    pub scanner: String,
    pub blocked: bool,
    pub excerpt: Option<String>,
}

pub(crate) fn safety_doc(s: &ConversationSafetyRow) -> SafetyDoc {
    SafetyDoc {
        created_at: s.created_at,
        request_id: s.request_id.clone(),
        phase: s.phase.clone(),
        category: s.category.clone(),
        severity: s.severity.clone(),
        scanner: s.scanner.clone(),
        blocked: s.blocked,
        excerpt: s.excerpt.clone(),
    }
}

// JSON: the hook payload as the harness posted it, shaped per event type.
#[derive(Debug, Serialize)]
pub(crate) struct HookEventDoc {
    pub created_at: DateTime<Utc>,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub prompt_preview: Option<String>,
    pub description: Option<String>,
    pub tool_use_id: Option<String>,
    pub trace_id: Option<String>,
    pub metadata: serde_json::Value,
}

pub(crate) fn hook_event_doc(e: &ConversationHookEventRow) -> HookEventDoc {
    HookEventDoc {
        created_at: e.created_at,
        event_type: e.event_type.clone(),
        tool_name: e.tool_name.clone(),
        plugin_id: e.plugin_id.clone(),
        prompt_preview: e.prompt_preview.clone(),
        description: e.description.clone(),
        tool_use_id: e.tool_use_id.clone(),
        trace_id: e.trace_id.clone(),
        metadata: e.metadata.clone(),
    }
}
