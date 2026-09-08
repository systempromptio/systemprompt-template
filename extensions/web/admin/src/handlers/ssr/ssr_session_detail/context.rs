//! Typed template-context structs for the session-detail page
//! (`session-detail.hbs`).

use serde::Serialize;

pub(super) use crate::handlers::ssr::types::BreadcrumbView;
use systemprompt::identifiers::{AiRequestId, ContextId, PluginId, SessionId, TraceId, UserId};

#[derive(Debug, Serialize)]
pub(super) struct SessionDetailPageContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) header: SessionHeaderView,
    pub(super) kpis: SessionKpisView,
    pub(super) contexts: Vec<SessionContextRowView>,
    pub(super) traces: Vec<SessionTraceRowView>,
    pub(super) requests: Vec<SessionRequestRowView>,
    pub(super) has_contexts: bool,
    pub(super) has_traces: bool,
    pub(super) has_requests: bool,
    pub(super) back_url: &'static str,
    // Why: absent until the hooks pipeline has summarised the run, which is
    // the normal state for a session still in flight — the page says so rather
    // than rendering an empty verdict panel.
    pub(super) analysis: Option<AnalysisView>,
    pub(super) ratings: Vec<RatingView>,
    pub(super) has_ratings: bool,
    pub(super) rating_count: usize,
    pub(super) rating_average: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct AnalysisView {
    pub(super) title: String,
    pub(super) summary: String,
    pub(super) category: String,
    pub(super) outcome: String,
    pub(super) goal_achieved: String,
    pub(super) quality_score: i16,
    pub(super) quality_tone: &'static str,
    pub(super) goal_tone: &'static str,
    pub(super) tags: Vec<String>,
    pub(super) has_tags: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) recommendations: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) improvement_hints: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) error_analysis: Option<String>,
    pub(super) corrections_count: i32,
    pub(super) duration_display: String,
    pub(super) turns_display: String,
    pub(super) updated_at_local: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RatingView {
    pub(super) user_id: UserId,
    pub(super) user_label: String,
    pub(super) user_url: String,
    pub(super) rating: i16,
    pub(super) stars: String,
    pub(super) tone: &'static str,
    pub(super) outcome: String,
    pub(super) notes: String,
    pub(super) created_at_local: String,
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
    pub(super) groups_display: Option<String>,
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
    pub(super) plugin_id: Option<PluginId>,
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
    pub(super) total_tokens: i64,
    pub(super) token_display: String,
    pub(super) cost_display: String,
    pub(super) error_count: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionTraceRowView {
    pub(super) trace_id: TraceId,
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
    pub(super) id: AiRequestId,
    pub(super) id_short: String,
    pub(super) request_url: String,
    pub(super) context_id: Option<ContextId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) context_url: Option<String>,
    pub(super) trace_id: Option<TraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_url: Option<String>,
    pub(super) model: String,
    pub(super) status: String,
    pub(super) is_error: bool,
    pub(super) is_rejected: bool,
    pub(super) latency_display: String,
    pub(super) cost_display: String,
    pub(super) created_at_local: String,
}
