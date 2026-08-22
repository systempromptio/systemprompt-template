//! Sessions-list repository — drives `/admin/entities/sessions`.
//!
//! A session id is written by two producers that never meet: the gateway
//! stamps it on every `ai_requests` row, and the hook pipeline rolls its
//! events up into `plugin_session_summaries`. Neither table is a superset of
//! the other, so both are `FULL OUTER JOIN`ed on `session_id` — the same shape
//! `session_detail::find_session_header` uses, which is what makes a row here
//! resolve to a detail page rather than a 404.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::{PluginId, SessionId, UserId};

mod kpis;
mod list;

pub use kpis::{SessionListKpis, get_session_list_kpis};
pub use list::list_sessions_paged;

/// Narrowing applied to both the list and the KPI strip, so the numbers above
/// the table always describe the rows inside it.
#[derive(Debug, Clone, Default)]
pub struct SessionListFilter {
    pub user_id: Option<UserId>,
    pub error_only: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct SessionPage {
    pub limit: i64,
    pub offset: i64,
}

#[derive(Debug, Clone)]
pub struct SessionListItem {
    pub session_id: SessionId,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub department: Option<String>,
    pub model: Option<String>,
    pub ai_title: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub client_source: Option<String>,
    pub permission_mode: Option<String>,
    pub status: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub request_count: i64,
    pub context_count: i64,
    pub trace_count: i64,
    pub tool_uses: i64,
    pub prompts: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub has_gateway: bool,
    pub has_hooks: bool,
}
