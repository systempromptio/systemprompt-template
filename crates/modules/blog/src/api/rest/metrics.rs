use crate::repository::MetricsRepository;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use serde_json::json;
use systemprompt_core_system::AppContext;

/// GET /api/v1/blog/analytics/:id
/// Get analytics for a specific article
pub async fn get_article_analytics(
    State(ctx): State<AppContext>,
    Path(article_id): Path<String>,
) -> impl IntoResponse {
    let metrics_repo = MetricsRepository::new(ctx.db_pool().clone());

    match metrics_repo.get_metrics(&article_id).await {
        Ok(Some(metrics)) => (StatusCode::OK, Json(metrics)).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "No metrics found for article"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/blog/analytics
/// Get dashboard-level analytics
pub async fn get_dashboard_analytics(State(ctx): State<AppContext>) -> impl IntoResponse {
    let metrics_repo = MetricsRepository::new(ctx.db_pool().clone());

    match metrics_repo.get_top_articles(10).await {
        Ok(top_articles) => {
            let total_articles = top_articles.len();
            let total_views: i32 = top_articles.iter().map(|m| m.total_views).sum();
            let avg_views = if total_articles > 0 {
                f64::from(total_views) / total_articles as f64
            } else {
                0.0
            };

            let dashboard = json!({
                "total_articles": total_articles,
                "total_views": total_views,
                "avg_views_per_article": avg_views,
                "top_articles": top_articles,
            });

            (StatusCode::OK, Json(dashboard)).into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/blog/analytics/top
/// Get top performing articles
pub async fn get_top_articles(State(ctx): State<AppContext>) -> impl IntoResponse {
    let metrics_repo = MetricsRepository::new(ctx.db_pool().clone());

    match metrics_repo.get_top_articles(20).await {
        Ok(articles) => {
            let response = json!({
                "total": articles.len(),
                "articles": articles,
            });
            (StatusCode::OK, Json(response)).into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// GET /api/v1/blog/analytics/:id/trend
/// Get trend data for an article
pub async fn get_article_trend(
    State(ctx): State<AppContext>,
    Path(article_id): Path<String>,
) -> impl IntoResponse {
    let metrics_repo = MetricsRepository::new(ctx.db_pool().clone());

    match metrics_repo.get_metrics(&article_id).await {
        Ok(Some(metrics)) => {
            let trend_data = json!({
                "article_id": article_id,
                "views": metrics.total_views,
                "unique_visitors": metrics.unique_visitors,
                "avg_engagement": metrics.avg_time_on_page_seconds,
                "trend_direction": metrics.trend_direction,
                "views_last_7_days": metrics.views_last_7_days,
                "views_last_30_days": metrics.views_last_30_days,
            });
            (StatusCode::OK, Json(trend_data)).into_response()
        },
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Article not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
