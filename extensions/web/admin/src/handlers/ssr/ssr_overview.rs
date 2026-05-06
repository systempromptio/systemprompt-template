//! `/admin/overview` — five real-time live panes plus an index.
//!
//! Each page renders an SSR snapshot from existing repository functions, then
//! the matching JS module opens an `EventSource` to `/admin/api/sse/overview/<pane>`
//! to patch the page in real time. The visual scaffolding follows the
//! Cost Analytics gold-standard: KPI strip → sparkline / chart → rollup table /
//! live ticker.

use std::sync::Arc;

use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::analytics_grp::cost_stats::{
    fetch_cost_by_model, fetch_cost_by_provider, fetch_cost_kpis, fetch_recent_requests,
};
use crate::repositories::analytics_grp::request_stats::fetch_cost_over_time;
use crate::repositories::governance_grp::governance::fetch_per_policy_counts;
use crate::repositories::governance_grp::paged::{
    fetch_decisions_paged, DecisionFilter, SortSpec,
};
use crate::repositories::governance_grp::portfolio::{
    fetch_decision_buckets, fetch_governance_counts_in_range, fetch_top_denies, BucketFilter,
    TopDenyGroup,
};
use crate::repositories::governance_grp::time_range::{
    parse_time_range, TimeRange, TimeRangePreset, TimeRangeQuery,
};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const HOUR_RANGE_PRESET: TimeRangePreset = TimeRangePreset::Hour1;

fn last_hour() -> TimeRange {
    let now = chrono::Utc::now();
    TimeRange {
        from: now - chrono::Duration::hours(1),
        to: now,
        preset: HOUR_RANGE_PRESET,
    }
}

fn last_24h() -> TimeRange {
    let now = chrono::Utc::now();
    TimeRange {
        from: now - chrono::Duration::hours(24),
        to: now,
        preset: TimeRangePreset::Hours24,
    }
}

fn require_admin(user_ctx: &UserContext) -> Option<Response> {
    if user_ctx.is_admin {
        None
    } else {
        Some((StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response())
    }
}

fn format_cost(microdollars: i64) -> String {
    let dollars = microdollars as f64 / 1_000_000.0;
    if dollars == 0.0 {
        "$0".to_string()
    } else if dollars.abs() < 0.01 {
        format!("${dollars:.6}")
    } else {
        format!("${dollars:.4}")
    }
}

fn format_int(v: i64) -> String {
    let neg = v < 0;
    let mut digits: Vec<char> = v.unsigned_abs().to_string().chars().collect();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    let mut count = 0;
    while let Some(c) = digits.pop() {
        if count > 0 && count % 3 == 0 {
            out.push(',');
        }
        out.push(c);
        count += 1;
    }
    if neg {
        out.push('-');
    }
    out.chars().rev().collect()
}

// =============================================================================
// /admin/overview — index (5 summary cards)
// =============================================================================

pub async fn overview_index_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let range = last_hour();
    let (kpis_res, gov_res) = tokio::join!(
        fetch_cost_kpis(&pool, range),
        fetch_governance_counts_in_range(&pool, range),
    );

    let kpis = kpis_res.unwrap_or_default();
    let gov = gov_res.unwrap_or_default();

    let dollars_per_min = if kpis.total_cost_microdollars > 0 {
        kpis.total_cost_microdollars as f64 / 1_000_000.0 / 60.0
    } else {
        0.0
    };

    let data = json!({
        "page": "overview-index",
        "title": "Live Overview",
        "panes": [
            {
                "key": "pulse",
                "label": "Pulse",
                "tagline": "Gateway heartbeat",
                "metric": format!("{}", kpis.requests),
                "metric_label": "requests / hr",
                "href": "/admin/overview/pulse",
                "tone": "info"
            },
            {
                "key": "identity",
                "label": "Identity",
                "tagline": "Active users & agents",
                "metric": format!("{}", kpis.requests),
                "metric_label": "requests last hour",
                "href": "/admin/overview/identity",
                "tone": "info"
            },
            {
                "key": "cost",
                "label": "Cost & models",
                "tagline": "Spend in motion",
                "metric": format_cost(kpis.total_cost_microdollars),
                "metric_label": format!("{:.4} $/min", dollars_per_min),
                "href": "/admin/overview/cost",
                "tone": "info"
            },
            {
                "key": "governance",
                "label": "Governance flow",
                "tagline": "Pipeline decisions",
                "metric": format!("{}", gov.total),
                "metric_label": format!("{} denials", gov.denied),
                "href": "/admin/overview/governance",
                "tone": if gov.denied > 0 { "warn" } else { "info" }
            },
            {
                "key": "services",
                "label": "Services & tools",
                "tagline": "Upstream + MCP health",
                "metric": format!("{}", kpis.distinct_models),
                "metric_label": format!("{} providers", kpis.distinct_providers),
                "href": "/admin/overview/services",
                "tone": if kpis.error_count > 0 { "warn" } else { "info" }
            }
        ],
    });

    super::render_page(&engine, "overview-index", &data, &user_ctx, &mkt_ctx)
}

