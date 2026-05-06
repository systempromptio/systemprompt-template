mod data;

use std::sync::Arc;

use crate::activity;
use crate::templates::AdminTemplateEngine;
use crate::types::{EventsQuery, MarketplaceContext, UserContext};
use axum::{
    extract::{Extension, Query, State},
    response::Response,
};
use sqlx::PgPool;

use data::{build_activity_template, compute_activity_stats, BuildActivityTemplateParams};

pub async fn my_activity_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<EventsQuery>,
) -> Response {
    let user_id = &user_ctx.user_id;

    let (activities, total, category_summary, gamification) =
        fetch_activity_data(&pool, user_id, &query).await;

    let stats = compute_activity_stats(&category_summary, gamification.as_ref(), total);

    let data = build_activity_template(&BuildActivityTemplateParams {
        activities: &activities,
        total,
        query: &query,
        category_summary: &category_summary,
        gamification: gamification.as_ref(),
        stats: &stats,
    });
    let value = serialize_activity_page(&data, &stats, total);
    super::render_page(&engine, "my-activity", &value, &user_ctx, &mkt_ctx)
}

async fn fetch_activity_data(
    pool: &PgPool,
    user_id: &systemprompt::identifiers::UserId,
    query: &EventsQuery,
) -> (
    Vec<activity::types::ActivityTimelineEvent>,
    i64,
    Vec<activity::types::ActivityCategorySummary>,
    Option<crate::types::UserGamificationProfile>,
) {
    let (activities, total) = activity::queries::search_user_entity_activity(
        pool,
        user_id.as_str(),
        query.search.as_deref(),
        query.limit,
        query.offset,
    )
    .await
    .unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to search user entity activity");
        (vec![], 0)
    });

    let category_summary = activity::queries::list_user_activity_summary(pool, user_id.as_str())
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to get user activity summary");
            vec![]
        });

    let gamification = crate::gamification::queries::find_user_gamification(pool, user_id.as_str())
        .await
        .map_err(|e| {
            tracing::warn!(error = %e, "Failed to fetch user gamification");
        })
        .ok()
        .flatten();

    (activities, total, category_summary, gamification)
}

fn serialize_activity_page(
    data: &crate::handlers::ssr::types::MyActivityPageData,
    stats: &data::ActivityStats,
    total: i64,
) -> serde_json::Value {
    let mut value = serde_json::to_value(data).unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to serialize my-activity page data");
        serde_json::Value::Null
    });
    if let Some(obj) = value.as_object_mut() {
        obj.insert(
            "page_stats".to_string(),
            serde_json::json!([
                {"value": stats.total_sessions, "label": "Sessions"},
                {"value": stats.total_edits, "label": "Edits"},
                {"value": total, "label": "Activities"},
            ]),
        );
    }
    value
}
