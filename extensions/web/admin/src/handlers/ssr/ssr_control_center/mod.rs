mod api_handlers;
pub mod enrichment;
mod helpers;
mod session_groups;
mod turns;
pub mod types;

use std::sync::Arc;

use crate::repositories::{daily_summaries, session_analyses};
use crate::templates::AdminTemplateEngine;
use crate::types::{DashboardQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use serde_json::json;
use sqlx::PgPool;

use types::EntityCounts;

pub use session_groups::build_session_groups_with_status;

pub use api_handlers::handle_analyse_session;
pub use api_handlers::handle_batch_update_session_status;
pub use api_handlers::handle_generate_report;
pub use api_handlers::handle_rate_session;
pub use api_handlers::handle_rate_skill;
pub use api_handlers::handle_update_session_status;

pub async fn control_center_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<DashboardQuery>,
) -> Response {
    let user_id = &user_ctx.user_id;
    let status_filter = query.status.as_str();

    let (
        recent_sessions,
        today_stats,
        outcome_stats,
        top_tools,
        analytics,
        gamification,
        apm_live,
        hourly_breakdown,
        perf_summary,
    ) = helpers::fetch_control_center_data(&pool, user_id, status_filter).await;
    let (health_metrics, recent_analyses, daily_summaries_data, global_averages) = tokio::join!(
        session_analyses::fetch_health_metrics(&pool, user_id),
        session_analyses::fetch_recent_analyses(&pool, user_id, 20),
        daily_summaries::fetch_recent_daily_summaries(pool.as_ref(), user_id.as_str(), 15),
        daily_summaries::fetch_global_averages(pool.as_ref()),
    );

    let services_path = super::get_services_path()
        .unwrap_or_else(|_| std::path::PathBuf::from("services"));
    let mut entity_counts = fetch_entity_counts(pool.as_ref(), user_id.as_str()).await;
    let (org_plugins, org_skills, org_agents, org_mcp) =
        crate::repositories::count_org_entity_additions(pool.as_ref(), &services_path).await;
    entity_counts.plugins += org_plugins;
    entity_counts.skills += org_skills;
    entity_counts.agents += org_agents;
    entity_counts.mcp_servers += org_mcp;
    let current_streak = gamification.as_ref().map_or(0, |g| g.current_streak);

    let mut session_groups = helpers::build_session_groups(&pool, user_id, &recent_sessions).await;

    let tools_with_pct = super::charts::compute_bar_chart(
        &top_tools,
        |t| t.count,
        |t, pct| json!({ "tool_name": t.tool_name, "count": t.count, "pct": pct }),
    );

    enrichment::enrich_session_groups(
        &mut session_groups,
        &analytics.session_ratings,
        &analytics.entity_links,
    );
    let (skills_usage, agents_usage, mcp_usage) =
        helpers::partition_entity_usage(&analytics.entity_usage);

    let data = helpers::build_template_data(&helpers::BuildTemplateDataParams {
        user_ctx: &user_ctx,
        mkt_ctx: &mkt_ctx,
        status_filter,
        today_stats: &today_stats,
        outcome_stats: &outcome_stats,
        session_groups: &session_groups,
        tools_with_pct: &tools_with_pct,
        skill_effectiveness: &analytics.skill_effectiveness,
        skills_usage: &skills_usage,
        agents_usage: &agents_usage,
        mcp_usage: &mcp_usage,
        skill_ratings: &analytics.skill_ratings,
        gamification: gamification.as_ref(),
        health_metrics: &health_metrics,
        recent_analyses: &recent_analyses,
        unused_skills: &analytics.unused_skills,
        today_summary: &analytics.today_summary,
        apm_live: &apm_live,
        hourly_breakdown: &hourly_breakdown,
        perf_summary: &perf_summary,
        daily_summaries: &daily_summaries_data,
        global_averages: &global_averages,
        entity_counts: &entity_counts,
        current_streak,
    });

    super::render_page(&engine, "control-center", &data, &user_ctx, &mkt_ctx)
}

async fn fetch_entity_counts(pool: &PgPool, user_id: &str) -> EntityCounts {
    struct ControlCenterCounts {
        plugins: Option<i64>,
        skills: Option<i64>,
        agents: Option<i64>,
        mcp_servers: Option<i64>,
        hooks: Option<i64>,
    }

    let counts = sqlx::query_as!(
        ControlCenterCounts,
        r"SELECT
            (SELECT COUNT(*) FROM user_plugins WHERE user_id = $1) AS plugins,
            (SELECT COUNT(*) FROM user_skills WHERE user_id = $1) AS skills,
            (SELECT COUNT(*) FROM user_agents WHERE user_id = $1) AS agents,
            (SELECT COUNT(*) FROM user_mcp_servers WHERE user_id = $1) AS mcp_servers,
            (SELECT COUNT(*) FROM user_hooks WHERE user_id = $1) AS hooks",
        user_id,
    )
    .fetch_one(pool)
    .await
    .unwrap_or(ControlCenterCounts {
        plugins: Some(0),
        skills: Some(0),
        agents: Some(0),
        mcp_servers: Some(0),
        hooks: Some(0),
    });

    EntityCounts {
        plugins: counts.plugins.unwrap_or(0),
        skills: counts.skills.unwrap_or(0),
        agents: counts.agents.unwrap_or(0),
        mcp_servers: counts.mcp_servers.unwrap_or(0),
        hooks: counts.hooks.unwrap_or(0),
    }
}