// =============================================================================
// /admin/overview/pulse — gateway-wide live ticker
// =============================================================================

pub async fn overview_pulse_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let range = last_hour();
    let (kpis, recent) = tokio::join!(
        fetch_cost_kpis(&pool, range),
        fetch_recent_requests(&pool, 50),
    );
    let kpis = kpis.unwrap_or_default();
    let recent = recent.unwrap_or_default();

    let elapsed_min = (range.to - range.from).num_seconds().max(1) as f64 / 60.0;
    let req_per_min = kpis.requests as f64 / elapsed_min;
    let error_rate = if kpis.requests > 0 {
        (kpis.error_count as f64 / kpis.requests as f64) * 100.0
    } else {
        0.0
    };
    let dollars_per_hr = (kpis.total_cost_microdollars as f64 / 1_000_000.0)
        * (3600.0 / (range.to - range.from).num_seconds().max(1) as f64);

    let recent_rows: Vec<serde_json::Value> = recent
        .iter()
        .map(|r| {
            let is_error = !matches!(
                r.status.as_str(),
                "completed" | "pending" | "streaming" | "ok" | "success"
            );
            let is_pending = r.status == "pending";
            let actor = r
                .department
                .as_deref()
                .filter(|s| !s.is_empty())
                .or(r.display_name.as_deref())
                .unwrap_or(&r.user_id[..8.min(r.user_id.len())]);
            let latency_display = r.latency_ms.map(|l| format!("{l}ms"));
            let cost_nonzero = r.cost_microdollars > 0;
            let error_href = r.trace_id.as_deref().map(|tid| {
                format!("/admin/analytics/requests?q={tid}&preset=7d")
            });
            json!({
                "id":            r.id,
                "user_id":       r.user_id,
                "display_name":  r.display_name,
                "department":    r.department,
                "actor":         actor,
                "trace_id":      r.trace_id,
                "session_id":    r.session_id,
                "context_id":    r.context_id,
                "model":         r.model,
                "status":        r.status,
                "is_error":      is_error,
                "is_pending":    is_pending,
                "latency":       latency_display,
                "cost_nonzero":  cost_nonzero,
                "error_href":    error_href,
                "error_message": r.error_message,
                "cost_display":  format!("${:.4}", r.cost_microdollars as f64 / 1_000_000.0),
                "latency_ms":    r.latency_ms.map_or_else(|| "—".to_string(), |l| format!("{l}ms")),
                "time":          r.created_at.format("%H:%M:%S").to_string(),
            })
        })
        .collect();

    let data = json!({
        "page": "overview-pulse",
        "title": "Pulse — Live gateway",
        "kpis": {
            "requests": kpis.requests,
            "req_per_min": format!("{req_per_min:.1}"),
            "distinct_models": kpis.distinct_models,
            "distinct_providers": kpis.distinct_providers,
            "burn_rate_display": format!("${dollars_per_hr:.2}/hr"),
            "error_count": kpis.error_count,
            "error_rate_display": format!("{error_rate:.1}%"),
        },
        "recent_requests": recent_rows,
        "sse_url": "/admin/api/sse/overview/pulse",
    });

    super::render_page(&engine, "overview-pulse", &data, &user_ctx, &mkt_ctx)
}

