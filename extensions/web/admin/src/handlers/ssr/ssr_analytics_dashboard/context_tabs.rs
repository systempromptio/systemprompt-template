//! Typed view models for the five entity tabs of the analytics dashboard —
//! Models, Skills, Tools, Sessions and Cost.
//!
//! Split from `context.rs` at the 300-line ceiling. Every field here is
//! already formatted for display: the template branches on booleans and prints
//! strings, and derives no scale, percentage or currency of its own.

use serde::Serialize;
use systemprompt::identifiers::{SessionId, UserId};

use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::SvgStackedChartView;

use super::context::{AttributionLink, KpiTile};

#[derive(Debug, Default, Serialize)]
pub(super) struct ModelsTabView {
    pub kpis: Vec<KpiTile>,
    pub model_count: usize,
    pub redirect_count: usize,
    pub rows: Vec<ModelUsageRowView>,
    pub has_rows: bool,
    pub redirects: Vec<RedirectRowView>,
    pub has_redirects: bool,
    pub total_display: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelUsageRowView {
    pub model: String,
    // Why: the vendor namespace is a prefix on most model ids and repeats down
    // the whole column. It moves to the muted second line so the part that
    // distinguishes one row from the next gets the width, uncut.
    pub name_display: String,
    pub qualifier_display: String,
    pub requests: i64,
    pub share_pct: i64,
    pub input_display: String,
    pub output_display: String,
    pub cache_display: String,
    pub reasoning_display: String,
    pub cost_display: String,
    pub p50_display: String,
    pub p95_display: String,
    pub error_display: String,
    pub error_tone: &'static str,
    pub redirected_display: String,
    pub is_unrouted: bool,
    pub drill_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct RedirectRowView {
    pub requested_model: String,
    pub served_model: String,
    pub requests: i64,
    pub drill_url: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct SkillsTabView {
    pub kpis: Vec<KpiTile>,
    pub skill_count: i64,
    pub by_model_count: usize,
    pub rows: Vec<SkillRowView>,
    pub has_rows: bool,
    pub pagination: Option<Pagination>,
    pub by_model: Vec<SkillModelRowView>,
    pub has_by_model: bool,
    pub measurement_note: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillRowView {
    pub skill: String,
    // Why: no qualifier field. The full `plugin:skill` is already on the
    // title, so a separate plugin string would be the same text twice.
    pub name_display: String,
    pub invocations: i64,
    pub share_pct: i64,
    pub slash_display: String,
    pub tool_display: String,
    pub users: i64,
    pub sessions: i64,
    pub cost_display: String,
    pub unattributed_display: String,
    pub rating_display: String,
    pub drill_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillModelRowView {
    pub skill: String,
    pub model: String,
    pub requests: i64,
    pub cost_display: String,
    pub drill_url: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct ToolsTabView {
    pub kpis: Vec<KpiTile>,
    pub server_count: usize,
    pub tool_count: i64,
    pub servers: Vec<ToolServerRowView>,
    pub has_servers: bool,
    pub rows: Vec<ToolRowView>,
    pub has_rows: bool,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Serialize)]
pub(super) struct ToolServerRowView {
    pub server_name: String,
    pub executions: i64,
    pub share_pct: i64,
    pub success_display: String,
    pub tools: i64,
    pub users: i64,
    pub drill_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ToolRowView {
    pub tool_name: String,
    pub server_name: String,
    pub executions: i64,
    pub share_pct: i64,
    pub success_display: String,
    pub success_tone: &'static str,
    pub failed: i64,
    pub pending_display: String,
    pub p50_display: String,
    pub p95_display: String,
    pub users: i64,
    pub drill_url: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct SessionsTabView {
    pub kpis: Vec<KpiTile>,
    // Why: the section header counts the whole result, not the page. A header
    // reading 0 beside a tile reading 3 is the bug that produced this field.
    pub session_count: i64,
    pub rows: Vec<SessionCostRowView>,
    pub has_rows: bool,
    pub pagination: Option<Pagination>,
}

#[derive(Debug, Serialize)]
// Why: named for the snapshot it renders, not for "a session row". The
// sessions list page has its own row type and the two are different things —
// this one is one client-reported cost snapshot, that one is a session.
pub(super) struct SessionCostRowView {
    pub session_id: SessionId,
    pub session_short: String,
    pub user_id: UserId,
    pub model_display: String,
    pub cost_display: String,
    pub share_pct: i64,
    pub context_display: String,
    pub input_display: String,
    pub output_display: String,
    pub cache_display: String,
    pub rating_display: String,
    pub outcome_display: String,
    pub updated_display: String,
    pub detail_url: String,
    pub user_url: String,
}

#[derive(Debug, Default, Serialize)]
pub(super) struct CostTabView {
    pub kpis: Vec<KpiTile>,
    pub provider_count: usize,
    pub model_count: usize,
    pub container_count: usize,
    pub audience_links: Vec<AttributionLink>,
    pub is_internal: bool,
    pub csv_url: String,
    pub day_chart: Option<SvgStackedChartView>,
    pub providers: Vec<SupplierRowView>,
    pub has_providers: bool,
    pub models: Vec<SupplierRowView>,
    pub has_models: bool,
    pub axis_links: Vec<AttributionLink>,
    pub containers: Vec<ContainerRowView>,
    pub has_containers: bool,
    pub axis_label: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct SupplierRowView {
    pub label: String,
    pub requests: i64,
    pub share_pct: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub drill_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ContainerRowView {
    pub container_id: String,
    pub users: i64,
    pub sessions: i64,
    pub requests: i64,
    pub share_pct: i64,
    pub input_display: String,
    pub output_display: String,
    pub is_unattributed: bool,
    pub drill_url: String,
}
