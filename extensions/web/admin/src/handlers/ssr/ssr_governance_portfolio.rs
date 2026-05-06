//! `/admin/governance/decisions/portfolio` — KPI portfolio for policy decisions.
//!
//! At-a-glance dashboard answering "are policies firing as expected, and what
//! changed since yesterday?". Six sparkline KPI cards, an anomalies-now panel,
//! a stacked-area chart of allow/deny over time, and two top-denies ranked
//! lists. There is no table on this page — every clickable element deep-links
//! into the Audit Trail (`/admin/governance/decisions`).

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;
use urlencoding::encode as urlencode;

use crate::repositories::governance_grp::anomalies::{fetch_decision_anomalies, Anomaly};
use crate::repositories::governance_grp::portfolio::{
    fetch_decision_buckets, fetch_governance_counts_in_range, fetch_top_denies, BucketFilter,
    BucketPolicyFilter, DecisionBucket, GovernanceCountsByPolicy, TopDeny, TopDenyGroup,
};
use crate::repositories::governance_grp::time_range::{
    parse_time_range, TimeRange, TimeRangePreset, TimeRangeQuery,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/governance/decisions/portfolio";
const AUDIT_URL: &str = "/admin/governance/decisions";
const KPI_SPARKLINE_BUCKETS: i32 = 24;
const CHART_BUCKETS: i32 = 24;
const TOP_DENIES_LIMIT: i64 = 8;
const ANOMALIES_LIMIT: usize = 5;

#[derive(Debug, Deserialize)]
pub struct PortfolioQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

pub async fn governance_portfolio_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<PortfolioQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });
    let preset_str = preset_to_str(&query, range);
    let bundle = fetch_portfolio_data(&pool, range).await;
    let data = render_context(&bundle, range, &preset_str);
    super::render_page(&engine, "governance-portfolio", &data, &user_ctx, &mkt_ctx)
}

struct PortfolioBundle {
    counts_now: GovernanceCountsByPolicy,
    counts_prior: GovernanceCountsByPolicy,
    chart_buckets: Vec<DecisionBucket>,
    spark_total: Vec<DecisionBucket>,
    spark_secret: Vec<DecisionBucket>,
    spark_blocklist: Vec<DecisionBucket>,
    spark_rate: Vec<DecisionBucket>,
    anomalies: Vec<Anomaly>,
    top_tool: Vec<TopDeny>,
    top_scope: Vec<TopDeny>,
}

async fn fetch_portfolio_data(pool: &PgPool, range: TimeRange) -> PortfolioBundle {
    let prior_range = prior_window(range);
    let bf_secret = BucketFilter {
        policies: BucketPolicyFilter::SecretScan,
        include_secret_reason: true,
    };
    let bf_blocklist = BucketFilter {
        policies: BucketPolicyFilter::Blocklist,
        include_secret_reason: false,
    };
    let bf_rate = BucketFilter {
        policies: BucketPolicyFilter::RateLimit,
        include_secret_reason: false,
    };

    let (counts_now, counts_prior, chart, spark_total, spark_secret, spark_blocklist,
         spark_rate, anomalies, top_tool, top_scope) = tokio::join!(
        fetch_governance_counts_in_range(pool, range),
        fetch_governance_counts_in_range(pool, prior_range),
        fetch_decision_buckets(pool, range, CHART_BUCKETS, BucketFilter::default()),
        fetch_decision_buckets(pool, range, KPI_SPARKLINE_BUCKETS, BucketFilter::default()),
        fetch_decision_buckets(pool, range, KPI_SPARKLINE_BUCKETS, bf_secret),
        fetch_decision_buckets(pool, range, KPI_SPARKLINE_BUCKETS, bf_blocklist),
        fetch_decision_buckets(pool, range, KPI_SPARKLINE_BUCKETS, bf_rate),
        fetch_decision_anomalies(pool, range),
        fetch_top_denies(pool, range, TopDenyGroup::Tool, TOP_DENIES_LIMIT),
        fetch_top_denies(pool, range, TopDenyGroup::AgentScope, TOP_DENIES_LIMIT),
    );

    PortfolioBundle {
        counts_now: log_or_default(counts_now, "fetch_governance_counts_in_range (now)"),
        counts_prior: counts_prior.unwrap_or_default(),
        chart_buckets: log_or_empty(chart, "fetch_decision_buckets (chart)"),
        spark_total: spark_total.unwrap_or_default(),
        spark_secret: spark_secret.unwrap_or_default(),
        spark_blocklist: spark_blocklist.unwrap_or_default(),
        spark_rate: spark_rate.unwrap_or_default(),
        anomalies: log_or_empty(anomalies, "fetch_decision_anomalies"),
        top_tool: top_tool.unwrap_or_default(),
        top_scope: top_scope.unwrap_or_default(),
    }
}