// =============================================================================
// /admin/overview/identity — who is doing what right now
// =============================================================================

#[derive(Debug, Deserialize)]
pub struct IdentityPageQuery {
    pub sort: Option<String>,
    pub dir: Option<String>,
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn overview_identity_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(params): Query<IdentityPageQuery>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let sort = crate::repositories::IdentitySort::parse(params.sort.as_deref());
    let dir = crate::repositories::IdentitySortDir::parse(params.dir.as_deref());
    let q = params.q.as_deref().map(str::trim).filter(|s| !s.is_empty());
    let limit = params.limit.unwrap_or(50).clamp(10, 200);
    let offset = params.offset.unwrap_or(0).max(0);

    let (rows, total) = crate::repositories::fetch_user_identity_rows(
        &pool, sort, dir, q, limit, offset,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_user_identity_rows failed");
        (Vec::new(), 0)
    });

    let total_users = total;
    let active_users = rows.iter().filter(|r| r.is_active).count() as i64;
    let total_tokens: i64 = rows.iter().map(|r| r.tokens).sum();
    let total_denies: i64 = rows.iter().map(|r| r.denies).sum();
    let total_cost: i64 = rows.iter().map(|r| r.cost_microdollars).sum();

    let table_rows: Vec<serde_json::Value> = rows
        .iter()
        .map(|r| {
            let display = r
                .display_name
                .clone()
                .filter(|s| !s.is_empty())
                .or_else(|| r.email.clone())
                .unwrap_or_else(|| r.user_id.chars().take(8).collect());
            let cost_dollars = (r.cost_microdollars as f64) / 1_000_000.0;
            json!({
                "user_id": r.user_id,
                "display_name": display,
                "email": r.email,
                "department": r.department,
                "is_active": r.is_active,
                "last_active": r.last_active.map(|t| t.to_rfc3339()),
                "requests": r.requests,
                "sessions": r.sessions,
                "contexts": r.contexts,
                "models": r.models,
                "tokens": r.tokens,
                "cost_microdollars": r.cost_microdollars,
                "cost_display": format!("${cost_dollars:.2}"),
                "denies": r.denies,
                "has_denies": r.denies > 0,
                "secret_breaches": r.secret_breaches,
                "scope_violations": r.scope_violations,
            })
        })
        .collect();

    let sort_slug = sort.slug();
    let dir_slug = dir.slug();
    let next_offset = offset + limit;
    let prev_offset = (offset - limit).max(0);

    let columns = [
        ("name", "User"),
        ("sessions", "Sessions"),
        ("contexts", "Contexts"),
        ("tokens", "Tokens"),
        ("cost", "Cost"),
        ("denies", "Denies"),
        ("last_active", "Last active"),
    ];
    let sort_links: Vec<serde_json::Value> = columns
        .iter()
        .map(|(slug, label)| {
            let active = *slug == sort_slug;
            let next_dir = if active && dir_slug == "desc" { "asc" } else { "desc" };
            json!({
                "slug": slug,
                "label": label,
                "active": active,
                "next_dir": next_dir,
                "indicator": if active { if dir_slug == "desc" { "↓" } else { "↑" } } else { "" },
            })
        })
        .collect();

    let data = json!({
        "page": "overview-identity",
        "title": "Identity",
        "subtitle": "Every user, what they are doing with AI, and where to drill in.",
        "kpis": {
            "users": total_users,
            "active": active_users,
            "tokens": total_tokens,
            "denies": total_denies,
            "cost_display": format!("${:.2}", (total_cost as f64) / 1_000_000.0),
        },
        "rows": table_rows,
        "has_rows": !rows.is_empty(),
        "sort": sort_slug,
        "dir": dir_slug,
        "q": q.unwrap_or(""),
        "sort_links": sort_links,
        "pagination": {
            "limit": limit,
            "offset": offset,
            "total": total,
            "page_start": offset + 1,
            "page_end": (offset + (rows.len() as i64)).min(total),
            "has_prev": offset > 0,
            "has_next": next_offset < total,
            "next_offset": next_offset,
            "prev_offset": prev_offset,
        },
    });

    super::render_page(&engine, "overview-identity", &data, &user_ctx, &mkt_ctx)
}

