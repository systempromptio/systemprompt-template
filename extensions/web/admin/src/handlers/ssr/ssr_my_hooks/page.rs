use std::sync::Arc;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{
    HookEventType, HooksQuery, MarketplaceContext, UserContext, MY_HOOKS_EVENT_TYPES,
};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use crate::handlers::ssr::types::{EventBreakdownView, HooksStats, MyHooksPageData, NamedEntity};

use super::analytics::{
    compute_avg_session_quality, fetch_hook_analytics, fetch_hooks_and_plugins,
};
use super::views::{build_event_breakdown_views, build_hook_views};

fn hook_event_type_names() -> Vec<&'static str> {
    MY_HOOKS_EVENT_TYPES
        .iter()
        .map(HookEventType::as_str)
        .collect::<Vec<_>>()
}

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
        tracing::error!(error = %e, "Failed to ensure default hooks");
    }

    let (hooks, user_plugins) = fetch_hooks_and_plugins(&pool, &user_ctx.user_id).await;

    let range = match query.range.as_str() {
        "24h" | "7d" | "14d" => query.range.as_str(),
        _ => "7d",
    };

    let (event_breakdown, timeseries, summary, hook_quality) =
        fetch_hook_analytics(&pool, &user_ctx.user_id, range).await;

    let chart = serde_json::to_value(crate::handlers::ssr::charts::compute_hooks_chart_data(
        &timeseries,
        range,
    ))
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize hooks chart data");
        serde_json::Value::Null
    });
    let avg_session_quality = compute_avg_session_quality(&hook_quality);
    let quality_map: std::collections::HashMap<&str, _> = hook_quality
        .iter()
        .map(|q| (q.event_type.as_str(), q))
        .collect();
    let event_breakdown_views = build_event_breakdown_views(event_breakdown, &quality_map);

    let plugin_name_map: std::collections::HashMap<String, String> = user_plugins
        .iter()
        .map(|p| (p.plugin_id.clone(), p.name.clone()))
        .collect();

    let total_count = hooks.len();
    let enabled_count = hooks.iter().filter(|h| h.enabled).count();

    let data = build_hooks_page_data(HooksPageInput {
        hooks,
        user_plugins,
        plugin_name_map,
        event_breakdown_views,
        summary: &summary,
        avg_session_quality,
        chart,
        range,
        total_count,
        enabled_count,
    });

    crate::handlers::ssr::render_typed_page(&engine, "my-hooks", &data, &user_ctx, &mkt_ctx)
}

struct HooksPageInput<'a> {
    hooks: Vec<crate::types::UserHook>,
    user_plugins: Vec<crate::types::UserPlugin>,
    plugin_name_map: std::collections::HashMap<String, String>,
    event_breakdown_views: Vec<EventBreakdownView>,
    summary: &'a crate::types::HookSummaryStats,
    avg_session_quality: f64,
    chart: serde_json::Value,
    range: &'a str,
    total_count: usize,
    enabled_count: usize,
}

fn build_hooks_page_data(input: HooksPageInput<'_>) -> MyHooksPageData {
    let hooks_views = build_hook_views(input.hooks, &input.plugin_name_map);
    let plugins: Vec<NamedEntity> = input
        .user_plugins
        .into_iter()
        .map(|p| NamedEntity {
            id: p.plugin_id,
            name: p.name,
        })
        .collect();

    MyHooksPageData {
        page: "my-hooks",
        title: "My Hooks",
        hooks: hooks_views,
        plugins,
        stats: HooksStats {
            total_count: input.total_count,
            enabled_count: input.enabled_count,
            total_events: input.summary.total_events,
            total_errors: input.summary.total_errors,
            content_input_bytes: input.summary.content_input_bytes,
            content_output_bytes: input.summary.content_output_bytes,
            avg_session_quality: format!("{:.1}", input.avg_session_quality),
        },
        event_breakdown: input.event_breakdown_views,
        chart: input.chart,
        range: input.range.to_string(),
        hook_event_types: hook_event_type_names(),
    }
}
