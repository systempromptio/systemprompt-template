//! The overview's page context: the window control and the assembly of the
//! four widgets.
//!
//! Every derived value is computed in this module and its two siblings, so
//! `overview.hbs` does no arithmetic and no formatting: a number the template
//! printed differently from the page it links to is the drift this split
//! exists to prevent.

use serde::Serialize;

use crate::handlers::ssr::types::{BreadcrumbView, SvgLineChartView};
use crate::util::time_range::{TimeRange, TimeRangePreset};

use super::data::OverviewData;
use super::kpis_view::{OverviewKpiStripView, kpi_strip, volume_chart};
use super::panels_view::{
    BoardSpec, ModelsBoardView, QueuesView, ScopeBoardView, models_board, queues, scope_board,
};

// Why: three windows, not the audit pages' six. The overview answers a question
// about now, and a fifteen-minute window is noise rather than signal at the
// scale it reads. It carries the same `?preset=` name those pages use, so a
// window chosen here survives the hop to the detail.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum OverviewRange {
    Day,
    Week,
    Month,
}

impl OverviewRange {
    // Why: anything unrecognised lands on 24 hours rather than 400ing — a
    // mistyped preset in a shared link should still show the page.
    pub(crate) fn from_query(raw: Option<&str>) -> Self {
        match raw {
            Some("7d") => Self::Week,
            Some("30d") => Self::Month,
            _ => Self::Day,
        }
    }

    pub(super) const fn preset(self) -> &'static str {
        match self {
            Self::Day => "24h",
            Self::Week => "7d",
            Self::Month => "30d",
        }
    }

    pub(super) const fn hours(self) -> i64 {
        match self {
            Self::Day => 24,
            Self::Week => 24 * 7,
            Self::Month => 24 * 30,
        }
    }

    const fn window_label(self) -> &'static str {
        match self {
            Self::Day => "the last 24 hours",
            Self::Week => "the last 7 days",
            Self::Month => "the last 30 days",
        }
    }

    // Why: the chart's x axis names the window's edges in the unit the reader
    // picked, so "7d ago … 3.5d ago … now" rather than a timestamp per tick.
    pub(super) const fn span_label(self) -> &'static str {
        match self {
            Self::Day => "24h",
            Self::Week => "7d",
            Self::Month => "30d",
        }
    }

    pub(super) const fn half_span_label(self) -> &'static str {
        match self {
            Self::Day => "12h",
            Self::Week => "3.5d",
            Self::Month => "15d",
        }
    }

    pub(super) const fn previous_label(self) -> &'static str {
        match self {
            Self::Day => "vs previous 24h",
            Self::Week => "vs previous 7d",
            Self::Month => "vs previous 30d",
        }
    }

    const fn timerange_preset(self) -> TimeRangePreset {
        match self {
            Self::Day => TimeRangePreset::Hours24,
            Self::Week => TimeRangePreset::Days7,
            Self::Month => TimeRangePreset::Days30,
        }
    }

    pub(crate) fn time_range(self) -> TimeRange {
        let to = chrono::Utc::now();
        TimeRange {
            from: to - chrono::Duration::hours(self.hours()),
            to,
            preset: self.timerange_preset(),
            rejected_bounds: false,
        }
    }
}

#[derive(Debug, Serialize)]
pub(super) struct OverviewContext {
    pub page: &'static str,
    pub title: &'static str,
    pub subtitle: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub ranges: Vec<RangeLinkView>,
    pub kpis: OverviewKpiStripView,
    pub volume_chart: SvgLineChartView,
    pub models: ModelsBoardView,
    pub queues: QueuesView,
    pub projects: ScopeBoardView,
    pub groups: ScopeBoardView,
}

#[derive(Debug, Serialize)]
pub(super) struct RangeLinkView {
    pub label: &'static str,
    pub href: String,
    pub is_active: bool,
}

pub(super) fn overview_page(range: OverviewRange, data: &OverviewData) -> OverviewContext {
    OverviewContext {
        page: "overview",
        title: "Overview",
        subtitle: format!(
            "Spend, failures, latency, models and what is waiting on a person, across {}.",
            range.window_label()
        ),
        breadcrumbs: vec![BreadcrumbView::current("Overview")],
        ranges: range_links(range),
        kpis: kpi_strip(range, data),
        volume_chart: volume_chart(range, data),
        models: models_board(range, data),
        queues: queues(data),
        projects: scope_board(
            BoardSpec {
                title: "Top projects by spend",
                caption: "Exclusive attribution — each person counts once.",
                all_href: "/admin/projects",
                unattributed: "Unattributed (no primary project)",
            },
            &data.projects,
        ),
        groups: scope_board(
            BoardSpec {
                title: "Top groups by spend",
                caption: "Exclusive attribution — each person counts once.",
                all_href: "/admin/groups",
                unattributed: "Unattributed (no primary group)",
            },
            &data.groups,
        ),
    }
}

fn range_links(active: OverviewRange) -> Vec<RangeLinkView> {
    [
        (OverviewRange::Day, "24h"),
        (OverviewRange::Week, "7d"),
        (OverviewRange::Month, "30d"),
    ]
    .into_iter()
    .map(|(range, label)| RangeLinkView {
        label,
        href: format!("/admin?preset={}", range.preset()),
        is_active: range == active,
    })
    .collect()
}