// =============================================================================
// /admin/overview/cost — live cost & models
// =============================================================================

pub async fn overview_cost_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let range = last_hour();
    let day = last_24h();

    let (kpis_res, models_res, providers_res, series_res, recent_res) = tokio::join!(
        fetch_cost_kpis(&pool, range),
        fetch_cost_by_model(&pool, range),
        fetch_cost_by_provider(&pool, range),
        fetch_cost_over_time(&pool, day),
        fetch_recent_requests(&pool, 50),
    );

    let kpis = kpis_res.unwrap_or_default();
    let models = models_res.unwrap_or_default();
    let providers = providers_res.unwrap_or_default();
    let series = series_res.unwrap_or_default();
    let recent = recent_res.unwrap_or_default();

    let recent_rows: Vec<serde_json::Value> = recent
        .iter()
        .map(|r| {
            let is_error = !matches!(
                r.status.as_str(),
                "completed" | "pending" | "streaming" | "ok" | "success"
            );
            json!({
                "time":     r.created_at.format("%H:%M:%S").to_string(),
                "msg":      format!("{}  user={}", r.model, r.user_id),
                "is_error": is_error,
            })
        })
        .collect();

    let cost_max = series.iter().map(|b| b.cost_microdollars).max().unwrap_or(0);
    let provider_max = providers
        .iter()
        .map(|p| p.total_cost_microdollars)
        .max()
        .unwrap_or(0);

    let elapsed_min = (range.to - range.from).num_seconds().max(1) as f64 / 60.0;
    let dollars_per_min = (kpis.total_cost_microdollars as f64 / 1_000_000.0) / elapsed_min;
    let projected_daily = dollars_per_min * 60.0 * 24.0;

    let data = json!({
        "page": "overview-cost",
        "title": "Cost & models — Live",
        "kpis": {
            "spend_display": format_cost(kpis.total_cost_microdollars),
            "spend_per_min_display": format!("${dollars_per_min:.4}/min"),
            "projected_daily_display": format!("${projected_daily:.2}"),
            "tokens_display": format_int(kpis.input_tokens + kpis.output_tokens),
            "tokens_per_min_display": format!("{:.0}", kpis.tokens_per_minute),
            "top_model": models.first().map_or_else(|| "—".to_string(), |m| m.model.clone()),
            "errors": kpis.error_count,
        },
        "cost_series": series.iter().map(|b| json!({
            "bucket_start": b.bucket_start.to_rfc3339(),
            "cost_microdollars": b.cost_microdollars,
            "cost_display": format_cost(b.cost_microdollars),
        })).collect::<Vec<_>>(),
        "cost_max": cost_max,
        "providers": providers.iter().map(|p| {
            let pct = if provider_max > 0 {
                (p.total_cost_microdollars as f64 / provider_max as f64) * 100.0
            } else { 0.0 };
            json!({
                "provider": p.provider,
                "total_cost_display": format_cost(p.total_cost_microdollars),
                "calls": p.calls,
                "share_pct": format!("{pct:.1}"),
            })
        }).collect::<Vec<_>>(),
        "has_providers": !providers.is_empty(),
        "models": models.iter().take(10).map(|m| json!({
            "model": m.model,
            "provider": m.provider,
            "calls": m.calls,
            "total_cost_display": format_cost(m.total_cost_microdollars),
            "errors": m.errors,
        })).collect::<Vec<_>>(),
        "has_models": !models.is_empty(),
        "recent_requests": recent_rows,
        "sse_url": "/admin/api/sse/overview/cost",
    });

    super::render_page(&engine, "overview-cost", &data, &user_ctx, &mkt_ctx)
}

