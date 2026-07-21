//! Contexts-list repository — drives `/admin/entities/contexts`.
//!
//! Aggregates every `ai_requests` row by `context_id` and `FULL OUTER JOIN`s
//! against `user_contexts` so we surface contexts that exist only in one side
//! (a `user_contexts` row with no traffic, or traffic that bypassed
//! `core contexts create`). Mirrors the JOIN shape used by
//! `context_detail::find_context_header`.

use chrono::{DateTime, Utc};
use systemprompt::identifiers::{ContextId, SessionId, UserId};

mod kpis;
mod list;
mod users;

pub use kpis::{ContextListKpis, get_context_list_kpis, list_distinct_models};
pub use list::list_context_list;
pub use users::list_context_user_summary;

#[derive(Debug, Clone, Default)]
pub struct ContextListFilter {
    pub user_id: Option<UserId>,
    pub model: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub limit: i64,
}

#[derive(Debug, Clone)]
pub struct ContextListItem {
    pub context_id: ContextId,
    pub name: Option<String>,
    /// `user_contexts.kind` — `"user"` or `"cli_session"` (ephemeral CLI rows).
    pub kind: Option<String>,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub model: Option<String>,
    pub request_count: i64,
    pub message_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub first_request_at: Option<DateTime<Utc>>,
    pub last_request_at: Option<DateTime<Utc>>,
    pub last_activity_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ContextUserSummary {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub context_count: i64,
    pub request_count: i64,
    pub message_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub error_count: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
    pub distinct_models: Vec<String>,
}

pub(super) const fn resolved_limit(requested: i64) -> i64 {
    if requested > 0 && requested <= 500 {
        requested
    } else {
        100
    }
}

pub(super) fn free_text_pattern(filter: &ContextListFilter) -> Option<String> {
    filter
        .free_text
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
}
