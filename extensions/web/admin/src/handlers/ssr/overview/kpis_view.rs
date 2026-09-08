//! The KPI strip: seven figures, each with its window-over-window delta and a
//! link to the page that owns the rows it counted, and the request-volume
//! chart drawn under it.
//!
//! The chart is its own row rather than a band behind the Requests tile: a
//! two-rem sparkline at half opacity was a shape nobody could read, and the
//! strip is scanned for numbers, not for lines.

use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, format_duration_ms, short_num};
use crate::handlers::ssr::types::{LineChartSpec, SvgLineChartView, SvgSeriesInput, line_chart};
use crate::repositories::overview::kpis::OverviewKpis;
use crate::repositories::overview::series::BUCKETS;
use crate::util::delta::{Delta, delta};

use super::data::OverviewData;
use super::view::OverviewRange;

#[derive(Debug, Serialize)]
pub(super) struct OverviewKpiStripView {
    pub error: Option<String>,
    pub comparison_note: Option<String>,
    pub tiles: Vec<KpiTileView>,
}

#[derive(Debug, Serialize)]
pub(super) struct KpiTileView {
    pub label: &'static str,
    pub value: String,
    pub sub: String,
    pub hint: String,
    pub href: String,
    pub tone: &'static str,
    pub delta: String,
    pub delta_dir: &'static str,
}

// Why: a percentage against a nearly empty prior window is arithmetic, not
// information — 53 requests yesterday and 1,400 today reads as "+2537.7%",
// which tells a reader nothing they can act on and hides the one fact that
// matters, which is that there is barely a prior window to compare with. Below
// the floor the tiles say what the prior window held instead.
const COMPARABLE_PRIOR_REQUESTS: i64 = 100;

pub(super) fn kpi_strip(range: OverviewRange, data: &OverviewData) -> OverviewKpiStripView {
    let k = data.kpis.as_ref().copied().unwrap_or_default();
    // Why: one gate for all seven tiles rather than a floor per figure. Every
    // one of them is derived from the same prior window's traffic, so either
    // that window is worth comparing against or none of them are.
    let comparable = k.prev_requests >= COMPARABLE_PRIOR_REQUESTS;
    OverviewKpiStripView {
        error: data
            .kpis
            .as_ref()
            .err()
            .map(|e| format!("The headline figures could not be read: {e}")),
        comparison_note: (!comparable).then(|| {
            format!(
                "The previous window carried {} requests, too few to compare against, so no tile shows a percentage.",
                k.prev_requests
            )
        }),
        tiles: traffic_tiles(&k, range, comparable)
            .into_iter()
            .chain(health_tiles(&k, range, comparable))
            .collect(),
    }
}

// Why: the request spine drawn as a real chart with axes, in the same
// component every analytics page uses, so the shape of the day reads the same
// here as it does one click deeper.
pub(super) fn volume_chart(range: OverviewRange, data: &OverviewData) -> SvgLineChartView {
    let buckets = data.buckets.as_deref().unwrap_or(&[]);
    let total: i64 = buckets.iter().sum();
    line_chart(LineChartSpec {
        title: "Request volume",
        subtitle: format!("{} requests, {}", short_num(total), bucket_caption(range)),
        empty_message: "No gateway traffic in this window.",
        series: vec![SvgSeriesInput {
            label: "Requests".to_owned(),
            values: buckets.to_vec(),
            value_display: short_num(total),
        }],
        ref_lines: Vec::new(),
        y_max: None,
        y_display: short_num,
        x_start_display: format!("{} ago", range.span_label()),
        x_mid_display: format!("{} ago", range.half_span_label()),
        x_end_display: "now".to_owned(),
        show_area: true,
    })
}

// Why: the first three tiles say how much happened; the four after them say
// how well it went. Two builders so each stays short enough to read whole.
fn traffic_tiles(k: &OverviewKpis, range: OverviewRange, comparable: bool) -> Vec<KpiTileView> {
    let preset = range.preset();
    let vs = range.previous_label();
    vec![
        tile(
            Tile {
                label: "Requests",
                value: short_num(k.requests),
                sub: comparison(&short_num(k.prev_requests), vs),
                hint: "gateway requests in the window".to_owned(),
                href: format!("/admin/requests?preset={preset}"),
                tone: "",
            },
            comparable.then(|| delta(k.requests, k.prev_requests, true)),
        ),
        tile(
            Tile {
                label: "Spend",
                value: format_cost(k.cost_microdollars),
                sub: comparison(&format_cost(k.prev_cost_microdollars), vs),
                hint: format!("{} across the window", format_cost(k.cost_microdollars)),
                href: format!("/admin/analytics?tab=cost&preset={preset}"),
                tone: "",
            },
            comparable.then(|| delta(k.cost_microdollars, k.prev_cost_microdollars, false)),
        ),
        tile(
            Tile {
                label: "Active people",
                value: k.active_users.to_string(),
                sub: comparison(&k.prev_active_users.to_string(), vs),
                hint: format!("{} distinct accounts made a request", k.active_users),
                href: "/admin/users".to_owned(),
                tone: "",
            },
            comparable.then(|| delta(k.active_users, k.prev_active_users, true)),
        ),
    ]
}

