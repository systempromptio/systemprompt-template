mod data_injectors;

use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{DashboardQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use super::ssr_dashboard_helpers::{build_dashboard_template, DashboardCounts};
use data_injectors::{inject_governance_data, inject_mcp_access_and_costs};

struct ChartParams<'a> {
    interval: &'a str,
    bucket: &'a str,
    range_key: &'a str,
    traffic_range: &'a str,
    content_range: &'a str,
}

async fn fetch_dashboard_data(
    pool: &PgPool,
    services_path: Option<&std::path::PathBuf>,
    user_roles: &[String],
    chart: &ChartParams<'_>,
) -> (crate::types::DashboardData, DashboardCounts) {
    let (dash_result, users_result, plugins_result) = tokio::join!(
        repositories::get_dashboard_data(
            pool,
            chart.interval,
            chart.bucket,
            chart.traffic_range,
            chart.content_range
        ),
        repositories::list_users(pool),
        async {
            match services_path.map(|p| repositories::list_plugins_for_roles(p, user_roles)) {
                Some(Ok(plugins)) => Ok(plugins),
                Some(Err(e)) => Err(e),
                None => Ok(vec![]),
            }
        },
    );

    let dash = dash_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch dashboard data");
        crate::types::DashboardData {
            timeline: vec![],
            top_users: vec![],
            popular_skills: vec![],
            hourly_activity: vec![],
            stats: crate::types::ActivityStats {
                events_today: 0,
                events_this_week: 0,
                total_edits: 0,
                mcp_tool_calls: 0,
                mcp_errors: 0,
                total_logins: 0,
            },
            usage_timeseries: vec![],
            active_users_24h: 0,
            tool_success_rates: vec![],
            traffic: None,
            recent_mcp_errors: vec![],
            top_pages_today: vec![],
            realtime_pulse: None,
            content_performance: vec![],
        }
    });
    let users = users_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list users for dashboard");
        vec![]
    });
    let plugins = plugins_result.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to list plugins for dashboard");
        vec![]
    });

    let counts = DashboardCounts {
        total_users: users.len(),
        total_plugins: plugins.len(),
        total_skills: plugins.iter().map(|p| p.skills.len()).sum(),
        agents_count: plugins.iter().map(|p| p.agents.len()).sum(),
        mcp_count: plugins.iter().map(|p| p.mcp_servers.len()).sum(),
    };

    (dash, counts)
}

pub async fn dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DashboardQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return axum::response::Redirect::to("/admin/my/plugins").into_response();
    }

    let services_path = super::get_services_path()
        .map_err(|_| {
            tracing::warn!("Failed to get services path for dashboard");
        })
        .ok();

    let chart_params = resolve_chart_params(&query);
    let (dash, counts) = fetch_dashboard_data(
        &pool,
        services_path.as_ref(),
        &user_ctx.roles,
        &chart_params,
    )
    .await;

    let tab = query.tab.as_str();
    let mut data = build_dashboard_data_json(&dash, &counts, &chart_params, tab);

    inject_page_stats(&mut data);
    inject_governance_data(&pool, &mut data).await;
    inject_mcp_access_and_costs(&pool, &mut data).await;
    inject_report_if_needed(tab, &pool, &mut data).await;

    super::render_page(&engine, "dashboard", &data, &user_ctx, &mkt_ctx)
}

fn resolve_chart_params(query: &DashboardQuery) -> ChartParams<'_> {
    let (interval, bucket, range_key) = match query.range.as_str() {
        "24h" => ("24 hours", "1 hour", "24h"),
        "14d" => ("14 days", "1 day", "14d"),
        _ => ("7 days", "4 hours", "7d"),
    };

    let traffic_range = match query.traffic_range.as_str() {
        "7d" => "7d",
        "30d" => "30d",
        _ => "today",
    };

    let content_range = match query.content_range.as_str() {
        "1h" => "1h",
        "24h" => "24h",
        "yesterday" => "yesterday",
        "30d" => "30d",
        _ => "7d",
    };

    ChartParams {
        interval,
        bucket,
        range_key,
        traffic_range,
        content_range,
    }
}

fn build_dashboard_data_json(
    dash: &crate::types::DashboardData,
    counts: &DashboardCounts,
    chart: &ChartParams<'_>,
    tab: &str,
) -> serde_json::Value {
    serde_json::to_value(build_dashboard_template(
        dash,
        counts,
        chart.range_key,
        chart.traffic_range,
        chart.content_range,
        tab,
    ))
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize dashboard template data");
        serde_json::Value::Null
    })
}

fn inject_page_stats(data: &mut serde_json::Value) {
    if let Some(obj) = data.as_object_mut() {
        let users_val = obj
            .get("total_users")
            .and_then(serde_json::Value::as_u64)
            .unwrap_or(0);
        let active_val = obj
            .get("active_users_24h")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let events_val = obj
            .get("stats")
            .and_then(|s| s.get("events_today"))
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        obj.insert(
            "page_stats".to_string(),
            json!([
                {"value": users_val, "label": "Users", "key": "total_users"},
                {"value": active_val, "label": "Active Today", "key": "active_users_24h"},
                {"value": events_val, "label": "Events Today", "key": "events_today"},
            ]),
        );
    }
}

async fn inject_report_if_needed(tab: &str, pool: &PgPool, data: &mut serde_json::Value) {
    if tab == "report" {
        if let Ok(Some(report_row)) =
            repositories::admin_traffic_reports::fetch_latest_report(pool).await
        {
            let report = super::ssr_dashboard_report::build_dashboard_report(&report_row);
            if let Some(obj) = data.as_object_mut() {
                obj.insert("has_dashboard_report".to_string(), json!(true));
                obj.insert("dashboard_report".to_string(), report);
            }
        }
    }
}
