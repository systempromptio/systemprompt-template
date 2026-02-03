use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use chrono::Utc;
use sqlx::PgPool;
use systemprompt::identifiers::{LinkClickId, SessionId};

const DEFAULT_CLICK_LIMIT: i64 = 100;

use crate::api::{
    BlogState, ContentJourneyQuery, GenerateLinkRequest, GenerateLinkResponse, ListLinksQuery,
    RecordClickRequest,
};
use crate::models::RecordClickParams;
use crate::repository::{LinkAnalyticsRepository, LinkRepository};
use crate::services::{LinkAnalyticsService, LinkGenerationService, LinkService};

pub async fn generate_link_handler(
    State(state): State<BlogState>,
    Json(request): Json<GenerateLinkRequest>,
) -> Response {
    let service = LinkGenerationService::new(state.pool.clone());

    match service
        .generate(
            request.target_url,
            request.campaign_name,
            request.utm_params,
        )
        .await
    {
        Ok(link) => {
            let base_url = state
                .config
                .as_ref()
                .map_or("https://example.com", |c| c.base_url().as_str());
            let response = GenerateLinkResponse {
                id: link.id,
                short_code: link.short_code.clone(),
                short_url: format!("{}/r/{}", base_url.trim_end_matches('/'), link.short_code),
                target_url: link.target_url,
            };
            Json(response).into_response()
        }
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn list_links_handler(
    State(state): State<BlogState>,
    Query(query): Query<ListLinksQuery>,
) -> Response {
    let repo = LinkRepository::new(state.pool.clone());

    let result = if let Some(ref campaign_id) = query.campaign_id {
        repo.list_links_by_campaign(campaign_id).await
    } else if let Some(ref content_id) = query.content_id {
        repo.list_links_by_source_content(content_id).await
    } else {
        Ok(vec![])
    };

    match result {
        Ok(links) => Json(serde_json::json!({
            "links": links,
            "total": links.len()
        }))
        .into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn record_click_handler(
    State(state): State<BlogState>,
    Json(request): Json<RecordClickRequest>,
) -> Response {
    let analytics_repo = LinkAnalyticsRepository::new(state.pool.clone());

    let click_id = LinkClickId::generate();
    let link_id = request.link_id;
    let session_id = SessionId::new(
        request
            .session_id
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
    );

    let params = RecordClickParams::new(click_id, link_id, session_id, Utc::now())
        .with_referrer_page(request.referrer_page)
        .with_referrer_url(request.referrer_url)
        .with_user_agent(request.user_agent)
        .with_device_type(request.device_type);

    match analytics_repo.record_click(&params).await {
        Ok(()) => StatusCode::CREATED.into_response(),
        Err(e) => {
            tracing::warn!(error = %e, "Failed to record click");
            error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string())
        }
    }
}

pub async fn link_performance_handler(
    State(state): State<BlogState>,
    Path(link_id): Path<String>,
) -> Response {
    let service = LinkService::new(state.pool.clone());

    match service.get_performance(&link_id).await {
        Ok(Some(perf)) => Json(perf).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Link not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn link_clicks_handler(
    State(state): State<BlogState>,
    Path(link_id): Path<String>,
) -> Response {
    let service = LinkService::new(state.pool.clone());

    match service.get_clicks(&link_id, DEFAULT_CLICK_LIMIT).await {
        Ok(clicks) => Json(clicks).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn campaign_performance_handler(
    State(state): State<BlogState>,
    Path(campaign_id): Path<String>,
) -> Response {
    let service = LinkAnalyticsService::new(state.pool.clone());

    match service.get_campaign_performance(&campaign_id).await {
        Ok(Some(perf)) => Json(perf).into_response(),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "Campaign not found"),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn content_journey_handler(
    State(state): State<BlogState>,
    Query(query): Query<ContentJourneyQuery>,
) -> Response {
    let service = LinkAnalyticsService::new(state.pool.clone());

    match service.get_content_journey(query.content_id.as_str()).await {
        Ok(journey) => Json(journey).into_response(),
        Err(e) => error_response(StatusCode::INTERNAL_SERVER_ERROR, &e.to_string()),
    }
}

pub async fn redirect_handler(
    State(pool): State<Arc<PgPool>>,
    Path(short_code): Path<String>,
) -> Response {
    let service = LinkService::new(pool);

    let session_id = uuid::Uuid::new_v4().to_string();

    match service
        .process_redirect(&short_code, &session_id, None, None)
        .await
    {
        Ok(target_url) => Redirect::temporary(&target_url).into_response(),
        Err(e) => {
            tracing::warn!(short_code = %short_code, error = %e, "Redirect failed");
            error_response(StatusCode::NOT_FOUND, "Link not found")
        }
    }
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
