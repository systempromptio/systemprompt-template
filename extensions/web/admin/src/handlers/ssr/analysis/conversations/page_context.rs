//! The Handlebars context of the conversations page: tiles, charts, the
//! filter ribbon, the breakdown, the rows, selection and export.

use serde::Serialize;

use super::kpis::KpisView;
use super::summary::{ConversationAnalysisFilterView, ConversationBucketView};
use super::view::ConversationRowView;
use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::analysis::reports::banner::ReportBannerView;
use crate::handlers::ssr::analysis::ribbon::RibbonView;
use crate::handlers::ssr::list_view::{Pagination, ScopeFilterView};
use crate::handlers::ssr::types::{SortHeaderView, SvgLineChartView, TabLinkView};

#[derive(Debug, Serialize)]
pub(crate) struct SortHeaders {
    pub activity: SortHeaderView,
    pub turns: SortHeaderView,
    pub tools: SortHeaderView,
    pub errors: SortHeaderView,
    pub tokens: SortHeaderView,
    pub cost: SortHeaderView,
    pub active: SortHeaderView,
    pub duration: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub kpis: KpisView,
    pub charts: Vec<SvgLineChartView>,
    pub range_links: Vec<TabLinkView>,
    pub filter: ConversationAnalysisFilterView,
    pub ribbon: RibbonView,
    pub scope_filter: ScopeFilterView,
    pub breakdown_tabs: Vec<TabLinkView>,
    pub breakdown_label: &'static str,
    pub breakdown: Vec<ConversationBucketView>,
    pub rows: Vec<ConversationRowView>,
    pub has_rows: bool,
    pub count_label: String,
    pub turns_toggle: TurnsToggleView,
    pub pagination: Pagination,
    pub sort_headers: SortHeaders,
    pub export: crate::export::ExportView,
    pub help: HelpView,
    pub report_banner: ReportBannerView,
    // Why: the page's own URL, so a row's Judge button and the bulk bar
    // return here after the POST.
    pub current_url: String,
    pub judge_all_url: &'static str,
    pub can_judge: bool,
    // Why: the profile's `judge.automatic` is off — verdicts arrive
    // only when a person asks, so the page shows the buttons that ask.
    pub manual_judge: bool,
    pub unjudged_in_view: i64,
}

// Why: conversations with no turn are hidden by default; the toggle says how
// many and links to the other view.
#[derive(Debug, Serialize)]
pub(crate) struct TurnsToggleView {
    pub show_all: bool,
    pub without_turns: i64,
    pub href: String,
}
