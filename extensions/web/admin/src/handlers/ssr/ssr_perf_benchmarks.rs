use std::sync::Arc;

use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use chrono::{DateTime, Utc};
use serde_json::json;
use sqlx::PgPool;

use super::ACCESS_DENIED_HTML;

#[derive(Debug)]
struct BenchRunRow {
    session_id: String,
    total_decisions: i64,
    allowed: i64,
    denied: i64,
    duration_seconds: f64,
    first_at: DateTime<Utc>,
}

#[derive(Debug)]
struct HourThroughputRow {
    bucket: DateTime<Utc>,
    decisions: i64,
}

/// SSR page for `/admin/performance/benchmarks` — surfaces the most recent
/// `demo/performance/02-load-test.sh` run by grouping governance decisions
/// with session ids beginning with `bench-`, plus a 24-hour throughput
/// histogram and an empty-state when no runs are recorded.
pub async fn perf_benchmarks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (runs_res, hourly_res) =
        tokio::join!(fetch_bench_runs(&pool), fetch_hourly_throughput(&pool));

    let runs = runs_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch benchmark runs");
        vec![]
    });
    let hourly = hourly_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch hourly throughput");
        vec![]
    });

    // JSON: template context for Handlebars rendering
    let runs_json: Vec<serde_json::Value> = runs
        .iter()
        .map(|r| {
            let rps = if r.duration_seconds > 0.0 {
                r.total_decisions as f64 / r.duration_seconds
            } else {
                r.total_decisions as f64
            };
            json!({
                "session_id": r.session_id,
                "total_decisions": r.total_decisions,
                "allowed": r.allowed,
                "denied": r.denied,
                "duration_seconds": format!("{:.2}", r.duration_seconds),
                "rps": format!("{rps:.0}"),
                "started_at": r.first_at,
            })
        })
        .collect();

    let hourly_json: Vec<serde_json::Value> = hourly
        .iter()
        .map(|h| json!({ "bucket": h.bucket, "decisions": h.decisions }))
        .collect();

    let latest = runs.first();
    let latest_rps = latest.map_or(0.0, |r| {
        if r.duration_seconds > 0.0 {
            r.total_decisions as f64 / r.duration_seconds
        } else {
            r.total_decisions as f64
        }
    });

    let data = json!({
        "page": "perf-benchmarks",
        "title": "Performance Benchmarks",
        "cli_command": "systemprompt infra logs view --level info --since 5m",
        "demo_script": "demo/performance/02-load-test.sh",
        "page_stats": [
            {"key": "runs", "value": runs_json.len(), "label": "Recent runs"},
            {"key": "last_rps", "value": format!("{latest_rps:.0}"), "label": "Last rps"},
            {"key": "last_n", "value": latest.map_or(0, |r| r.total_decisions), "label": "Decisions"},
        ],
        "runs": runs_json,
        "has_runs": !runs_json.is_empty(),
        "hourly": hourly_json,
        "has_hourly": !hourly_json.is_empty(),
    });
    super::render_page(&engine, "perf-benchmarks", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_bench_runs(pool: &PgPool) -> Result<Vec<BenchRunRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            session_id,
            COUNT(*)::bigint AS "total_decisions!",
            COUNT(*) FILTER (WHERE decision = 'allow')::bigint AS "allowed!",
            COUNT(*) FILTER (WHERE decision = 'deny')::bigint AS "denied!",
            GREATEST(
                EXTRACT(EPOCH FROM (MAX(created_at) - MIN(created_at))),
                0.001
            )::float8 AS "duration_seconds!",
            MIN(created_at) AS "first_at!"
        FROM governance_decisions
        WHERE session_id LIKE 'bench-%'
          OR session_id LIKE '%-govern'
          OR session_id LIKE '%-track'
        GROUP BY session_id
        HAVING COUNT(*) >= 10
        ORDER BY MIN(created_at) DESC
        LIMIT 10"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| BenchRunRow {
            session_id: r.session_id,
            total_decisions: r.total_decisions,
            allowed: r.allowed,
            denied: r.denied,
            duration_seconds: r.duration_seconds,
            first_at: r.first_at,
        })
        .collect())
}

async fn fetch_hourly_throughput(pool: &PgPool) -> Result<Vec<HourThroughputRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"SELECT
            date_trunc('hour', created_at) AS "bucket!",
            COUNT(*)::bigint AS "decisions!"
        FROM governance_decisions
        WHERE created_at >= NOW() - INTERVAL '24 hours'
        GROUP BY date_trunc('hour', created_at)
        ORDER BY date_trunc('hour', created_at) ASC"#,
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| HourThroughputRow {
            bucket: r.bucket,
            decisions: r.decisions,
        })
        .collect())
}
