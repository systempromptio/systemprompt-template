use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::types::{DashboardQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::{IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

struct DashboardCounts {
    total_users: usize,
    total_plugins: usize,
    total_skills: usize,
    agents_count: usize,
    mcp_count: usize,
}

async fn fetch_dashboard_data(
    pool: &Arc<PgPool>,
    services_path: Option<&std::path::PathBuf>,
    user_roles: &[String],
    chart_interval: &str,
) -> (crate::admin::types::DashboardData, DashboardCounts) {
    let (dash_result, users_result, plugins_result) = tokio::join!(
        repositories::get_dashboard_data(pool, None, chart_interval),
        repositories::list_users(pool, None),
        async {
            match services_path.map(|p| repositories::list_plugins_for_roles(p, user_roles)) {
                Some(Ok(plugins)) => Ok(plugins),
                Some(Err(e)) => Err(e),
                None => Ok(vec![]),
            }
        },
    );

    let dash = dash_result.unwrap_or_else(|_| crate::admin::types::DashboardData {
        timeline: vec![],
        top_users: vec![],
        popular_skills: vec![],
        hourly_activity: vec![],
        department_activity: vec![],
        stats: crate::admin::types::ActivityStats {
            events_today: 0,
            events_this_week: 0,
            total_sessions: 0,
            error_count: 0,
            tool_uses: 0,
            prompts: 0,
            subagents_spawned: 0,
            total_input_tokens: 0,
            total_output_tokens: 0,
            total_cost_usd: 0.0,
            failure_count: 0,
        },
        model_usage: vec![],
        event_breakdown: vec![],
        usage_timeseries: vec![],
        active_users_24h: 0,
        avg_session_duration_secs: 0,
        project_activity: vec![],
        tool_success_rates: vec![],
        mcp_access_events: vec![],
        mcp_access_stats: vec![],
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

fn build_event_chart(items: &[crate::admin::types::EventTypeBreakdown]) -> Vec<serde_json::Value> {
    super::compute_bar_chart(
        items,
        |e| e.count,
        |e, pct| {
            let label = match e.event_type.as_str() {
                "claude_code_PostToolUse" => "Tool Use",
                "claude_code_PostToolUseFailure" => "Tool Failure",
                "claude_code_SessionStart" => "Session Start",
                "claude_code_SessionEnd" => "Session End",
                "claude_code_Stop" => "Turn Complete",
                "claude_code_SubagentStart" => "Subagent Start",
                "claude_code_SubagentStop" => "Subagent Stop",
                "claude_code_UserPromptSubmit" => "User Prompt",
                other => other.strip_prefix("claude_code_").unwrap_or(other),
            };
            let is_error = e.event_type.contains("Failure") || e.event_type.contains("error");
            json!({ "event_type": e.event_type, "label": label, "count": e.count, "pct": pct, "is_error": is_error })
        },
    )
}

#[allow(clippy::too_many_lines)]
pub(crate) async fn dashboard_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DashboardQuery>,
) -> Response {
    if !user_ctx.is_admin {
        return axum::response::Redirect::temporary(&format!(
            "/admin/user?id={}",
            user_ctx.user_id
        ))
        .into_response();
    }

    let services_path = super::get_services_path()
        .map_err(|_| {
            tracing::warn!("Failed to get services path for dashboard");
        })
        .ok();
    let user_roles = user_ctx.roles.clone();

    let (interval_sql, range_key) = match query.range.as_str() {
        "24h" => ("24 hours", "24h"),
        "14d" => ("14 days", "14d"),
        _ => ("7 days", "7d"),
    };

    let (dash, counts) =
        fetch_dashboard_data(&pool, services_path.as_ref(), &user_roles, interval_sql).await;

    let hourly_with_pct = super::compute_hourly_chart(&dash.hourly_activity);
    let skills_with_pct = super::compute_bar_chart(
        &dash.popular_skills,
        |s| s.count,
        |s, pct| json!({ "tool_name": s.tool_name, "count": s.count, "pct": pct }),
    );
    let dept_with_pct = super::compute_bar_chart(
        &dash.department_activity,
        |d| d.count,
        |d, pct| json!({ "department": d.department, "count": d.count, "pct": pct }),
    );
    let events_with_pct = build_event_chart(&dash.event_breakdown);
    let models_with_pct = super::compute_bar_chart(
        &dash.model_usage,
        |m| m.count,
        |m, pct| {
            let short = m
                .model
                .replace("claude-", "")
                .trim_end_matches(|c: char| c == '-' || c.is_ascii_digit())
                .to_string();
            json!({ "model": m.model, "short": short, "count": m.count, "pct": pct })
        },
    );

    let project_with_pct = super::compute_bar_chart(
        &dash.project_activity,
        |p| p.event_count,
        |p, pct| json!({ "project_path": p.project_path, "project_name": p.project_name, "event_count": p.event_count, "session_count": p.session_count, "pct": pct }),
    );
    let tools_with_pct = super::compute_bar_chart(
        &dash.tool_success_rates,
        |t| t.total,
        |t, _pct| {
            let color_class = if t.success_pct < 90.0 {
                "progress-red"
            } else if t.success_pct < 98.0 {
                "progress-amber"
            } else {
                "progress-green"
            };
            json!({ "tool_name": t.tool_name, "total": t.total, "successes": t.successes, "failures": t.failures, "success_pct": format!("{:.1}", t.success_pct), "pct": t.success_pct, "color_class": color_class })
        },
    );

    let mcp_events_json: Vec<serde_json::Value> = dash
        .mcp_access_events
        .iter()
        .map(|e| {
            json!({
                "display_name": e.display_name,
                "server_name": e.entity_name,
                "action": e.action,
                "description": e.description,
                "created_at": e.created_at,
                "is_rejected": e.action == "rejected",
            })
        })
        .collect();

    let total_rejected: i64 = dash.mcp_access_stats.iter().map(|s| s.rejected).sum();

    let chart_data = super::compute_area_chart_data(&dash.usage_timeseries, range_key);

    let total_tokens = dash.stats.total_input_tokens + dash.stats.total_output_tokens;
    #[allow(clippy::cast_precision_loss)]
    let total_tokens_display = if total_tokens >= 1_000_000 {
        format!("{:.1}M", total_tokens as f64 / 1_000_000.0)
    } else if total_tokens >= 1_000 {
        format!("{:.1}K", total_tokens as f64 / 1_000.0)
    } else {
        total_tokens.to_string()
    };

    let error_rate_pct = if dash.stats.events_this_week > 0 {
        dash.stats.error_count.saturating_mul(100) / dash.stats.events_this_week
    } else {
        0
    };

    let avg_duration_secs = dash.avg_session_duration_secs;
    let avg_session_display = if avg_duration_secs >= 60 {
        let m = avg_duration_secs / 60;
        let s = avg_duration_secs % 60;
        format!("{m}m {s}s")
    } else {
        format!("{avg_duration_secs}s")
    };

    let data = json!({
        "page": "dashboard",
        "title": "Dashboard",
        "stats": dash.stats,
        "timeline": dash.timeline,
        "top_users": dash.top_users,
        "popular_skills": skills_with_pct,
        "hourly_activity": hourly_with_pct,
        "department_activity": dept_with_pct,
        "event_breakdown": events_with_pct,
        "model_usage": models_with_pct,
        "total_users": counts.total_users,
        "total_plugins": counts.total_plugins,
        "total_skills": counts.total_skills,
        "agents_count": counts.agents_count,
        "mcp_count": counts.mcp_count,
        "chart": chart_data,
        "range": range_key,
        "range_24h": range_key == "24h",
        "range_7d": range_key == "7d",
        "range_14d": range_key == "14d",
        "active_users_24h": dash.active_users_24h,
        "error_rate_pct": error_rate_pct,
        "avg_session_display": avg_session_display,
        "project_activity": project_with_pct,
        "tool_success_rates": tools_with_pct,
        "total_tokens_display": total_tokens_display,
        "mcp_access_events": mcp_events_json,
        "mcp_access_stats": dash.mcp_access_stats,
        "total_rejected": total_rejected,
    });
    super::render_page(&engine, "dashboard", &data, &user_ctx, &mkt_ctx)
}
