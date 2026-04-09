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

struct ChartParams<'a> {
    interval: &'a str,
    bucket: &'a str,
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

    let dash = dash_result.unwrap_or_else(|_| crate::types::DashboardData {
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

async fn inject_mcp_access_and_costs(pool: &PgPool, data: &mut serde_json::Value) {
    let (mcp_access, token_usage) = tokio::join!(
        repositories::dashboard_queries::fetch_mcp_access_events(pool),
        repositories::dashboard_queries::fetch_token_usage_by_user(pool),
    );

    let mcp_events = mcp_access.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch MCP access events for dashboard");
        vec![]
    });
    let mcp_json: Vec<serde_json::Value> = mcp_events
        .iter()
        .map(|r| {
            json!({
                "server_name": r.server_name,
                "action": r.action,
                "is_granted": r.action == "granted",
                "description": r.description,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    let tokens = token_usage.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch token usage for dashboard");
        vec![]
    });
    let max_tokens: i64 = tokens
        .iter()
        .map(|t| t.input_tokens + t.output_tokens)
        .max()
        .unwrap_or(1)
        .max(1);
    let tokens_json: Vec<serde_json::Value> = tokens
        .iter()
        .map(|r| {
            let total = r.input_tokens + r.output_tokens;
            let pct = total.saturating_mul(100) / max_tokens;
            json!({
                "label": r.label,
                "input_tokens": r.input_tokens,
                "output_tokens": r.output_tokens,
                "total_tokens": total,
                "event_count": r.event_count,
                "pct": pct,
            })
        })
        .collect();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("mcp_access_events".to_string(), json!(mcp_json));
        obj.insert(
            "has_mcp_access_events".to_string(),
            json!(!mcp_json.is_empty()),
        );
        obj.insert("token_usage".to_string(), json!(tokens_json));
        obj.insert(
            "has_token_usage".to_string(),
            json!(!tokens_json.is_empty()),
        );
    }
}

async fn inject_governance_data(pool: &PgPool, data: &mut serde_json::Value) {
    let (gov_events, gov_counts) = tokio::join!(
        repositories::governance::fetch_governance_events(pool),
        repositories::governance::fetch_governance_counts(pool),
    );
    let gov_events = gov_events.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance events for dashboard");
        vec![]
    });
    let counts = gov_counts.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch governance counts for dashboard");
        repositories::governance::GovernanceCounts {
            total: 0,
            allowed: 0,
            denied: 0,
            secret_breaches: 0,
        }
    });
    let gov_json: Vec<serde_json::Value> = gov_events
        .iter()
        .map(|r| {
            json!({
                "user_id": r.user_id,
                "tool_name": r.tool_name,
                "agent_id": r.agent_id,
                "decision": r.decision,
                "is_denied": r.decision == "deny",
                "reason": r.reason,
                "created_at": r.created_at.with_timezone(&chrono::Local).format("%Y-%m-%d %H:%M:%S").to_string(),
            })
        })
        .collect();

    if let Some(obj) = data.as_object_mut() {
        obj.insert("governance_total".to_string(), json!(counts.total));
        obj.insert("governance_allowed".to_string(), json!(counts.allowed));
        obj.insert("governance_denied".to_string(), json!(counts.denied));
        obj.insert(
            "governance_secret_breaches".to_string(),
            json!(counts.secret_breaches),
        );
        obj.insert("governance_events".to_string(), json!(gov_json));
        obj.insert(
            "has_governance_events".to_string(),
            json!(!gov_json.is_empty()),
        );
    }
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
    let user_roles = user_ctx.roles.clone();

    let (interval_sql, bucket_sql, range_key) = match query.range.as_str() {
        "24h" => ("24 hours", "1 hour", "24h"),
        "14d" => ("14 days", "1 day", "14d"),
        _ => ("7 days", "4 hours", "7d"),
    };

    let traffic_range_key = match query.traffic_range.as_str() {
        "7d" => "7d",
        "30d" => "30d",
        _ => "today",
    };

    let content_range_key = match query.content_range.as_str() {
        "1h" => "1h",
        "24h" => "24h",
        "yesterday" => "yesterday",
        "30d" => "30d",
        _ => "7d",
    };

    let chart_params = ChartParams {
        interval: interval_sql,
        bucket: bucket_sql,
        traffic_range: traffic_range_key,
        content_range: content_range_key,
    };
    let (dash, counts) =
        fetch_dashboard_data(&pool, services_path.as_ref(), &user_roles, &chart_params).await;

    let tab = query.tab.as_str();
    let mut data = serde_json::to_value(build_dashboard_template(
        &dash,
        &counts,
        range_key,
        traffic_range_key,
        content_range_key,
        tab,
    ))
    .unwrap_or_else(|_| serde_json::Value::Null);

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

    inject_governance_data(&pool, &mut data).await;
    inject_mcp_access_and_costs(&pool, &mut data).await;

    if tab == "report" {
        if let Ok(Some(report_row)) =
            repositories::admin_traffic_reports::fetch_latest_report(&pool).await
        {
            let report = super::ssr_dashboard_report::build_dashboard_report(&report_row);
            if let Some(obj) = data.as_object_mut() {
                obj.insert("has_dashboard_report".to_string(), json!(true));
                obj.insert("dashboard_report".to_string(), report);
            }
        }
    }

    super::render_page(&engine, "dashboard", &data, &user_ctx, &mkt_ctx)
}