// =============================================================================
// /admin/overview/governance — governance cockpit
// =============================================================================

#[derive(Debug, Default, Deserialize)]
pub struct GovernanceOverviewQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

pub async fn overview_governance_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(q): Query<GovernanceOverviewQuery>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: q.from.clone(),
        to: q.to.clone(),
        preset: q.preset.clone(),
    });

    // Hooks come from disk — read synchronously before the join.
    let hooks = match super::get_services_path() {
        Ok(services_path) => super::ssr_hooks::list_hooks_from_filesystem(&services_path),
        Err(_) => Vec::new(),
    };

    let deny_filter = DecisionFilter {
        decision: Some("deny".to_string()),
        ..Default::default()
    };
    let bucket_count = 30_i32;
    let (counts_res, denies_res, recent_denials_res, policies_res, buckets_res) = tokio::join!(
        fetch_governance_counts_in_range(&pool, range),
        fetch_top_denies(&pool, range, TopDenyGroup::Tool, 10),
        fetch_decisions_paged(&pool, &deny_filter, range, SortSpec::default(), 25, 0),
        fetch_per_policy_counts(&pool),
        fetch_decision_buckets(&pool, range, bucket_count, BucketFilter::default()),
    );

    let counts = counts_res.unwrap_or_default();
    let denies = denies_res.unwrap_or_default();
    let recent_denials = recent_denials_res.map(|(rows, _)| rows).unwrap_or_default();
    let policies = policies_res.unwrap_or_default();
    let buckets = buckets_res.unwrap_or_default();

    let denial_rows: Vec<serde_json::Value> = recent_denials
        .iter()
        .map(|d| {
            let is_breach = d.policy.eq_ignore_ascii_case("secret_scan")
                || d.policy.eq_ignore_ascii_case("secret_injection");
            json!({
                "id":        d.id,
                "time":      d.created_at.format("%H:%M:%S").to_string(),
                "user_id":   d.user_id,
                "tool":      d.tool_name,
                "policy":    d.policy,
                "reason":    d.reason,
                "is_breach": is_breach,
                "detail_href": format!("/admin/governance/decisions/{}", d.id),
            })
        })
        .collect();

    let deny_rate = if counts.total > 0 {
        (counts.denied as f64 / counts.total as f64) * 100.0
    } else {
        0.0
    };
    let allow_rate = if counts.total > 0 {
        (counts.allowed as f64 / counts.total as f64) * 100.0
    } else {
        0.0
    };

    let now_utc = chrono::Utc::now();
    let policy_rows: Vec<serde_json::Value> = {
        let mut rows: Vec<serde_json::Value> = policies
            .iter()
            .map(|p| {
                let total = p.allowed + p.denied;
                let dr = if total > 0 {
                    (p.denied as f64 / total as f64) * 100.0
                } else {
                    0.0
                };
                let last_fired = p
                    .last_at
                    .map(|t| format_relative(now_utc, t))
                    .unwrap_or_else(|| "—".to_string());
                let last_at_iso = p
                    .last_at
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default();
                let denied = p.denied;
                json!({
                    "policy":        p.policy,
                    "allowed":       format_int(p.allowed),
                    "denied":        format_int(denied),
                    "denied_raw":    denied,
                    "total":         format_int(total),
                    "deny_rate":     format!("{dr:.1}%"),
                    "deny_rate_raw": dr,
                    "last_fired":    last_fired,
                    "last_at":       last_at_iso,
                    "tone":          policy_tone(denied, dr),
                    "detail_href":   format!("/admin/governance/decisions?policy={}&preset=24h", p.policy),
                })
            })
            .collect();
        rows.sort_by(|a, b| {
            let ad = a.get("denied_raw").and_then(|v| v.as_i64()).unwrap_or(0);
            let bd = b.get("denied_raw").and_then(|v| v.as_i64()).unwrap_or(0);
            bd.cmp(&ad)
        });
        rows
    };

    let hook_rows: Vec<serde_json::Value> = hooks
        .iter()
        .map(|h| {
            json!({
                "id":         h.id.as_str(),
                "name":       if h.name.is_empty() { h.id.as_str().to_string() } else { h.name.clone() },
                "plugin_id":  h.plugin_id,
                "event":      h.event,
                "matcher":    if h.matcher.is_empty() { "*".to_string() } else { h.matcher.clone() },
                "is_async":   h.is_async,
                "enabled":    h.enabled,
                "system":     h.system,
                "valid":      !h.command.is_empty() && !h.event.is_empty(),
                "detail_href": "/admin/governance/hooks",
            })
        })
        .collect();

    // Decision-buckets chart: precompute geometry so the template can render
    // pure SVG without any client-side math.
    let max_total = buckets
        .iter()
        .map(|b| b.allow + b.deny)
        .max()
        .unwrap_or(0)
        .max(1) as f64;
    let bar_count = buckets.len().max(1);
    let bar_width = 100.0 / bar_count as f64;
    let chart_buckets: Vec<serde_json::Value> = buckets
        .iter()
        .enumerate()
        .map(|(i, b)| {
            let total = (b.allow + b.deny) as f64;
            let h_total = (total / max_total) * 100.0;
            let h_deny = if total > 0.0 {
                (b.deny as f64 / max_total) * 100.0
            } else {
                0.0
            };
            let h_allow = (h_total - h_deny).max(0.0);
            let x = (i as f64) * bar_width;
            let allow_y = 100.0 - h_total;
            let deny_y = allow_y + h_allow;
            json!({
                "i":         i,
                "x":         format!("{x:.4}"),
                "w":         format!("{:.4}", (bar_width - 0.4).max(0.2)),
                "allow":     b.allow,
                "deny":      b.deny,
                "allow_y":   format!("{allow_y:.4}"),
                "allow_h":   format!("{h_allow:.4}"),
                "deny_y":    format!("{deny_y:.4}"),
                "deny_h":    format!("{h_deny:.4}"),
            })
        })
        .collect();

    let preset_keys = [
        ("15m", "15 min"),
        ("1h", "1 hour"),
        ("24h", "24 hours"),
        ("7d", "7 days"),
        ("30d", "30 days"),
    ];
    let active_preset = preset_for(range.preset);
    let time_presets: Vec<serde_json::Value> = preset_keys
        .iter()
        .map(|(key, label)| {
            json!({
                "key":    key,
                "label":  label,
                "href":   format!("/admin/overview/governance?preset={key}"),
                "active": *key == active_preset,
            })
        })
        .collect();

    let total_hooks = hook_rows.len();
    let enabled_hooks = hooks.iter().filter(|h| h.enabled).count();

    let data = json!({
        "page": "overview-governance",
        "title": "Governance — Cockpit",
        "kpis": {
            "total":              format_int(counts.total),
            "total_raw":          counts.total,
            "allowed":            format_int(counts.allowed),
            "allowed_raw":        counts.allowed,
            "denied":             format_int(counts.denied),
            "denied_raw":         counts.denied,
            "secret_breaches":    format_int(counts.secret_scan),
            "secret_breaches_raw": counts.secret_scan,
            "deny_rate_display":  format!("{deny_rate:.1}%"),
            "allow_rate_display": format!("{allow_rate:.1}%"),
        },
        "time_presets":  time_presets,
        "active_preset": active_preset,
        "chart": {
            "buckets":     chart_buckets,
            "has_data":    counts.total > 0,
            "bucket_count": bucket_count,
        },
        "policies":     policy_rows,
        "has_policies": !policies.is_empty(),
        "hooks":        hook_rows,
        "has_hooks":    !hook_rows.is_empty(),
        "hooks_total":  total_hooks,
        "hooks_enabled": enabled_hooks,
        "top_denies": denies.iter().map(|d| json!({
            "label": d.label,
            "count": format_int(d.deny_count),
        })).collect::<Vec<_>>(),
        "has_denies":     !denies.is_empty(),
        "recent_denials": denial_rows,
        "has_denials":    !recent_denials.is_empty(),
        "sse_url":        "/admin/api/sse/overview/governance",
    });

    super::render_page(&engine, "overview-governance", &data, &user_ctx, &mkt_ctx)
}