fn log_or_default<T: Default>(r: Result<T, sqlx::Error>, ctx: &str) -> T {
    r.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "{ctx} failed");
        T::default()
    })
}

fn log_or_empty<T>(r: Result<Vec<T>, sqlx::Error>, ctx: &str) -> Vec<T> {
    r.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "{ctx} failed");
        Vec::new()
    })
}

fn render_context(b: &PortfolioBundle, range: TimeRange, preset: &str) -> serde_json::Value {
    let kpis = build_kpis(
        &b.counts_now,
        &b.counts_prior,
        &b.spark_total,
        &b.spark_secret,
        &b.spark_blocklist,
        &b.spark_rate,
        preset,
    );
    let chart_data_json = serde_json::to_string(&json!({
        "buckets": chart_buckets_to_json(&b.chart_buckets, range),
    }))
    .unwrap_or_else(|_| "{\"buckets\":[]}".to_string());

    json!({
        "page": "governance-portfolio",
        "title": "Policy Decisions",
        "time_range": time_range_context(range, preset),
        "prior_window_label": preset_label(preset),
        "kpis": kpis,
        "anomalies": top_anomalies_to_json(&b.anomalies, range, preset),
        "has_chart_data": !b.chart_buckets.is_empty(),
        "chart_bucket_count": CHART_BUCKETS,
        "chart_data_json": chart_data_json,
        "top_denies_by_tool": top_denies_to_json(&b.top_tool, "tool_name", range, preset),
        "top_denies_by_scope": top_denies_to_json(&b.top_scope, "agent_scope", range, preset),
    })
}

// ─────────────────────────────────────────────────────────────────────────
// Time range helpers
// ─────────────────────────────────────────────────────────────────────────

fn prior_window(range: TimeRange) -> TimeRange {
    let span = range.to - range.from;
    TimeRange {
        from: range.from - span,
        to: range.from,
        preset: TimeRangePreset::Custom,
    }
}

fn preset_to_str(query: &PortfolioQuery, range: TimeRange) -> String {
    if let Some(p) = query.preset.as_deref() {
        if !p.is_empty() {
            return p.to_string();
        }
    }
    if query.from.is_some() && query.to.is_some() {
        return "custom".to_string();
    }
    match range.preset {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
    .to_string()
}

const fn preset_label(preset: &str) -> &str {
    match preset.as_bytes() {
        b"15m" => "15 minutes",
        b"1h" => "hour",
        b"7d" => "7 days",
        b"30d" => "30 days",
        b"custom" => "window",
        _ => "24 hours",
    }
}

fn time_range_context(range: TimeRange, preset: &str) -> serde_json::Value {
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": "",
    })
}

// ─────────────────────────────────────────────────────────────────────────
// KPI cards
// ─────────────────────────────────────────────────────────────────────────

#[allow(clippy::too_many_arguments)]
fn build_kpis(
    now: &GovernanceCountsByPolicy,
    prior: &GovernanceCountsByPolicy,
    spark_total: &[DecisionBucket],
    spark_secret: &[DecisionBucket],
    spark_blocklist: &[DecisionBucket],
    spark_rate: &[DecisionBucket],
    preset: &str,
) -> Vec<serde_json::Value> {
    let allow_rate_now = rate_pct(now.allowed, now.total);
    let allow_rate_prior = rate_pct(prior.allowed, prior.total);
    let deny_rate_now = rate_pct(now.denied, now.total);
    let deny_rate_prior = rate_pct(prior.denied, prior.total);

    let total_total = spark_total_values(spark_total);
    let allow_total = spark_allow_values(spark_total);
    let deny_total = spark_deny_values(spark_total);
    let deny_secret = spark_deny_values(spark_secret);
    let deny_blocklist = spark_deny_values(spark_blocklist);
    let deny_rate = spark_deny_values(spark_rate);
    let denies_label = format!("{} denies", format_int(now.denied));
    vec![
        kpi(
            "Total decisions",
            &format_int(now.total),
            None,
            None,
            &delta_int(now.total, prior.total, false),
            &total_total,
            "primary",
            &audit_url_for(&[("preset", preset)]),
        ),
        kpi(
            "Allow rate",
            &format_pct(allow_rate_now),
            None,
            Some("success"),
            &delta_pct(allow_rate_now, allow_rate_prior, false),
            &allow_total,
            "success",
            &audit_url_for(&[("preset", preset), ("decision", "allow")]),
        ),
        kpi(
            "Deny rate",
            &format_pct(deny_rate_now),
            Some(&denies_label),
            Some("danger"),
            &delta_pct(deny_rate_now, deny_rate_prior, true),
            &deny_total,
            "danger",
            &audit_url_for(&[("preset", preset), ("decision", "deny")]),
        ),
        kpi(
            "Secret-scan hits",
            &format_int(now.secret_scan),
            None,
            Some("danger"),
            &delta_int(now.secret_scan, prior.secret_scan, true),
            &deny_secret,
            "danger",
            &audit_url_for(&[("preset", preset), ("policy", "secret_scan")]),
        ),
        kpi(
            "Blocklist hits",
            &format_int(now.blocklist),
            None,
            Some("warning"),
            &delta_int(now.blocklist, prior.blocklist, true),
            &deny_blocklist,
            "warning",
            &audit_url_for(&[("preset", preset), ("policy", "tool_blocklist")]),
        ),
        kpi(
            "Rate-limit denies",
            &format_int(now.rate_limit),
            None,
            Some("info"),
            &delta_int(now.rate_limit, prior.rate_limit, true),
            &deny_rate,
            "info",
            &audit_url_for(&[("preset", preset), ("policy", "rate_limit")]),
        ),
    ]
}

