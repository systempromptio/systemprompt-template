//! Assembling the page context both lenses hand their template: tiles,
//! charts, ribbon, breakdown rows, the rows table, sort headers, export.

use serde::Serialize;

use super::figures::{ToolTileView, charts, tiles};
use super::query::{Lens, ToolsQuery};
use super::ribbon::ribbon;
use super::rows::{ToolActivityRowView, format_bytes, kind_icon, kind_label, tool_row};
use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::analysis::ribbon::RibbonView;
use crate::handlers::ssr::analysis::tone::{deny_tone, error_rate_tone, latency_tone, percent};
use crate::handlers::ssr::format::format_duration_ms;
use crate::handlers::ssr::list_view::{DEFAULT_PAGE_SIZE, PageWindow, Pagination, ScopeFilterView};
use crate::handlers::ssr::types::{BreadcrumbView, SortHeaderView, SvgLineChartView, TabLinkView};
use crate::repositories::analysis::tools::{
    ToolActivityResult, ToolBreakdownBy, ToolBucketRow, ToolSort,
};
use crate::util::time_range::TimeRange;

#[derive(Debug, Serialize)]
pub(crate) struct ToolBucketView {
    pub label: String,
    pub icon: &'static str,
    pub href: Option<String>,
    pub export_href: Option<String>,
    pub calls: i64,
    pub executed: i64,
    pub failed: i64,
    pub failed_tone: &'static str,
    pub denied: i64,
    pub denied_tone: &'static str,
    pub users: i64,
    pub artifacts: i64,
    pub artifact_pct: String,
    pub share_pct: i64,
    pub duration_display: String,
    pub duration_tone: &'static str,
    pub bytes_display: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolSortHeaders {
    pub time: SortHeaderView,
    pub tool: SortHeaderView,
    pub duration: SortHeaderView,
    pub size: SortHeaderView,
}

#[derive(Debug, Serialize)]
pub(crate) struct ToolsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub range_links: Vec<TabLinkView>,
    pub range_rejected: bool,
    // Why: a link from a conversation or skill row pins the window to that
    // row's span; the page says so and offers the way out.
    pub window_pinned: bool,
    pub window_display: String,
    pub widen_href: String,
    pub scope_filter: ScopeFilterView,
    pub kpis: Vec<ToolTileView>,
    pub charts: Vec<SvgLineChartView>,
    pub ribbon: RibbonView,
    pub breakdown_tabs: Vec<TabLinkView>,
    pub breakdown_label: &'static str,
    pub breakdown: Vec<ToolBucketView>,
    pub rows: Vec<ToolActivityRowView>,
    pub has_rows: bool,
    pub count_label: String,
    pub pagination: Pagination,
    pub sort_headers: ToolSortHeaders,
    pub export: crate::export::ExportView,
    pub help: HelpView,
    pub current_url: String,
}

fn bucket_rows(lens: Lens, q: &ToolsQuery, data: &ToolActivityResult) -> Vec<ToolBucketView> {
    let by = q.breakdown(lens);
    let total = data.totals.calls.max(1);
    data.breakdown
        .iter()
        .map(|b: &ToolBucketRow| {
            let (href, export_href) = b.value.as_deref().map_or((None, None), |v| {
                (
                    Some(q.narrowed(lens, by.param(), v)),
                    Some(q.export_href(lens, by.param(), v)),
                )
            });
            let icon = match by {
                ToolBreakdownBy::Kind => kind_icon(b.value.as_deref()),
                ToolBreakdownBy::Server => "plug",
                ToolBreakdownBy::User => "user",
                ToolBreakdownBy::Client => "model",
                ToolBreakdownBy::Skill => "skill",
                ToolBreakdownBy::Tool => "wrench",
            };
            ToolBucketView {
                label: if by == ToolBreakdownBy::Kind {
                    kind_label(b.value.as_deref()).to_owned()
                } else {
                    b.label.clone()
                },
                icon,
                href,
                export_href,
                calls: b.calls,
                executed: b.executed,
                failed: b.failed,
                failed_tone: error_rate_tone(b.failed, b.calls),
                denied: b.denied,
                denied_tone: deny_tone(b.denied),
                users: b.users,
                artifacts: b.artifacts,
                artifact_pct: percent(b.artifacts, b.calls),
                share_pct: b.calls * 100 / total,
                duration_display: b.p95_duration_ms.map_or_else(
                    || "\u{2014}".to_owned(),
                    |ms| format_duration_ms(ms.round() as i64),
                ),
                duration_tone: latency_tone(b.p95_duration_ms),
                bytes_display: format_bytes(b.bytes),
            }
        })
        .collect()
}

