//! Tool-call read models for the MCP Tools page and per-tool drill page.
//!
//! Sources:
//! - `plugin_usage_events` (tool invocations — `event_type ILIKE '%ToolUse%'`)
//! - `governance_decisions` (verdict for the same `session_id` + `tool_name`)
//! - `ai_requests` (parent gateway request — for `trace_id` surfacing)
//!
//! [`fetch_tool_calls_paged`] (in `paged`) returns one row per tool invocation
//! with its governance verdict; the per-tool aggregates (`stats`) back the
//! drill-down page.

use chrono::{DateTime, Utc};
use serde::Serialize;
use systemprompt::identifiers::{AgentId, PluginId, SessionId, TraceId, UserId};

mod paged;
mod stats;

pub use paged::{ToolCallPage, fetch_tool_calls_paged};
pub use stats::{
    ToolActorGroup, ToolDenyReason, ToolDetailStats, ToolTopActor, fetch_tool_deny_reasons,
    fetch_tool_detail_stats, fetch_tool_top_actors,
};

#[derive(Debug, Clone, Default)]
pub struct ToolCallFilter {
    pub tool_name: Option<String>,
    pub user_id: Option<UserId>,
    pub agent_scope: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub decision: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub enum ToolSortColumn {
    CreatedAt,
    Bytes,
    Latency,
}

impl ToolSortColumn {
    const fn sql_key(self) -> &'static str {
        match self {
            Self::CreatedAt => "created_at",
            Self::Bytes => "bytes",
            Self::Latency => "latency",
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
pub struct ToolSortSpec {
    pub column: ToolSortColumn,
    pub dir: SortDir,
}

impl Default for ToolSortSpec {
    fn default() -> Self {
        Self {
            column: ToolSortColumn::CreatedAt,
            dir: SortDir::Desc,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolCallRow {
    pub id: String,
    pub created_at: DateTime<Utc>,
    pub event_type: String,
    pub tool_name: Option<String>,
    pub plugin_id: Option<PluginId>,
    pub user_id: UserId,
    pub session_id: SessionId,
    pub agent_id: Option<AgentId>,
    pub agent_scope: Option<String>,
    pub content_input_bytes: i64,
    pub content_output_bytes: i64,
    pub decision: Option<String>,
    pub policy: Option<String>,
    pub reason: Option<String>,
    pub trace_id: Option<TraceId>,
    pub ar_latency_ms: Option<i32>,
    pub metadata: serde_json::Value,
}
