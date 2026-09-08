//! Typed template-context structs for the conversation reader
//! (`context-detail.hbs`).

use serde::Serialize;

pub(super) use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use systemprompt::identifiers::{AiRequestId, ContextId, SessionId, TraceId, UserId};

pub(super) use crate::handlers::ssr::conversation_header::{
    ConversationStatsView, StatusBadgeView,
};
pub(super) use crate::handlers::ssr::transcript_view::ConversationView;

#[derive(Debug, Serialize)]
pub(super) struct ContextDetailPageContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) header: HeaderView,
    pub(super) stats: ConversationStatsView,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) status_badge: Option<StatusBadgeView>,
    pub(super) tabs: Vec<TabLinkView>,
    pub(super) show_conversation: bool,
    pub(super) show_requests: bool,
    pub(super) show_touched: bool,
    pub(super) conversation: ConversationView,
    pub(super) requests: Vec<ContextRequestRowView>,
    pub(super) has_requests: bool,
    pub(super) request_count: usize,
    pub(super) back_url: String,
    pub(super) back_label: String,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    // Why: what the session this context belongs to actually touched — the
    // files, tools and repositories the hooks pipeline recorded. Empty for a
    // context with no session, which is the gateway-only case.
    pub(super) entity_links: Vec<EntityLinkView>,
    pub(super) has_entity_links: bool,
    pub(super) entity_link_count: usize,
}

#[derive(Debug, Serialize)]
pub(super) struct EntityLinkView {
    pub(super) entity_type: String,
    pub(super) entity_name: String,
    pub(super) usage_count: i32,
    // Why: the width of the bar in the usage column, as a percentage of the
    // most-used entity on this context — a table of bare counts hides which
    // one dominated.
    pub(super) share_pct: f64,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) client_session_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) hooks_session_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) timeline: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct ContextRequestRowView {
    pub(super) id: AiRequestId,
    pub(super) id_short: String,
    pub(super) request_url: String,
    pub(super) trace_id: Option<TraceId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_id_short: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) trace_url: Option<String>,
    pub(super) kind: String,
    pub(super) kind_tone: &'static str,
    pub(super) message_count: i64,
    pub(super) model: String,
    pub(super) status: String,
    pub(super) is_error: bool,
    pub(super) latency_display: String,
    pub(super) cost_display: String,
    pub(super) created_at_local: String,
}