fn sort_headers(lens: Lens, q: &ToolsQuery) -> ToolSortHeaders {
    ToolSortHeaders {
        time: q.sort_header(
            lens,
            ToolSort::Time,
            ("When", "sp-col-date", "When the call happened"),
        ),
        tool: q.sort_header(
            lens,
            ToolSort::Tool,
            ("Tool", "sp-tools-col-tool", "Tool name"),
        ),
        duration: q.sort_header(
            lens,
            ToolSort::Duration,
            ("Duration", "sp-table__cell--num", "Execution time"),
        ),
        size: q.sort_header(
            lens,
            ToolSort::Size,
            (
                "Size",
                "sp-table__cell--num",
                "Payload bytes of the artifact",
            ),
        ),
    }
}

pub(crate) struct Frame<'a> {
    pub range: TimeRange,
    pub scope_filter: ScopeFilterView,
    pub help: HelpView,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub noun: &'static str,
    pub datasets: &'a [&'a str],
}

pub(crate) fn build_context(
    lens: Lens,
    q: &ToolsQuery,
    data: &ToolActivityResult,
    frame: Frame<'_>,
) -> ToolsPageContext {
    let rows: Vec<ToolActivityRowView> = data.rows.iter().map(tool_row).collect();
    let total = if lens == Lens::Artifacts {
        data.totals.artifacts
    } else {
        data.totals.calls
    };
    let shown = i64::try_from(rows.len()).unwrap_or(0);
    let window = PageWindow::new(q.page(), DEFAULT_PAGE_SIZE, total, shown, frame.noun);
    let hourly = (frame.range.to - frame.range.from).num_hours() <= 48;
    let current_url = format!("{}?{}", lens.base_url(), q.query_string(&[]));
    ToolsPageContext {
        page: match lens {
            Lens::Tools => "tools",
            Lens::Artifacts => "artifacts",
        },
        title: match lens {
            Lens::Tools => "Tools",
            Lens::Artifacts => "Artifacts",
        },
        breadcrumbs: frame.breadcrumbs,
        range_links: q.range_links(lens, frame.range),
        range_rejected: frame.range.rejected_bounds,
        window_pinned: q.from.is_some() && q.to.is_some(),
        window_display: format!(
            "{} → {}",
            frame.range.from.format("%b %-d, %H:%M"),
            frame.range.to.format("%b %-d, %H:%M")
        ),
        widen_href: format!(
            "{}preset=30d",
            q.link_prefix(lens, &["preset", "from", "to", "page"])
        ),
        scope_filter: frame.scope_filter,
        kpis: tiles(lens, &data.totals, &data.series),
        charts: charts(lens, &data.series, hourly),
        ribbon: ribbon(lens, q, data),
        breakdown_tabs: q.breakdown_tabs(lens),
        breakdown_label: q.breakdown(lens).label(),
        breakdown: bucket_rows(lens, q, data),
        has_rows: !rows.is_empty(),
        count_label: format!("{total} {}", frame.noun),
        rows,
        pagination: q.pagination(lens, window),
        sort_headers: sort_headers(lens, q),
        export: crate::export::ExportView::new(frame.datasets, &q.query_string(&["page", "ids"])),
        help: frame.help,
        current_url,
    }
}
