//! Gateway-request read models for the analytics requests page.
//!
//! [`fetch_requests_paged`] (in `paged`) pages `ai_requests` with optional
//! filters and per-row governance / tool-call counts;
//! [`list_recent_gateway_requests`] (in `recent`) is the lightweight
//! recent-activity feed; the dropdown option lists live in `options`.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentId, SessionId, UserId};

mod options;
mod paged;
mod recent;

pub use options::{RequestFilterOptions, fetch_request_filter_options};
pub use paged::{RequestPage, fetch_requests_paged};
pub use recent::{RecentGatewayRequestRow, list_recent_gateway_requests};

#[derive(Debug, Clone, Default)]
pub struct RequestFilter {
    pub user_id: Option<UserId>,
    pub agent_id: Option<AgentId>,
    pub model: Option<String>,
    pub provider: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum RequestSortColumn {
    CreatedAt,
    Cost,
    Latency,
    Tokens,
}

impl RequestSortColumn {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::Cost => "cost",
            Self::Latency => "latency",
            Self::Tokens => "tokens",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct RequestSortSpec {
    pub column: RequestSortColumn,
    pub dir: SortDir,
}

impl Default for RequestSortSpec {
    fn default() -> Self {
        Self {
            column: RequestSortColumn::CreatedAt,
            dir: SortDir::Desc,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct RequestRow {
    pub id: String,
    pub request_id: String,
    pub created_at: DateTime<Utc>,
    pub user_id: UserId,
    pub user_label: Option<String>,
    pub session_id: Option<SessionId>,
    pub trace_id: Option<String>,
    pub provider: String,
    pub model: String,
    pub status: String,
    pub input_tokens: Option<i32>,
    pub output_tokens: Option<i32>,
    pub cost_microdollars: i64,
    pub latency_ms: Option<i32>,
    pub error_message: Option<String>,
    pub decision_count: i64,
    pub deny_count: i64,
    pub tool_call_count: i64,
}
