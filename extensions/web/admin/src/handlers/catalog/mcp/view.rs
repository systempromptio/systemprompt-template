//! View-model types for `/admin/mcp` and `/admin/mcp/{id}`.
//!
//! An MCP server has two halves that can disagree, and the pages exist to show
//! the disagreement: the declaration in `services/mcp/*.yaml`, and the runtime
//! record of sessions and tool calls. A server declared but never connected and
//! a server serving traffic under a name nothing declares are both real states,
//! so `configured` and `has_runtime` are separate fields rather than one.

use serde::Serialize;

use crate::handlers::catalog::sorting::SortHeaderView;
use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::BreadcrumbView;

use super::super::view::LinkedEntity;

// Why: One row of the server list: the declaration, the liveness, and the
// traffic.
#[derive(Debug, Clone, Serialize)]
pub(super) struct McpServerRow {
    pub id: String,
    pub description: String,
    pub detail_url: String,
    pub matrix_url: String,
    pub source_path: String,
    pub configured: bool,
    pub enabled: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub server_type: String,
    pub transport: String,
    pub auth_label: String,
    pub oauth_required: bool,
    pub sessions_open: i64,
    pub alive: bool,
    pub last_heartbeat_display: String,
    pub proxy_identities: i64,
    pub calls: i64,
    pub errors: i64,
    pub error_rate_display: String,
    pub error_tone: &'static str,
    pub p95_display: String,
    pub distinct_users: i64,
    pub delta_display: String,
    pub delta_dir: &'static str,
    pub last_call_display: String,
    pub plugin_count: usize,
    pub assignment_count: i64,
}

#[derive(Debug, Clone, Serialize)]
pub(super) struct McpKpiView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub tone: &'static str,
    pub unit: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct McpPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub window_label: &'static str,
    pub heartbeat_label: String,
    pub kpis: Vec<McpKpiView>,
    pub sort_headers: Vec<SortHeaderView>,
    pub servers: Vec<McpServerRow>,
    pub servers_count: usize,
    pub unconfigured_count: usize,
    pub access_control_url: &'static str,
    pub sort_key: String,
    pub sort_dir: String,
    pub search: String,
}

// Why: A tool the server has actually served, with how well it served it.
#[derive(Debug, Clone, Serialize)]
pub(super) struct McpToolRow {
    pub tool_name: String,
    pub calls: i64,
    pub failures: i64,
    pub error_rate_display: String,
    pub error_tone: &'static str,
    pub distinct_users: i64,
    pub avg_display: String,
    pub max_display: String,
    pub last_call_display: String,
}

// Why: One line of the call log.
#[derive(Debug, Clone, Serialize)]
pub(super) struct McpExecutionRowView {
    pub execution_id: String,
    pub short_id: String,
    pub tool_name: String,
    pub status: String,
    pub status_tone: &'static str,
    pub started_display: String,
    pub duration_display: String,
    pub caller: String,
    pub user_url: String,
    pub session: String,
    pub session_url: Option<String>,
    pub trace_url: Option<String>,
    pub error_message: String,
}

// Why: One client attached to the server.
#[derive(Debug, Clone, Serialize)]
pub(super) struct McpSessionRowView {
    pub session: String,
    pub short_id: String,
    pub caller: String,
    pub user_url: String,
    pub status: String,
    pub status_tone: &'static str,
    pub alive: bool,
    pub started_display: String,
    pub last_activity_display: String,
    pub expires_display: String,
    pub identity_label: String,
}

// Why: One access-control rule pointing at this server.
#[derive(Debug, Clone, Serialize)]
pub(super) struct McpGrantRow {
    pub subject_kind: String,
    pub subject: String,
    pub access: String,
    pub is_allow: bool,
    pub updated_display: String,
}

// Why: A `label: value` line of the configuration summary.
#[derive(Debug, Clone, Serialize)]
pub(super) struct ConfigFactView {
    pub label: &'static str,
    pub value: String,
    pub mono: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct McpDetailData {
    pub page: &'static str,
    pub title: String,
    pub subtitle: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub id: String,
    pub configured: bool,
    pub enabled: bool,
    pub status_label: &'static str,
    pub status_tone: &'static str,
    pub window_label: &'static str,
    pub kpis: Vec<McpKpiView>,
    pub tools: Vec<McpToolRow>,
    pub tools_count: usize,
    pub executions: Vec<McpExecutionRowView>,
    pub executions_count: i64,
    pub pagination: Pagination,
    pub sessions: Vec<McpSessionRowView>,
    pub sessions_count: usize,
    pub grants: Vec<McpGrantRow>,
    pub grants_count: usize,
    pub default_included: bool,
    pub config_facts: Vec<ConfigFactView>,
    pub oauth_scopes: Vec<String>,
    pub included_by: Vec<LinkedEntity>,
    pub included_by_count: usize,
    pub matrix_url: String,
    pub access_control_url: &'static str,
}