fn health_tiles(k: &OverviewKpis, range: OverviewRange, comparable: bool) -> Vec<KpiTileView> {
    let preset = range.preset();
    let vs = range.previous_label();
    vec![
        tile(
            Tile {
                label: "p50 latency",
                value: format_duration_ms(k.p50_latency_ms),
                sub: comparison(&format_duration_ms(k.prev_p50_latency_ms), vs),
                hint: "half of routed requests finished faster than this".to_owned(),
                href: format!("/admin/traces?preset={preset}"),
                tone: "",
            },
            comparable.then(|| delta(k.p50_latency_ms, k.prev_p50_latency_ms, false)),
        ),
        tile(
            Tile {
                label: "p95 latency",
                value: format_duration_ms(k.p95_latency_ms),
                sub: comparison(&format_duration_ms(k.prev_p95_latency_ms), vs),
                hint: "95% of routed requests finished faster than this".to_owned(),
                href: format!("/admin/traces?preset={preset}"),
                tone: "",
            },
            comparable.then(|| delta(k.p95_latency_ms, k.prev_p95_latency_ms, false)),
        ),
        tile(
            Tile {
                label: "Error rate",
                value: percent_tenths(k.error_rate_tenths(), k.requests),
                sub: comparison(
                    &percent_tenths(k.prev_error_rate_tenths(), k.prev_requests),
                    vs,
                ),
                hint: format!("{} failed of {}", k.errors, short_num(k.requests)),
                href: format!("/admin/requests?status=failed&preset={preset}"),
                tone: if k.errors > 0 { "err" } else { "" },
            },
            comparable.then(|| delta(k.error_rate_tenths(), k.prev_error_rate_tenths(), false)),
        ),
        tile(
            Tile {
                label: "Denied",
                value: k.denied.to_string(),
                sub: comparison(&k.prev_denied.to_string(), vs),
                hint: "refused by policy or quota before routing".to_owned(),
                href: format!("/admin/requests?status=rejected&preset={preset}"),
                tone: if k.denied > 0 { "warn" } else { "" },
            },
            comparable.then(|| delta(k.denied, k.prev_denied, false)),
        ),
    ]
}

// Why: the tile's own fields travel together as one value so the builder takes
// two arguments rather than six — the delta is the only thing computed
// separately, because its polarity is the caller's call and not the tile's.
struct Tile {
    label: &'static str,
    value: String,
    sub: String,
    // Why: the figure a tile's own sub line has no room for. It reaches the
    // reader as the tile's title, so the sub line can stay one band and still
    // say what the previous window held.
    hint: String,
    href: String,
    tone: &'static str,
}

// Why: an absent delta is not a zero delta. `None` means the tile has nothing
// honest to compare against, and it renders no delta element at all rather
// than an em dash a reader could take for a measured no-change.
fn tile(spec: Tile, d: Option<Delta>) -> KpiTileView {
    KpiTileView {
        label: spec.label,
        value: spec.value,
        sub: spec.sub,
        hint: spec.hint,
        href: spec.href,
        tone: spec.tone,
        delta: d.map(|d| d.display()).unwrap_or_default(),
        delta_dir: d.map_or("", delta_dir),
    }
}

// Why: the prior window's own figure belongs on every tile; the reason the
// percentage is missing belongs above the strip, said once. Repeating it seven
// times wrapped every tile onto a second line and cost the whole row its
// height budget, to tell the reader the same thing seven times.
fn comparison(prior: &str, vs: &str) -> String {
    format!("{prior} {vs}")
}

// Why: the delta component takes one direction class, not a Delta — polarity
// and direction fold into that token here so the template branches on nothing.
// A flat delta gets no modifier and keeps the base arrow.
fn delta_dir(d: Delta) -> &'static str {
    match (d.direction, d.tone) {
        ("up", "bad") => "up-bad",
        ("down", "good") => "down-good",
        ("up", _) => "up",
        ("down", _) => "down",
        _ => "",
    }
}

// Why: an empty window has no rate to report; a zero would claim a
// measurement nobody took.
fn percent_tenths(tenths: i64, sample: i64) -> String {
    if sample <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{}.{}%", tenths / 10, tenths % 10)
}

// Why: the chart always draws the same number of points, so what one point
// covers changes with the window and the caption has to say which it is.
fn bucket_caption(range: OverviewRange) -> String {
    match range.hours() / BUCKETS {
        1 => "one point per hour".to_owned(),
        n => format!("one point per {n}h"),
    }
}