const fn preset_for(p: TimeRangePreset) -> &'static str {
    match p {
        TimeRangePreset::Min15 => "15m",
        TimeRangePreset::Hour1 => "1h",
        TimeRangePreset::Hours24 => "24h",
        TimeRangePreset::Days7 => "7d",
        TimeRangePreset::Days30 => "30d",
        TimeRangePreset::Custom => "custom",
    }
}

const fn policy_tone(denied: i64, deny_rate: f64) -> &'static str {
    if denied == 0 {
        "ok"
    } else if deny_rate >= 25.0 {
        "danger"
    } else {
        "warn"
    }
}

fn format_relative(now: chrono::DateTime<chrono::Utc>, t: chrono::DateTime<chrono::Utc>) -> String {
    let delta = now - t;
    let secs = delta.num_seconds();
    if secs < 0 {
        return "just now".to_string();
    }
    if secs < 60 {
        return format!("{secs}s ago");
    }
    let mins = delta.num_minutes();
    if mins < 60 {
        return format!("{mins}m ago");
    }
    let hours = delta.num_hours();
    if hours < 48 {
        return format!("{hours}h ago");
    }
    let days = delta.num_days();
    format!("{days}d ago")
}

// =============================================================================
// /admin/overview/services — services & MCP tools
// =============================================================================

