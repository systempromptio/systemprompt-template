use std::sync::Arc;

use crate::repositories::{self, conversation_analytics};
use crate::templates::AdminTemplateEngine;
use crate::types::{HooksQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use super::types::{
    EventBreakdownView, HookCodeEntry, HookCodeHook, HookView, HooksStats, MyHooksPageData,
    NamedEntity,
};

const HOOK_EVENT_TYPES: &[&str] = &[
    "PreToolUse",
    "PostToolUse",
    "PostToolUseFailure",
    "PermissionRequest",
    "UserPromptSubmit",
    "Stop",
    "SubagentStop",
    "TaskCompleted",
    "SessionStart",
    "SessionEnd",
    "SubagentStart",
    "Notification",
    "TeammateIdle",
    "InstructionsLoaded",
];

pub async fn my_hooks_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<HooksQuery>,
) -> Response {
    if let Err(e) =
        repositories::user_hooks::ensure_default_hooks(&pool, &user_ctx.user_id, &mkt_ctx.site_url)
            .await
    {
        tracing::warn!(error = %e, "Failed to ensure default hooks");
    }

    let (hooks, user_plugins) = fetch_hooks_and_plugins(&pool, &user_ctx.user_id).await;

    let range = match query.range.as_str() {
        "24h" | "7d" | "14d" => query.range.as_str(),
        _ => "7d",
    };

    let (event_breakdown, timeseries, summary, hook_quality) =
        fetch_hook_analytics(&pool, &user_ctx.user_id, range).await;

    let chart = serde_json::to_value(super::charts::compute_hooks_chart_data(&timeseries, range))
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to serialize hooks chart data");
            serde_json::Value::Null
        });
    let avg_session_quality = compute_avg_session_quality(&hook_quality);
    let quality_map: std::collections::HashMap<&str, _> = hook_quality
        .iter()
        .map(|q| (q.event_type.as_str(), q))
        .collect();
    let event_breakdown_views = build_event_breakdown_views(&event_breakdown, &quality_map);

    let plugin_name_map: std::collections::HashMap<String, String> = user_plugins
        .iter()
        .map(|p| (p.plugin_id.clone(), p.name.clone()))
        .collect();

    let data = build_hooks_page_data(&HooksPageInput {
        hooks: &hooks,
        user_plugins: &user_plugins,
        plugin_name_map: &plugin_name_map,
        event_breakdown_views,
        summary: &summary,
        avg_session_quality,
        chart,
        range,
    });

    let value = serde_json::to_value(&data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize hooks page data");
        serde_json::Value::Object(serde_json::Map::new())
    });
    super::render_page(&engine, "my-hooks", &value, &user_ctx, &mkt_ctx)
}

async fn fetch_hooks_and_plugins(
    pool: &PgPool,
    user_id: &UserId,
) -> (Vec<crate::types::UserHook>, Vec<crate::types::UserPlugin>) {
    let hooks = repositories::user_hooks::list_user_hooks(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user hooks");
            vec![]
        });
    let user_plugins = repositories::list_user_plugins(pool, user_id)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to list user plugins");
            vec![]
        });
    (hooks, user_plugins)
}

async fn fetch_hook_analytics(
    pool: &PgPool,
    user_id: &UserId,
    range: &str,
) -> (
    Vec<crate::types::HookEventTypeStat>,
    Vec<crate::types::HookTimeSeriesBucket>,
    crate::types::HookSummaryStats,
    Vec<crate::types::conversation_analytics::HookSessionQuality>,
) {
    let (event_breakdown, timeseries, summary, hook_quality) = tokio::join!(
        repositories::user_hooks::get_hook_event_breakdown(pool, user_id),
        repositories::user_hooks::get_hook_timeseries(pool, user_id, range),
        repositories::user_hooks::get_hook_summary_stats(pool, user_id, range),
        async {
            conversation_analytics::fetch_hook_session_quality(pool, user_id)
                .await
                .unwrap_or_else(|e| {
                    tracing::warn!(error = %e, "Failed to fetch hook session quality");
                    vec![]
                })
        },
    );
    let event_breakdown = event_breakdown.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch hook event breakdown");
        vec![]
    });
    let timeseries = timeseries.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch hook timeseries");
        vec![]
    });
    (event_breakdown, timeseries, summary, hook_quality)
}

fn compute_avg_session_quality(
    hook_quality: &[crate::types::conversation_analytics::HookSessionQuality],
) -> f64 {
    if hook_quality.is_empty() {
        return 0.0;
    }
    let count = hook_quality.len();
    hook_quality.iter().map(|q| q.avg_quality).sum::<f64>()
        / f64::from(u32::try_from(count).unwrap_or(1))
}

