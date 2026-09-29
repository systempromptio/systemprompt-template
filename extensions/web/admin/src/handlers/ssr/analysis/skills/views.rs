//! The Handlebars shapes of the Skills page.

use serde::Serialize;

use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::analysis::reports::banner::ReportBannerView;
use crate::handlers::ssr::analysis::ribbon::RibbonView;
use crate::handlers::ssr::types::{SparklineView, SvgLineChartView, TabLinkView};

// Why: one skill row, every number pre-formatted and toned.
#[derive(Debug, Serialize)]
pub(crate) struct SkillFactView {
    pub skill: String,
    pub skill_name: String,
    pub href: String,
    pub plugin: String,
    pub marketplace: String,
    pub invocations: i64,
    pub invocations_note: String,
    pub users: i64,
    pub entitled: usize,
    pub reach_display: String,
    pub installs: i64,
    pub conversations: i64,
    pub turns: i64,
    pub tokens_display: String,
    pub tokens_title: String,
    pub cost_display: String,
    pub cost_per_invocation: String,
    pub errors: i64,
    pub errors_tone: &'static str,
    pub denied: i64,
    pub denied_tone: &'static str,
    pub tool_calls: i64,
    pub tool_calls_failed: i64,
    pub tools_href: String,
    pub artifacts: i64,
    pub artifacts_href: String,
    pub latency_display: String,
    pub latency_tone: &'static str,
    pub models: Vec<String>,
    pub clients: Vec<String>,
    pub judged: i64,
    pub completion_display: String,
    pub completion_tone: &'static str,
    // Why: "82 · 12/40" — the mean beside how many of the skill's
    // conversations it rests on.
    pub completion_note: String,
    pub attributed_pct: String,
    pub spark: SparklineView,
    pub first_used: String,
    pub last_used: String,
}

#[derive(Debug, Serialize)]
pub(super) struct PluginGroupView {
    pub(super) plugin: String,
    pub(super) rows: Vec<SkillFactView>,
    pub(super) invocations: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct MarketplaceGroupView {
    pub(super) marketplace: String,
    pub(super) marketplace_name: String,
    pub(super) versions_href: String,
    pub(super) plugins: Vec<PluginGroupView>,
    pub(super) skills: usize,
    pub(super) invocations: i64,
    pub(super) users: i64,
}

// Why: one Overview row — a marketplace's funnel and its window, every rate
// over its own denominator so none can pass 100.
#[derive(Debug, Serialize)]
pub(super) struct AdoptionView {
    pub(super) marketplace: String,
    pub(super) marketplace_name: String,
    pub(super) versions_href: String,
    pub(super) skills_href: String,
    pub(super) activity_href: String,
    pub(super) entitled: usize,
    pub(super) installed: i64,
    // Why: installed people the rules entitle; the rate's numerator. The
    // difference is shown beside the count, never folded into the rate.
    pub(super) installed_entitled: usize,
    pub(super) installed_outside: i64,
    pub(super) active: i64,
    pub(super) install_rate: String,
    pub(super) install_rate_pct: i64,
    pub(super) activation_rate: String,
    pub(super) active_pct: i64,
    pub(super) hosts_title: String,
    pub(super) skills: i64,
    pub(super) skills_used: i64,
    pub(super) plugins: i64,
    pub(super) invocations: i64,
    pub(super) conversations: i64,
    pub(super) cost_display: String,
    pub(super) completion_display: String,
    pub(super) completion_tone: &'static str,
    pub(super) version_short: String,
    pub(super) versions: i64,
    pub(super) last_install: String,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillsKpiView {
    pub(super) label: &'static str,
    pub(super) icon: &'static str,
    pub(super) value: String,
    pub(super) note: String,
    pub(super) tone: &'static str,
    pub(super) hint: &'static str,
    pub(super) spark: SparklineView,
}

// Why: the Activity tab's list under the chart — the skills the chart
// draws, three columns, each a link into the table.
#[derive(Debug, Serialize)]
pub(super) struct TopSkillView {
    pub(super) skill: String,
    pub(super) href: String,
    pub(super) marketplace_name: String,
    pub(super) invocations: i64,
    pub(super) users: i64,
    pub(super) share: String,
    pub(super) share_pct: i64,
    pub(super) last_used: String,
}

// Why: the marketplace scope strip; a name is a String, unlike a tab's.
#[derive(Debug, Serialize)]
pub(super) struct ScopeLinkView {
    pub(super) label: String,
    pub(super) href: String,
    pub(super) is_active: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct SkillsPageContext {
    pub(super) page: &'static str,
    pub(super) title: &'static str,
    pub(super) tab: &'static str,
    pub(super) tabs: Vec<TabLinkView>,
    pub(super) days: i64,
    pub(super) range_links: Vec<TabLinkView>,
    pub(super) marketplace_links: Vec<ScopeLinkView>,
    pub(super) marketplace_name: Option<String>,
    pub(super) scope_label: String,
    pub(super) ribbon: RibbonView,
    pub(super) kpis: Vec<SkillsKpiView>,
    pub(super) chart: Option<SvgLineChartView>,
    pub(super) top_skills: Vec<TopSkillView>,
    pub(super) adoption: Vec<AdoptionView>,
    pub(super) groups: Vec<MarketplaceGroupView>,
    pub(super) row_count: usize,
    pub(super) unused: Vec<UnusedSkillView>,
    pub(super) unused_count: usize,
    pub(super) export: crate::export::ExportView,
    pub(super) is_admin: bool,
    pub(super) help: HelpView,
    pub(super) report_banner: ReportBannerView,
    pub(super) current_url: String,
}

#[derive(Debug, Serialize)]
pub(super) struct UnusedSkillView {
    pub(super) skill: String,
    pub(super) plugin: String,
    pub(super) marketplace: String,
    pub(super) entitled: usize,
    pub(super) catalog_href: String,
}