pub async fn overview_services_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if let Some(deny) = require_admin(&user_ctx) {
        return deny;
    }

    let range = last_hour();
    let (kpis_res, models_res, recent_res) = tokio::join!(
        fetch_cost_kpis(&pool, range),
        fetch_cost_by_model(&pool, range),
        fetch_recent_requests(&pool, 50),
    );

    let kpis = kpis_res.unwrap_or_default();
    let models = models_res.unwrap_or_default();
    let recent = recent_res.unwrap_or_default();

    let recent_rows: Vec<serde_json::Value> = recent
        .iter()
        .map(|r| {
            let is_error = !matches!(
                r.status.as_str(),
                "completed" | "pending" | "streaming" | "ok" | "success"
            );
            json!({
                "time":     r.created_at.format("%H:%M:%S").to_string(),
                "msg":      format!("req: {}  status={}", r.model, r.status),
                "is_error": is_error,
            })
        })
        .collect();

    let elapsed_min = (range.to - range.from).num_seconds().max(1) as f64 / 60.0;
    let avg_latency = if models.is_empty() {
        0.0
    } else {
        models.iter().map(|m| m.avg_latency_ms).sum::<f64>() / models.len() as f64
    };

    let data = json!({
        "page": "overview-services",
        "title": "Services & tools — Live",
        "kpis": {
            "providers": kpis.distinct_providers,
            "models": kpis.distinct_models,
            "requests_per_min": format!("{:.1}", kpis.requests as f64 / elapsed_min),
            "avg_latency_display": format!("{} ms", avg_latency.round() as i64),
            "errors": kpis.error_count,
        },
        "providers": models.iter().fold(Vec::<serde_json::Value>::new(), |mut acc, m| {
            if !acc.iter().any(|p| p.get("name").and_then(|v| v.as_str()) == Some(m.provider.as_str())) {
                acc.push(json!({
                    "name": m.provider,
                    "calls": m.calls,
                    "avg_latency_display": format!("{} ms", m.avg_latency_ms.round() as i64),
                    "errors": m.errors,
                }));
            }
            acc
        }),
        "models": models.iter().take(10).map(|m| json!({
            "model": m.model,
            "provider": m.provider,
            "calls": m.calls,
            "avg_latency_display": format!("{} ms", m.avg_latency_ms.round() as i64),
            "errors": m.errors,
        })).collect::<Vec<_>>(),
        "has_models": !models.is_empty(),
        "recent_requests": recent_rows,
        "sse_url": "/admin/api/sse/overview/services",
    });

    super::render_page(&engine, "overview-services", &data, &user_ctx, &mkt_ctx)
}
