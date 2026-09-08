//! Conversation rows — the one repository behind `/admin/contexts`,
//! `/admin/sessions`, the user detail Usage tab, and the profile pane.
//!
//! A conversation is a context; the SQL views `conversation_requests` and
//! `conversation_rollups` (schema `27_conversation_requests.sql`) classify each
//! gateway request as a turn or a side call and roll them up per context, so
//! every page reads the same row and every number above a table describes the
//! rows inside it. Filtering, paging, and sorting are bound parameters chosen
//! by `CASE` arms, never interpolated text.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt::identifiers::{ContextId, SessionId, UserId};

mod list;
mod page;
pub use page::{ConversationPageMode, ConversationPageResult, load_conversation_page};
mod recent;
mod totals;
mod users;

pub use list::list_conversations_paged;
pub use recent::{find_latest_conversation, list_recent_conversations};
pub use totals::{ConversationTotals, get_conversation_totals, list_distinct_models};
pub use users::UserConversationSummary;

/// One conversation as the lists and profile pages show it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationRow<Id = ContextId> {
    pub context_id: Id,
    pub title: String,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    pub session_id: Option<SessionId>,
    pub client_session_id: Option<String>,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub model: Option<String>,
    pub turn_count: i64,
    pub side_call_count: i64,
    pub side_call_cost_microdollars: i64,
    pub tool_call_count: i64,
    pub error_count: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub total_cost_microdollars: i64,
    pub first_at: Option<DateTime<Utc>>,
    pub last_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
}

/// Narrowing applied to the list, its totals, and the by-user rollup alike.
#[derive(Debug, Clone, Default)]
pub struct ConversationFilter {
    pub user_id: Option<UserId>,
    // Why: the caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
    pub model: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    // Why: `false` hides conversations made only of probes and utility calls.
    pub include_side_calls: bool,
    pub error_only: bool,
}

impl ConversationFilter {
    pub(super) fn free_text_pattern(&self) -> Option<String> {
        self.free_text
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
    }
}

/// The columns a conversation list can be ordered by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ConversationSort {
    #[default]
    Activity,
    Turns,
    Tokens,
    Cost,
}

impl ConversationSort {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Activity => "activity",
            Self::Turns => "turns",
            Self::Tokens => "tokens",
            Self::Cost => "cost",
        }
    }

    #[must_use]
    pub fn parse_conversation_sort(value: Option<&str>) -> Self {
        match value {
            Some("turns") => Self::Turns,
            Some("tokens") => Self::Tokens,
            Some("cost") => Self::Cost,
            _ => Self::Activity,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversationPage {
    pub sort: ConversationSort,
    pub descending: bool,
    pub limit: i64,
    pub offset: i64,
}

impl Default for ConversationPage {
    fn default() -> Self {
        Self {
            sort: ConversationSort::Activity,
            descending: true,
            limit: 50,
            offset: 0,
        }
    }
}
