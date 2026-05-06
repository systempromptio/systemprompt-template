//! `/admin/analytics/costs` — Grafana-style cost dashboard.
//!
//! Reads the same `ai_requests` spine as `/admin/analytics/requests` and the
//! `systemprompt analytics costs` CLI. KPI strip + cost-over-time spark +
//! token-throughput spark + per-model and per-provider rollups.

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
    fetch_cost_by_model, fetch_cost_by_provider, fetch_cost_kpis,
    fetch_token_throughput_over_time, CostKpis, ModelCostRow, ProviderCostRow, ThroughputBucket,
};
use crate::repositories::analytics_grp::request_stats::{fetch_cost_over_time, CostBucket};
use crate::repositories::governance_grp::time_range::{parse_time_range, TimeRange, TimeRangeQuery};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

const BASE_URL: &str = "/admin/analytics/costs";

#[derive(Debug, Deserialize)]
pub struct CostsQuery {
    pub from: Option<String>,
    pub to: Option<String>,
    pub preset: Option<String>,
}

pub async fn analytics_costs_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<CostsQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });

    let (kpis_res, models_res, providers_res, cost_res, tput_res) = tokio::join!(
        fetch_cost_kpis(&pool, range),
        fetch_cost_by_model(&pool, range),
        fetch_cost_by_provider(&pool, range),
        fetch_cost_over_time(&pool, range),
        fetch_token_throughput_over_time(&pool, range),
    );

    let kpis = kpis_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_cost_kpis failed");
        CostKpis::default()
    });
    let models = models_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_cost_by_model failed");
        Vec::new()
    });
    let providers = providers_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_cost_by_provider failed");
        Vec::new()
    });
    let cost_series = cost_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_cost_over_time failed");
        Vec::new()
    });
    let throughput_series = tput_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "fetch_token_throughput_over_time failed");
        Vec::new()
    });

    let cost_max = cost_series.iter().map(|b| b.cost_microdollars).max().unwrap_or(0);
    let throughput_max = throughput_series
        .iter()
        .map(|b| b.total_tokens)
        .max()
        .unwrap_or(0);

    let provider_max_cost = providers
        .iter()
        .map(|p| p.total_cost_microdollars)
        .max()
        .unwrap_or(0);

    let data = json!({
        "page": "analytics-costs",
        "title": "Cost Analytics",
        "cli_command": "systemprompt analytics costs summary",
        "time_range": time_range_context(&query, &range),
        "kpis": kpis_to_json(&kpis),
        "cost_series": cost_series.iter().map(cost_bucket_to_json).collect::<Vec<_>>(),
        "cost_max": cost_max,
        "throughput_series": throughput_series.iter().map(throughput_bucket_to_json).collect::<Vec<_>>(),
        "throughput_max": throughput_max,
        "models": models.iter().map(model_row_to_json).collect::<Vec<_>>(),
        "has_models": !models.is_empty(),
        "providers": providers
            .iter()
            .map(|p| provider_row_to_json(p, provider_max_cost))
            .collect::<Vec<_>>(),
        "has_providers": !providers.is_empty(),
        "has_data": kpis.requests > 0,
    });

    super::render_page(&engine, "analytics-costs", &data, &user_ctx, &mkt_ctx)
}

fn kpis_to_json(k: &CostKpis) -> serde_json::Value {
    json!({
        "requests": k.requests,
        "total_cost_display": format_cost(k.total_cost_microdollars),
        "avg_cost_display": format_cost(k.avg_cost_microdollars.round() as i64),
        "max_cost_display": format_cost(k.max_cost_microdollars),
        "input_tokens": k.input_tokens,
        "output_tokens": k.output_tokens,
        "total_tokens": k.total_tokens,
        "input_tokens_display": format_int(k.input_tokens),
        "output_tokens_display": format_int(k.output_tokens),
        "total_tokens_display": format_int(k.total_tokens),
        "tokens_per_minute_display": format!("{:.0}", k.tokens_per_minute),
        "cost_per_request_display": format_cost(k.cost_per_request_microdollars.round() as i64),
        "cost_per_1k_input_display": format_cost(k.cost_per_1k_input_microdollars.round() as i64),
        "cost_per_1k_output_display": format_cost(k.cost_per_1k_output_microdollars.round() as i64),
        "distinct_models": k.distinct_models,
        "distinct_providers": k.distinct_providers,
        "error_count": k.error_count,
    })
}

fn cost_bucket_to_json(b: &CostBucket) -> serde_json::Value {
    json!({
        "bucket_index": b.bucket_index,
        "bucket_start": b.bucket_start.to_rfc3339(),
        "cost_microdollars": b.cost_microdollars,
        "cost_display": format_cost(b.cost_microdollars),
    })
}

fn throughput_bucket_to_json(b: &ThroughputBucket) -> serde_json::Value {
    json!({
        "bucket_index": b.bucket_index,
        "bucket_start": b.bucket_start.to_rfc3339(),
        "input_tokens": b.input_tokens,
        "output_tokens": b.output_tokens,
        "total_tokens": b.total_tokens,
    })
}

fn model_row_to_json(r: &ModelCostRow) -> serde_json::Value {
    let total_tokens = r.input_tokens + r.output_tokens;
    json!({
        "provider": r.provider,
        "model": r.model,
        "calls": r.calls,
        "input_tokens": r.input_tokens,
        "output_tokens": r.output_tokens,
        "input_tokens_display": format_int(r.input_tokens),
        "output_tokens_display": format_int(r.output_tokens),
        "total_tokens_display": format_int(total_tokens),
        "total_cost_display": format_cost(r.total_cost_microdollars),
        "avg_cost_display": format_cost(r.avg_cost_microdollars.round() as i64),
        "avg_latency_display": format!("{} ms", r.avg_latency_ms.round() as i64),
        "errors": r.errors,
    })
}

fn provider_row_to_json(r: &ProviderCostRow, max_cost: i64) -> serde_json::Value {
    let pct = if max_cost > 0 {
        (r.total_cost_microdollars as f64 / max_cost as f64) * 100.0
    } else {
        0.0
    };
    json!({
        "provider": r.provider,
        "calls": r.calls,
        "total_cost_microdollars": r.total_cost_microdollars,
        "total_cost_display": format_cost(r.total_cost_microdollars),
        "input_tokens_display": format_int(r.input_tokens),
        "output_tokens_display": format_int(r.output_tokens),
        "distinct_models": r.distinct_models,
        "share_pct": format!("{pct:.1}"),
    })
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

fn time_range_context(query: &CostsQuery, range: &TimeRange) -> serde_json::Value {
    let preset = query.preset.clone().unwrap_or_else(|| {
        if query.from.is_some() && query.to.is_some() {
            "custom".to_string()
        } else {
            "24h".to_string()
        }
    });
    json!({
        "preset": preset,
        "from": range.from.to_rfc3339(),
        "to": range.to.to_rfc3339(),
        "base_url": BASE_URL,
        "query": "",
    })
}
