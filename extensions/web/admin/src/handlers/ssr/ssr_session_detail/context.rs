//! Typed template-context structs for the session-detail page
//! (`session-detail.hbs`).

use serde::Serialize;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

#[derive(Debug, Serialize)]
pub(super) struct SessionDetailPageContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) header: SessionHeaderView,
    pub(super) kpis: SessionKpisView,
    pub(super) contexts: Vec<SessionContextRowView>,
    pub(super) traces: Vec<SessionTraceRowView>,
    pub(super) requests: Vec<SessionRequestRowView>,
    pub(super) has_contexts: bool,
    pub(super) has_traces: bool,
    pub(super) has_requests: bool,
    pub(super) back_url: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionHeaderView {
    pub(super) session_id: SessionId,
    pub(super) session_id_short: String,
    pub(super) user_id: Option<UserId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) user_url: Option<String>,
    pub(super) display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) department: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_activity_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_activity_at_local: Option<String>,
    pub(super) duration_display: String,
    pub(super) status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) plugin_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) ai_title: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionKpisView {
    pub(super) request_count: i64,
    pub(super) context_count: i64,
    pub(super) trace_count: i64,
    pub(super) error_count: i64,
    pub(super) total_input_tokens: i64,
    pub(super) total_output_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) total_cost_microdollars: i64,
    pub(super) total_cost_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionContextRowView {
    pub(super) context_id: ContextId,
    pub(super) context_id_short: String,
    pub(super) context_url: String,
    pub(super) name: String,
    pub(super) request_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at_local: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) model: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionTraceRowView {
    pub(super) trace_id: String,
    pub(super) trace_id_short: String,
    pub(super) trace_url: String,
    pub(super) request_count: i64,
    pub(super) error_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) started_at_local: Option<String>,
    pub(super) duration_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionRequestRowView {
    pub(super) id: String,
    pub(super) id_short: String,
    pub(super) request_url: String,
    pub(super) context_id: Option<ContextId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_url: Option<String>,
    pub(super) trace_id: Option<String>,
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
