mod analysis;
mod archetype;
mod data_loading;
mod report;
mod trends;

use std::sync::Arc;

use crate::admin::repositories::{daily_summaries, profile_reports, session_analyses};
use crate::admin::templates::AdminTemplateEngine;
use crate::admin::tier_enforcement::{check_limit, TierEnforcementCache};
use crate::admin::tier_limits::{Feature, LimitCheck};
use crate::admin::types::{MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;
use systemprompt::ai::AiService;

const PROFILE_PERIOD_DAYS: i32 = 30;

pub(crate) async fn profile_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let user_id = &user_ctx.user_id;

    let (
        daily_summaries_data,
        global_averages,
        user_metrics,
        gamification,
        recent_analyses,
        stored_report,
    ) = tokio::join!(
        daily_summaries::fetch_recent_daily_summaries(pool.as_ref(), user_id.as_str(), 30),
        daily_summaries::fetch_global_averages(pool.as_ref()),
        profile_reports::fetch_user_aggregate_metrics(
            pool.as_ref(),
            user_id.as_str(),
            PROFILE_PERIOD_DAYS
        ),
        crate::admin::gamification::queries::find_user_gamification(&pool, user_id.as_str()),
        session_analyses::fetch_recent_analyses(&pool, user_id, 50),
        profile_reports::fetch_profile_report(pool.as_ref(), user_id.as_str()),
    );

    let hooks_count =
        crate::admin::repositories::user_hooks::list_user_hooks(&pool, &user_ctx.user_id)
            .await
            .map_or(0, |h| h.len());

    let entity_counts = json!({
        "plugins": mkt_ctx.total_plugins,
        "skills": mkt_ctx.total_skills,
        "agents": mkt_ctx.agents_count,
        "mcp_servers": mkt_ctx.mcp_count,
        "hooks": hooks_count,
    });
    let archetype_result = archetype::classify_archetype(&user_metrics, &global_averages);
    let (strengths, weaknesses) =
        analysis::compute_strengths_weaknesses(&user_metrics, &global_averages);
    let trend_data = trends::build_trend_data(&daily_summaries_data, &global_averages);
    let comparison_grid = data_loading::build_comparison_grid(&user_metrics, &global_averages);
    let category_breakdown = data_loading::build_category_breakdown(&recent_analyses);

    let strengths_json =
        serde_json::to_value(&strengths).unwrap_or(serde_json::Value::Array(vec![]));
    let weaknesses_json =
        serde_json::to_value(&weaknesses).unwrap_or(serde_json::Value::Array(vec![]));
    let metrics_json = serde_json::to_value(&user_metrics)
        .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
    let gamification_profile = gamification.ok().flatten();
    let has_gamification = gamification_profile.is_some();

    let data = json!({
        "page": "profile",
        "title": "Profile & Insights",
        "archetype": {
            "id": archetype_result.id,
            "name": archetype_result.name,
            "description": archetype_result.description,
            "confidence": archetype_result.confidence,
        },
        "strengths": strengths_json,
        "has_strengths": !strengths.is_empty(),
        "weaknesses": weaknesses_json,
        "has_weaknesses": !weaknesses.is_empty(),
        "metrics": metrics_json,
        "has_metrics": user_metrics.total_days > 0,
        "comparison": comparison_grid,
        "trends": trend_data,
        "category_breakdown": category_breakdown,
        "has_category_breakdown": !category_breakdown.is_empty(),
        "entity_counts": entity_counts,
        "has_gamification": has_gamification,
        "gamification": data_loading::build_gamification_data(gamification_profile.as_ref()),
        "ai_report": data_loading::build_ai_report_data(stored_report.as_ref()),
        "has_ai_report": stored_report.as_ref().is_some_and(|r| r.ai_narrative.is_some()),
    });

    super::render_page(&engine, "profile", &data, &user_ctx, &mkt_ctx)
}

pub(crate) async fn handle_generate_profile_report(
    Extension(user_ctx): Extension<UserContext>,
    Extension(ai_service): Extension<Option<Arc<AiService>>>,
    Extension(tier_cache): Extension<TierEnforcementCache>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    let check = check_limit(
        &tier_cache,
        &pool,
        &user_ctx.user_id,
        LimitCheck::FeatureAccess(Feature::AiDailySummaries),
    )
    .await;
    if !check.allowed {
        return (
            StatusCode::FORBIDDEN,
            Json(json!({"error": "feature_denied", "message": check.reason})),
        )
            .into_response();
    }

    let Some(ref ai) = ai_service else {
        return (StatusCode::SERVICE_UNAVAILABLE, "AI service not available").into_response();
    };

    let user_id = user_ctx.user_id.as_str();
    let (global, user_metrics) = tokio::join!(
        daily_summaries::fetch_global_averages(pool.as_ref()),
        profile_reports::fetch_user_aggregate_metrics(pool.as_ref(), user_id, PROFILE_PERIOD_DAYS),
    );

    let archetype_result = archetype::classify_archetype(&user_metrics, &global);
    let (strengths, weaknesses) = analysis::compute_strengths_weaknesses(&user_metrics, &global);
    let context = report::build_ai_context(
        &user_metrics,
        &global,
        &archetype_result,
        &strengths,
        &weaknesses,
    );

    let ai_result = report::generate_ai_profile(ai, user_id, &context).await;

    let Some(ai_report) = ai_result else {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": "ai_failed", "message": "Failed to generate AI analysis. Please try again."})),
        )
            .into_response();
    };

    let input = profile_reports::ProfileReportInput {
        archetype: archetype_result.id,
        archetype_description: archetype_result.description,
        archetype_confidence: i16::from(archetype_result.confidence),
        strengths: serde_json::to_value(&strengths).unwrap_or(serde_json::Value::Array(vec![])),
        weaknesses: serde_json::to_value(&weaknesses).unwrap_or(serde_json::Value::Array(vec![])),
        ai_narrative: Some(ai_report.narrative),
        ai_style_analysis: Some(ai_report.style_analysis),
        ai_comparison: Some(ai_report.comparison),
        ai_patterns: Some(ai_report.patterns),
        ai_improvements: Some(ai_report.improvements),
        ai_tips: Some(ai_report.tips),
        metrics_snapshot: serde_json::to_value(&user_metrics)
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new())),
        period_days: PROFILE_PERIOD_DAYS,
    };

    match profile_reports::upsert_profile_report(pool.as_ref(), user_id, &input).await {
        Ok(()) => Json(json!({"status": "ok"})).into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to persist profile report");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "save_failed"})),
            )
                .into_response()
        }
    }
}