#[allow(clippy::too_many_arguments)]
fn kpi(
    label: &str,
    value: &str,
    subtitle: Option<&str>,
    variant: Option<&str>,
    delta: &serde_json::Value,
    sparkline: &[i64],
    sparkline_color: &str,
    href: &str,
) -> serde_json::Value {
    json!({
        "label": label,
        "value": value,
        "subtitle": subtitle,
        "variant": variant,
        "delta": delta,
        "sparkline": sparkline,
        "sparkline_color": sparkline_color,
        "href": href,
    })
}

fn rate_pct(part: i64, total: i64) -> f64 {
    if total <= 0 {
        0.0
    } else {
        (part as f64 / total as f64) * 100.0
    }
}

fn format_int(n: i64) -> String {
    // Insert thin commas for readability.
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(bytes.len() + bytes.len() / 3);
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn format_pct(p: f64) -> String {
    format!("{p:.1}%")
}

/// Δ pill — `bad_when_up = true` means "deny going up is bad".
fn delta_int(now: i64, prior: i64, bad_when_up: bool) -> serde_json::Value {
    if prior == 0 && now == 0 {
        return delta_pill("flat", "0", "neutral");
    }
    if prior == 0 {
        return delta_pill("up", "new", if bad_when_up { "bad" } else { "good" });
    }
    let diff = now - prior;
    if diff == 0 {
        return delta_pill("flat", "0", "neutral");
    }
    let pct = (diff as f64 / prior.max(1) as f64) * 100.0;
    let direction = if diff > 0 { "up" } else { "down" };
    let tone = match (direction, bad_when_up) {
        ("up", true) | ("down", false) => "bad",
        ("up", false) | ("down", true) => "good",
        _ => "neutral",
    };
    delta_pill(direction, &format!("{:+.0}%", pct), tone)
}

fn delta_pct(now: f64, prior: f64, bad_when_up: bool) -> serde_json::Value {
    let diff = now - prior;
    if diff.abs() < 0.05 {
        return delta_pill("flat", "0%", "neutral");
    }
    let direction = if diff > 0.0 { "up" } else { "down" };
    let tone = match (direction, bad_when_up) {
        ("up", true) | ("down", false) => "bad",
        ("up", false) | ("down", true) => "good",
        _ => "neutral",
    };
    delta_pill(direction, &format!("{diff:+.1} pp"), tone)
}

fn delta_pill(direction: &str, magnitude: &str, tone: &str) -> serde_json::Value {
    json!({
        "direction": direction,
        "magnitude": magnitude,
        "tone": tone,
    })
}

fn spark_total_values(b: &[DecisionBucket]) -> Vec<i64> {
    b.iter().map(|x| x.allow + x.deny).collect()
}

fn spark_allow_values(b: &[DecisionBucket]) -> Vec<i64> {
    b.iter().map(|x| x.allow).collect()
}

fn spark_deny_values(b: &[DecisionBucket]) -> Vec<i64> {
    b.iter().map(|x| x.deny).collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Chart + anomaly serializers
// ─────────────────────────────────────────────────────────────────────────

fn chart_buckets_to_json(b: &[DecisionBucket], range: TimeRange) -> Vec<serde_json::Value> {
    if b.is_empty() {
        return Vec::new();
    }
    let span = range.to - range.from;
    let bucket_span = span / b.len().max(1) as i32;
    let total_seconds = span.num_seconds().max(0);
    let label_fn: fn(chrono::DateTime<chrono::Utc>) -> String = if total_seconds <= 24 * 3600 {
        |dt| {
            dt.with_timezone(&chrono::Local)
                .format("%H:%M")
                .to_string()
        }
    } else if total_seconds <= 7 * 24 * 3600 {
        |dt| {
            dt.with_timezone(&chrono::Local)
                .format("%a %H:%M")
                .to_string()
        }
    } else {
        |dt| {
            dt.with_timezone(&chrono::Local)
                .format("%b %d")
                .to_string()
        }
    };
    b.iter()
        .map(|bucket| {
            let bucket_start =
                range.from + bucket_span * bucket.bucket_index;
            json!({
                "label": label_fn(bucket_start),
                "allow": bucket.allow,
                "deny": bucket.deny,
            })
        })
        .collect()
}

fn top_anomalies_to_json(
    anomalies: &[Anomaly],
    range: TimeRange,
    preset: &str,
) -> Vec<serde_json::Value> {
    anomalies
        .iter()
        .take(ANOMALIES_LIMIT)
        .map(|a| {
            json!({
                "policy": a.policy,
                "decision": a.decision,
                "window_count": a.window_count,
                "baseline_mean_display": format_baseline(a.baseline_mean),
                "z_score_display": format_z(a.z_score),
                "affected_user_count": a.affected_user_count,
                "affected_agent_count": a.affected_agent_count,
                "audit_url": audit_url_with_range(
                    &[
                        ("preset", preset),
                        ("policy", a.policy.as_str()),
                        ("decision", a.decision.as_str()),
                    ],
                    range,
                    preset,
                ),
            })
        })
        .collect()
}

fn format_baseline(v: f64) -> String {
    if v < 1.0 {
        format!("{v:.2}")
    } else if v < 10.0 {
        format!("{v:.1}")
    } else {
        format!("{}", v.round() as i64)
    }
}

fn format_z(z: f64) -> String {
    if z >= 999.0 {
        ">999".to_string()
    } else {
        format!("{z:.1}")
    }
}

fn top_denies_to_json(
    rows: &[TopDeny],
    param: &str,
    range: TimeRange,
    preset: &str,
) -> Vec<serde_json::Value> {
    let max = rows.iter().map(|r| r.deny_count).max().unwrap_or(1).max(1);
    rows.iter()
        .map(|r| {
            let scale = if max == 0 {
                0.0
            } else {
                (r.deny_count as f64 / max as f64).clamp(0.0, 1.0)
            };
            json!({
                "label": r.label,
                "key": r.key,
                "deny_count": r.deny_count,
                "bar_scale": format!("{scale:.3}"),
                "audit_url": audit_url_with_range(
                    &[
                        ("preset", preset),
                        ("decision", "deny"),
                        (param, r.key.as_str()),
                    ],
                    range,
                    preset,
                ),
            })
        })
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────
// Audit Trail URL builders
// ─────────────────────────────────────────────────────────────────────────

fn audit_url_for(params: &[(&str, &str)]) -> String {
    let qs = params
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| format!("{}={}", k, urlencode(v)))
        .collect::<Vec<_>>()
        .join("&");
    if qs.is_empty() {
        AUDIT_URL.to_string()
    } else {
        format!("{AUDIT_URL}?{qs}")
    }
}

/// Same as `audit_url_for`, but always emits absolute `from`/`to` so the
/// destination page lands on the same window even if `preset` defaults to a
/// shorter window on the audit side.
fn audit_url_with_range(
    params: &[(&str, &str)],
    range: TimeRange,
    preset: &str,
) -> String {
    let mut all: Vec<(String, String)> = params
        .iter()
        .filter(|(_, v)| !v.is_empty())
        .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
        .collect();
    if !all.iter().any(|(k, _)| k == "preset") {
        all.push(("preset".to_string(), preset.to_string()));
    }
    if preset == "custom" {
        all.push(("from".to_string(), range.from.to_rfc3339()));
        all.push(("to".to_string(), range.to.to_rfc3339()));
    }
    let qs = all
        .iter()
        .map(|(k, v)| format!("{}={}", k, urlencode(v)))
        .collect::<Vec<_>>()
        .join("&");
    if qs.is_empty() {
        AUDIT_URL.to_string()
    } else {
        format!("{AUDIT_URL}?{qs}")
    }
}

