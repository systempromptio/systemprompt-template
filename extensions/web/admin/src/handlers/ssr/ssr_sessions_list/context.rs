//! Typed template context for the sessions list (`sessions.hbs`).

use serde::Serialize;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

use crate::handlers::ssr::list_view::{
    AnnotatedOption, Chip, Pagination, Preserved, ScopeFilterView, TimeRangeContext,
};
use crate::handlers::ssr::types::{BreadcrumbView, SortHeaderView};

#[derive(Debug, Serialize)]
pub(super) struct SessionsListPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) current: CurrentSessionView,
    pub(super) time_range: TimeRangeContext,
    pub(super) filter_ribbon: FilterRibbon,
    pub(super) scope_filter: ScopeFilterView,
    pub(super) stats: StatsView,
    pub(super) sessions: Vec<SessionRowView>,
    pub(super) has_sessions: bool,
    pub(super) total_count: i64,
    pub(super) count_label: String,
    pub(super) pagination: Pagination,
    pub(super) sort_headers: SessionsSortHeaders,
    pub(super) error_only: bool,
    pub(super) error_toggle_url: String,
    pub(super) show_side: bool,
    pub(super) side_toggle_url: String,
}

// Why: The "you are here" strip: who this browser is signed in as, and a way
// into that session's own detail page.
#[derive(Debug, Serialize)]
pub(super) struct CurrentSessionView {
    pub(super) username: String,
    pub(super) session_id: Option<SessionId>,
    pub(super) session_id_short: Option<String>,
    pub(super) session_url: Option<String>,
}

#[derive(Debug, Serialize)]
pub(super) struct FilterRibbon {
    pub(super) base_url: &'static str,
    pub(super) preserved: Vec<Preserved>,
    pub(super) options: SessionFilterOptionsView,
    pub(super) chips: Vec<Chip>,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct SessionFilterOptionsView {
    pub(super) users: Vec<AnnotatedOption>,
}

#[derive(Debug, Serialize)]
pub(super) struct StatsView {
    pub(super) conversations: i64,
    pub(super) error_conversations: i64,
    pub(super) turns: i64,
    pub(super) side_calls: i64,
    pub(super) side_call_cost_display: String,
    pub(super) tokens_display: String,
    pub(super) cost_display: String,
    pub(super) conversations_delta: Option<String>,
    pub(super) conversations_delta_dir: Option<&'static str>,
    pub(super) turns_delta: Option<String>,
    pub(super) turns_delta_dir: Option<&'static str>,
    pub(super) tokens_delta: Option<String>,
    pub(super) tokens_delta_dir: Option<&'static str>,
    pub(super) cost_delta: Option<String>,
    pub(super) cost_delta_dir: Option<&'static str>,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionsSortHeaders {
    pub(super) started: SortHeaderView,
    pub(super) turns: SortHeaderView,
    pub(super) tokens: SortHeaderView,
    pub(super) cost: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(super) struct SessionRowView {
    pub(super) context_id: ContextId,
    pub(super) detail_url: String,
    // Why: not `title` — the layout's `title=` hash parameter shadows a field
    // of that name at every depth of the template.
    pub(super) conversation_title: String,
    pub(super) session_id: Option<SessionId>,
    pub(super) user_id: Option<UserId>,
    pub(super) user_label: String,
    pub(super) user_url: Option<String>,
    // Why: "unattributed" when no default row covers the person — a bucket
    // the page shows rather than a blank it hides.
    pub(super) group_label: String,
    pub(super) project_label: String,
    pub(super) model: Option<String>,
    pub(super) turn_count: i64,
    pub(super) side_call_count: i64,
    pub(super) tool_call_count: i64,
    pub(super) tokens_display: String,
    pub(super) tokens_title: String,
    pub(super) cost_display: String,
    pub(super) duration_display: String,
    pub(super) started_at: Option<String>,
    pub(super) started_relative: Option<String>,
    pub(super) error_count: i64,
    pub(super) has_error: bool,
    pub(super) status_label: String,
}
