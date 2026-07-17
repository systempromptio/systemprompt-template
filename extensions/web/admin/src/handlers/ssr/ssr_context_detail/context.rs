//! Typed template-context structs for the context-detail page
//! (`context-detail.hbs`).

use serde::Serialize;
use systemprompt::identifiers::{ContextId, SessionId, TraceId, UserId};

#[derive(Debug, Serialize)]
pub(super) struct ContextDetailPageContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) header: HeaderView,
    pub(super) kpis: KpisView,
    pub(super) transcript: Vec<TranscriptEntryView>,
    pub(super) has_transcript: bool,
    pub(super) requests: Vec<RequestRowView>,
    pub(super) has_requests: bool,
    pub(super) back_url: String,
    pub(super) back_label: String,
}

#[derive(Debug, Serialize)]
pub(super) struct HeaderView {
    pub(super) context_id: ContextId,
    pub(super) context_id_short: String,
    pub(super) user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) user_url: Option<String>,
    pub(super) display_name: Option<String>,
    pub(super) session_id: Option<SessionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) session_url: Option<String>,
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) created_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) updated_at_local: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct KpisView {
    pub(super) request_count: i64,
    pub(super) trace_count: i64,
    pub(super) error_count: i64,
    pub(super) total_input_tokens: i64,
    pub(super) total_output_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) total_cost_microdollars: i64,
    pub(super) total_cost_display: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_request_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at_local: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct TranscriptEntryView {
    // Why: display DTO; request id carried as string from the transcript grouping key
    pub(super) request_id: String,
    pub(super) request_url: String,
    pub(super) ts_local: String,
    pub(super) ts_full: String,
    pub(super) kind: &'static str,
    pub(super) role: String,
    pub(super) is_user: bool,
    pub(super) is_assistant: bool,
    pub(super) is_system: bool,
    pub(super) is_tool: bool,
    pub(super) content_preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tool_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tool_input_pretty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) tool_result_pretty: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct RequestRowView {
    pub(super) id: String,
    pub(super) id_short: String,
    pub(super) request_url: String,
    pub(super) trace_id: Option<TraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_url: Option<String>,
    pub(super) model: String,
    pub(super) status: String,
    pub(super) is_error: bool,
    pub(super) latency_display: String,
    pub(super) cost_display: String,
    pub(super) created_at_local: String,
}
