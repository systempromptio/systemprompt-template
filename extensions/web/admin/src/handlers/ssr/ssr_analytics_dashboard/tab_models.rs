//! View assembly for the Models tab.
//!
//! One row per model the gateway actually served, plus the `unrouted` bucket
//! for requests rejected before a route was chosen. The redirect column
//! counts requests whose `requested_model` differed from what served them,
//! and the table below names the pairs — a count alone cannot say what the
//! route did.

use crate::handlers::ssr::format::format_cost;
use crate::handlers::ssr::types::bar_pct;
use crate::repositories::analytics::site::models::{ModelRedirectRow, ModelStatsRow};

use super::context::{KpiTile, ModelUsageRowView, ModelsTabView, RedirectRowView};
use super::view::compact;
use super::{AnalyticsDashboardQuery, urls};

pub(super) fn models_tab(
    rows: &[ModelStatsRow],
    redirects: &[ModelRedirectRow],
    query: &AnalyticsDashboardQuery,
) -> ModelsTabView {
    let max = rows.iter().map(|r| r.requests).max().unwrap_or(0);
    let total: i64 = rows.iter().map(|r| r.requests).sum();
    let views: Vec<ModelUsageRowView> = rows.iter().map(|r| model_row(r, max, query)).collect();
    ModelsTabView {
        kpis: kpis(rows, total),
        model_count: rows.len(),
        redirect_count: redirects.len(),
        has_rows: !views.is_empty(),
        rows: views,
        has_redirects: !redirects.is_empty(),
        redirects: redirects
            .iter()
            .map(|r| RedirectRowView {
                requested_model: r.requested_model.clone(),
                served_model: r.served_model.clone(),
                requests: r.requests,
                drill_url: urls::drill_url(query, "model", &r.served_model),
            })
            .collect(),
        total_display: format!("{total} requests across {} models", rows.len()),
    }
}

fn model_row(r: &ModelStatsRow, max: i64, query: &AnalyticsDashboardQuery) -> ModelUsageRowView {
    let error_pct = pct(r.errors, r.requests);
    ModelUsageRowView {
        model: r.model.clone(),
        name_display: short_name(&r.model, '/'),
        // Why: a rejected request never reached a provider, so the cell says
        // so rather than borrowing the provider of the route it never took.
        qualifier_display: qualifier(&r.model, '/', r.provider.as_deref()),
        requests: r.requests,
        share_pct: share(r.requests, max),
        input_display: compact(r.input_tokens),
        output_display: compact(r.output_tokens),
        cache_display: compact(r.cache_tokens),
        reasoning_display: compact(r.reasoning_tokens),
        cost_display: format_cost(r.cost_microdollars),
        p50_display: ms(r.p50_latency_ms),
        p95_display: ms(r.p95_latency_ms),
        error_display: format!("{error_pct:.1}%"),
        error_tone: tone(error_pct),
        redirected_display: if r.redirected == 0 {
            "—".to_owned()
        } else {
            r.redirected.to_string()
        },
        is_unrouted: r.is_unrouted,
        drill_url: urls::drill_url(query, "model", &r.model),
    }
}

fn kpis(rows: &[ModelStatsRow], total: i64) -> Vec<KpiTile> {
    let unrouted: i64 = rows
        .iter()
        .filter(|r| r.is_unrouted)
        .map(|r| r.requests)
        .sum();
    let redirected: i64 = rows.iter().map(|r| r.redirected).sum();
    let errors: i64 = rows.iter().map(|r| r.errors).sum();
    let cost: i64 = rows.iter().map(|r| r.cost_microdollars).sum();
    let served = rows.iter().filter(|r| !r.is_unrouted).count();
    vec![
        KpiTile {
            label: "Models served".to_owned(),
            value: served.to_string(),
            sub: format!("{total} requests in window"),
            tone: "accent",
        },
        KpiTile {
            label: "Spend".to_owned(),
            value: format_cost(cost),
            sub: format!("{} per request", format_cost(per(cost, total))),
            tone: "ok",
        },
        KpiTile {
            label: "Route redirects".to_owned(),
            value: redirected.to_string(),
            sub: "requested model differed from the one served".to_owned(),
            tone: if redirected > 0 { "warn" } else { "ok" },
        },
        KpiTile {
            label: "Unrouted".to_owned(),
            value: unrouted.to_string(),
            sub: "rejected before a route was chosen".to_owned(),
            tone: if unrouted > 0 { "err" } else { "ok" },
        },
        KpiTile {
            label: "Error rate".to_owned(),
            value: format!("{:.1}%", pct(errors, total)),
            sub: format!("{errors} failed or rejected"),
            tone: tone(pct(errors, total)),
        },
    ]
}

// Why: `bar_pct` floors a small share to a bar too thin to see, so a model
// with real traffic renders identically to one with none. Anything non-zero
// gets at least a visible sliver; only a true zero draws nothing.
const MIN_VISIBLE_BAR_PCT: i64 = 4;

pub(super) fn share(value: i64, max: i64) -> i64 {
    if value <= 0 {
        return 0;
    }
    bar_pct(value, max).max(MIN_VISIBLE_BAR_PCT)
}

// Why: model and skill ids carry a namespace prefix — `deepseek-ai/` on a
// model, `example-plugin:` on a skill — that repeats down the
// whole column and pushes the distinguishing half out of view. The name cell
// shows what differs; the prefix goes to the muted line under it.
pub(crate) fn short_name(id: &str, sep: char) -> String {
    id.rsplit_once(sep)
        .map_or_else(|| id.to_owned(), |(_, tail)| tail.to_owned())
}

pub(crate) fn qualifier(id: &str, sep: char, extra: Option<&str>) -> String {
    let prefix = id.rsplit_once(sep).map(|(head, _)| head.to_owned());
    match (prefix, extra) {
        (Some(p), Some(e)) => format!("{p} · {e}"),
        (Some(p), None) => p,
        (None, Some(e)) => e.to_owned(),
        (None, None) => "—".to_owned(),
    }
}

pub(super) fn pct(part: i64, whole: i64) -> f64 {
    if whole > 0 {
        part as f64 / whole as f64 * 100.0
    } else {
        0.0
    }
}

pub(super) const fn per(total: i64, count: i64) -> i64 {
    if count > 0 { total / count } else { 0 }
}

// Why: the same three bands everywhere a rate is shown, so a red cell means
// the same thing on the Models tab as it does on Tools.
pub(super) fn tone(error_pct: f64) -> &'static str {
    if error_pct >= 10.0 {
        "err"
    } else if error_pct >= 2.0 {
        "warn"
    } else {
        "ok"
    }
}

pub(crate) fn ms(value: Option<f64>) -> String {
    value.map_or_else(
        || "—".to_owned(),
        |v| {
            if v >= 1000.0 {
                format!("{:.1}s", v / 1000.0)
            } else {
                format!("{v:.0}ms")
            }
        },
    )
}
