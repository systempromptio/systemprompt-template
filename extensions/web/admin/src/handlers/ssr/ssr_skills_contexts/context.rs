//! Typed template-context structs for the contexts list page
//! (`skills-contexts.hbs`).
//!
//! `ContextItemView` is shared by the flat `contexts` list and the nested
//! `contexts` field inside each `UserSummaryView` — the nested table renders
//! a subset of the same fields, so the unused ones are simply left `None`/`0`.

use serde::Serialize;

#[derive(Debug, Serialize)]
pub(super) struct ContextsPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) contexts: Vec<ContextItemView>,
    pub(super) user_summaries: Vec<UserSummaryView>,
    pub(super) users_for_filter: Vec<UserForFilterView>,
    pub(super) models: Vec<String>,
    pub(super) kpis: PageKpisView,
    pub(super) filter: FilterView,
    pub(super) view_is_users: bool,
    pub(super) view_is_contexts: bool,
    pub(super) page_stats: Vec<PageStat>,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct ContextItemView {
    pub(super) context_id: String,
    pub(super) name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) user_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) display_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) session_id: Option<String>,
    pub(super) model: Option<String>,
    pub(super) request_count: i64,
    pub(super) message_count: i64,
    pub(super) error_count: i64,
    pub(super) input_tokens: i64,
    pub(super) output_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) cost_usd: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) first_request_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(super) last_request_at: Option<String>,
    pub(super) last_activity: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct UserSummaryView {
    pub(super) user_id: String,
    pub(super) display_name: Option<String>,
    pub(super) context_count: i64,
    pub(super) request_count: i64,
    pub(super) message_count: i64,
    pub(super) input_tokens: i64,
    pub(super) output_tokens: i64,
    pub(super) total_tokens: i64,
    pub(super) cost_usd: f64,
    pub(super) error_count: i64,
    pub(super) last_activity: Option<String>,
    pub(super) models: Vec<String>,
    pub(super) contexts: Vec<ContextItemView>,
}

#[derive(Debug, Serialize)]
pub(super) struct UserForFilterView {
    pub(super) user_id: String,
    pub(super) display_name: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct PageKpisView {
    pub(super) total_contexts: i64,
    pub(super) active_users: i64,
    pub(super) total_requests: i64,
    pub(super) total_messages: i64,
    pub(super) total_tokens: i64,
    pub(super) total_cost_usd: f64,
}

// Every field is always emitted (empty string when unset): the template reads
// `{{filter.q}}` etc. directly under Handlebars strict mode, which errors on an
// absent key — so these must be present, matching the pre-refactor `json!`.
#[derive(Debug, Serialize)]
pub(super) struct FilterView {
    pub(super) user_id: String,
    pub(super) model: String,
    pub(super) q: String,
    pub(super) since: String,
    pub(super) view: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PageStat {
    pub(super) value: i64,
    pub(super) label: &'static str,
}
