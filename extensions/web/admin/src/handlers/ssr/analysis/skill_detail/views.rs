//! The Handlebars shapes of the skill detail page.

use serde::Serialize;

use crate::handlers::ssr::analysis::conversations::view::ConversationRowView;
use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::analysis::skills::SkillFactView;
use crate::handlers::ssr::list_view::Pagination;
use crate::handlers::ssr::types::{BreadcrumbView, SvgLineChartView, TabLinkView};

#[derive(Debug, Serialize)]
pub(super) struct SkillKpiView {
    pub(super) label: &'static str,
    pub(super) icon: &'static str,
    pub(super) value: String,
    pub(super) note: String,
    pub(super) tone: &'static str,
    pub(super) hint: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct BucketView {
    pub(super) label: String,
    pub(super) user_href: Option<String>,
    pub(super) invocations: i64,
    pub(super) share_pct: i64,
    pub(super) users: i64,
    pub(super) conversations: i64,
    pub(super) tokens_display: String,
    pub(super) cost_display: String,
    pub(super) errors: i64,
    pub(super) errors_tone: &'static str,
    pub(super) latency_display: String,
    pub(super) latency_tone: &'static str,
    pub(super) judged: i64,
    pub(super) completion_display: String,
    pub(super) completion_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillDetailContext {
    pub(super) page: &'static str,
    pub(super) title: String,
    pub(super) breadcrumbs: Vec<BreadcrumbView>,
    pub(super) skill_key: String,
    pub(super) skill_name: String,
    pub(super) plugin: String,
    pub(super) plugin_name: String,
    pub(super) marketplaces_display: String,
    pub(super) catalog_href: String,
    pub(super) skills_href: &'static str,
    pub(super) days: i64,
    pub(super) range_links: Vec<TabLinkView>,
    pub(super) used: bool,
    pub(super) row: Option<SkillFactView>,
    pub(super) kpis: Vec<SkillKpiView>,
    pub(super) charts: Vec<SvgLineChartView>,
    pub(super) breakdown_tabs: Vec<TabLinkView>,
    pub(super) breakdown_label: &'static str,
    pub(super) breakdown: Vec<BucketView>,
    pub(super) releases: Vec<super::runs::ReleaseRowView>,
    pub(super) runs: Vec<super::runs::RunRowView>,
    pub(super) run_count: usize,
    pub(super) conversations: Vec<ConversationRowView>,
    pub(super) conversation_count: i64,
    pub(super) pagination: Pagination,
    pub(super) export: crate::export::ExportView,
    pub(super) help: HelpView,
}