struct HooksPageInput<'a> {
    hooks: &'a [crate::types::UserHook],
    user_plugins: &'a [crate::types::UserPlugin],
    plugin_name_map: &'a std::collections::HashMap<String, String>,
    event_breakdown_views: Vec<EventBreakdownView>,
    summary: &'a crate::types::HookSummaryStats,
    avg_session_quality: f64,
    chart: serde_json::Value,
    range: &'a str,
}

fn build_hooks_page_data(input: &HooksPageInput<'_>) -> MyHooksPageData {
    let hooks_views = build_hook_views(input.hooks, input.plugin_name_map);
    let plugins: Vec<NamedEntity> = input
        .user_plugins
        .iter()
        .map(|p| NamedEntity {
            id: p.plugin_id.clone(),
            name: p.name.clone(),
        })
        .collect();

    MyHooksPageData {
        page: "my-hooks",
        title: "My Hooks",
        hooks: hooks_views,
        plugins,
        stats: HooksStats {
            total_count: input.hooks.len(),
            enabled_count: input.hooks.iter().filter(|h| h.enabled).count(),
            total_events: input.summary.total_events,
            total_errors: input.summary.total_errors,
            content_input_bytes: input.summary.content_input_bytes,
            content_output_bytes: input.summary.content_output_bytes,
            avg_session_quality: format!("{:.1}", input.avg_session_quality),
        },
        event_breakdown: input.event_breakdown_views.clone(),
        chart: input.chart.clone(),
        range: input.range.to_string(),
        hook_event_types: HOOK_EVENT_TYPES.to_vec(),
    }
}

fn build_event_breakdown_views(
    event_breakdown: &[crate::types::HookEventTypeStat],
    quality_map: &std::collections::HashMap<
        &str,
        &crate::types::conversation_analytics::HookSessionQuality,
    >,
) -> Vec<EventBreakdownView> {
    let max_event_count = event_breakdown
        .iter()
        .map(|e| e.event_count)
        .max()
        .unwrap_or(1)
        .max(1);
    event_breakdown
        .iter()
        .map(|e| {
            let pct = e.event_count.saturating_mul(100) / max_event_count;
            let quality = quality_map.get(e.event_type.as_str());
            EventBreakdownView {
                event_type: e.event_type.clone(),
                event_count: e.event_count,
                error_count: e.error_count,
                content_input_bytes: e.content_input_bytes,
                content_output_bytes: e.content_output_bytes,
                pct,
                avg_quality: quality
                    .map_or_else(|| "0.0".to_string(), |q| format!("{:.1}", q.avg_quality)),
                quality_goal_pct: quality.map_or_else(
                    || "0.0".to_string(),
                    |q| format!("{:.0}", q.goal_achievement_pct),
                ),
                quality_sessions: quality.map_or(0, |q| q.session_count),
            }
        })
        .collect()
}

fn build_hook_views(
    hooks: &[crate::types::UserHook],
    plugin_name_map: &std::collections::HashMap<String, String>,
) -> Vec<HookView> {
    hooks
        .iter()
        .map(|h| {
            let hook_code_entry = if h.hook_type == "http" {
                HookCodeEntry {
                    matcher: h.matcher.clone(),
                    hooks: vec![HookCodeHook {
                        hook_type: "http".to_string(),
                        url: Some(h.url.clone()),
                        headers: Some(h.headers.clone()),
                        command: None,
                        is_async: None,
                        timeout: Some(h.timeout),
                    }],
                }
            } else {
                HookCodeEntry {
                    matcher: h.matcher.clone(),
                    hooks: vec![HookCodeHook {
                        hook_type: "command".to_string(),
                        url: None,
                        headers: None,
                        command: Some(h.command.clone()),
                        is_async: Some(h.is_async),
                        timeout: Some(h.timeout),
                    }],
                }
            };
            let hook_code = serde_json::to_string_pretty(&[&hook_code_entry]).unwrap_or_else(|e| {
                tracing::warn!(error = %e, "Failed to serialize hook code entry");
                String::new()
            });
            HookView {
                id: h.id.clone(),
                hook_name: h.hook_name.clone(),
                description: h.description.clone(),
                event_type: h.event_type.clone(),
                hook_type: h.hook_type.clone(),
                matcher: h.matcher.clone(),
                url: h.url.clone(),
                command: h.command.clone(),
                headers: h.headers.clone(),
                timeout: h.timeout,
                is_async: h.is_async,
                enabled: h.enabled,
                is_default: h.is_default,
                plugin_id: h.plugin_id.clone(),
                plugin_name: h
                    .plugin_id
                    .as_ref()
                    .and_then(|pid| plugin_name_map.get(pid))
                    .unwrap_or(&String::new())
                    .clone(),
                hook_code,
            }
        })
        .collect()
}
