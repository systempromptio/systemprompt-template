//! The panels under the KPI strip: the model board, the human queues, and
//! the two spend leaderboards.
//!
//! Each panel carries its loader's error instead of its rows when the read
//! failed, so one broken aggregate costs the reader that panel and nothing
//! else.

use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::handlers::ssr::format::{format_cost, short_num};
use crate::handlers::ssr::ssr_analytics_dashboard::{ms, qualifier, short_name};
use crate::repositories::analytics::site::models::ModelStatsRow;
use crate::repositories::overview::queues::{UsageAnomalyRow, anomaly_tone};
use crate::repositories::overview::scopes::ScopeCostRow;

use super::data::{OverviewData, TOP_MODELS, TOP_SCOPES};
use super::view::OverviewRange;

#[derive(Debug, Serialize)]
pub(super) struct QueuesView {
    pub approvals_error: Option<String>,
    pub approvals: i64,
    pub approvals_tone: &'static str,
    pub anomalies_error: Option<String>,
    pub anomalies: Vec<OverviewAnomalyRowView>,
    pub anomaly_count: usize,
    pub anomalies_tone: &'static str,
    pub has_anomalies: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct OverviewAnomalyRowView {
    pub metric: &'static str,
    pub window_display: String,
    pub observed_display: String,
    pub baseline_display: String,
    pub severity_display: String,
    pub severity_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelsBoardView {
    pub error: Option<String>,
    pub rows: Vec<OverviewModelRowView>,
    pub has_rows: bool,
    pub all_href: String,
    pub summary: String,
}

#[derive(Debug, Serialize)]
pub(super) struct OverviewModelRowView {
    pub model: String,
    pub name_display: String,
    pub qualifier_display: String,
    pub href: String,
    pub requests_display: String,
    pub share_display: String,
    pub cost_display: String,
    pub p50_display: String,
    pub is_unrouted: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ScopeBoardView {
    pub title: &'static str,
    pub caption: &'static str,
    pub all_href: &'static str,
    pub error: Option<String>,
    pub rows: Vec<ScopeRowView>,
    pub has_rows: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ScopeRowView {
    pub label: String,
    pub href: Option<String>,
    pub requests_display: String,
    pub cost_display: String,
    pub share_display: String,
    pub is_unattributed: bool,
}

pub(super) fn queues(data: &OverviewData) -> QueuesView {
    let approvals = data.pending_approvals.as_ref().copied().unwrap_or(0);
    let anomalies: Vec<OverviewAnomalyRowView> = data
        .anomalies
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(anomaly_row)
        .collect();
    QueuesView {
        anomalies_tone: worst_tone(&anomalies),
        approvals_error: data
            .pending_approvals
            .as_ref()
            .err()
            .map(|e| format!("The approval queue could not be read: {e}")),
        approvals,
        approvals_tone: if approvals > 0 { "warn" } else { "ok" },
        anomalies_error: data
            .anomalies
            .as_ref()
            .err()
            .map(|e| format!("Open anomalies could not be read: {e}")),
        has_anomalies: !anomalies.is_empty(),
        anomaly_count: anomalies.len(),
        anomalies,
    }
}

fn anomaly_row(row: &UsageAnomalyRow) -> OverviewAnomalyRowView {
    let is_cost = row.metric == "cost";
    let show = |value: i64| {
        if is_cost {
            format_cost(value)
        } else {
            short_num(value)
        }
    };
    OverviewAnomalyRowView {
        metric: match row.metric.as_str() {
            "cost" => "Spend",
            "errors" => "Errors",
            _ => "Requests",
        },
        window_display: ago_display(Utc::now(), row.window_start),
        observed_display: show(row.observed),
        baseline_display: show(row.baseline),
        severity_display: severity_display(row.observed, row.baseline),
        severity_tone: anomaly_tone(row.observed, row.baseline),
    }
}

fn severity_display(observed: i64, baseline: i64) -> String {
    if baseline <= 0 {
        return "new".to_owned();
    }
    format!("{:.1}\u{d7} baseline", observed as f64 / baseline as f64)
}

// Why: the tile above the table carries the worst row's colour, so an
// operator sees red before they read a row.
fn worst_tone(rows: &[OverviewAnomalyRowView]) -> &'static str {
    if rows.iter().any(|r| r.severity_tone == "err") {
        "err"
    } else if rows.iter().any(|r| r.severity_tone == "warn") {
        "warn"
    } else {
        "ok"
    }
}

// Why: the models the window actually ran, in the order the Models tab lists
// them. Share is of every request the window carried, so the five rows shown
// tell the reader how much of the traffic they are looking at.
pub(super) fn models_board(range: OverviewRange, data: &OverviewData) -> ModelsBoardView {
    let rows = data.models.as_deref().unwrap_or(&[]);
    let total: i64 = rows.iter().map(|r| r.requests).sum();
    let preset = range.preset();
    let all_href = format!("/admin/analytics?tab=models&preset={preset}");
    let views: Vec<OverviewModelRowView> = rows
        .iter()
        .take(TOP_MODELS)
        .map(|r| model_row(r, total, &all_href))
        .collect();
    ModelsBoardView {
        error: data
            .models
            .as_ref()
            .err()
            .map(|e| format!("Model usage could not be read: {e}")),
        summary: format!("{} requests across {} models", short_num(total), rows.len()),
        has_rows: !views.is_empty(),
        rows: views,
        all_href,
    }
}

fn model_row(r: &ModelStatsRow, total: i64, all_href: &str) -> OverviewModelRowView {
    OverviewModelRowView {
        model: r.model.clone(),
        name_display: short_name(&r.model, '/'),
        qualifier_display: qualifier(&r.model, '/', r.provider.as_deref()),
        href: format!("{all_href}&model={}", urlencoding::encode(&r.model)),
        requests_display: short_num(r.requests),
        share_display: share_display(r.requests, total),
        cost_display: format_cost(r.cost_microdollars),
        p50_display: ms(r.p50_latency_ms),
        is_unrouted: r.is_unrouted,
    }
}

#[derive(Clone, Copy)]
pub(super) struct BoardSpec {
    pub title: &'static str,
    pub caption: &'static str,
    pub all_href: &'static str,
    pub unattributed: &'static str,
}

pub(super) fn scope_board(
    spec: BoardSpec,
    loaded: &Result<Vec<ScopeCostRow>, sqlx::Error>,
) -> ScopeBoardView {
    let rows = loaded.as_deref().unwrap_or(&[]);
    let total: i64 = rows.iter().map(|row| row.cost_microdollars).sum();
    let rows: Vec<ScopeRowView> = rows
        .iter()
        .take(usize::try_from(TOP_SCOPES).unwrap_or(5))
        .map(|row| scope_row(row, total, spec.all_href, spec.unattributed))
        .collect();
    ScopeBoardView {
        title: spec.title,
        caption: spec.caption,
        all_href: spec.all_href,
        error: loaded
            .as_ref()
            .err()
            .map(|e| format!("{} could not be read: {e}", spec.title)),
        has_rows: !rows.is_empty(),
        rows,
    }
}

// Why: the unattributed bucket is a real row with no page behind it — traffic
// with no user, a job or MCP actor, or a person no primary container covers.
// It is listed rather than dropped so the board sums back to the spend tile.
fn scope_row(row: &ScopeCostRow, total: i64, base: &str, absent: &str) -> ScopeRowView {
    let unattributed = row.is_unattributed();
    ScopeRowView {
        label: if unattributed {
            absent.to_owned()
        } else {
            row.label.clone()
        },
        href: (!unattributed).then(|| format!("{base}/{}", urlencoding::encode(&row.scope_id))),
        requests_display: short_num(row.requests),
        cost_display: format_cost(row.cost_microdollars),
        share_display: share_display(row.cost_microdollars, total),
        is_unattributed: unattributed,
    }
}

fn share_display(part: i64, total: i64) -> String {
    if total <= 0 {
        return "\u{2014}".to_owned();
    }
    format!("{:.0}%", part as f64 / total as f64 * 100.0)
}

// Why: an absolute timestamp makes a reader do the subtraction the state badge
// already did. The elapsed time is the fact the row is about.
fn ago_display(now: DateTime<Utc>, ts: DateTime<Utc>) -> String {
    let secs = (now - ts).num_seconds().max(0);
    if secs < 60 {
        format!("{secs}s ago")
    } else if secs < 3600 {
        format!("{}m ago", secs / 60)
    } else if secs < 86_400 {
        format!("{}h ago", secs / 3600)
    } else {
        format!("{}d ago", secs / 86_400)
    }
}
