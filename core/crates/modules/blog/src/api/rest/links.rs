use crate::models::{LinkType, UtmParams};
use crate::services::{LinkAnalyticsService, LinkGenerationService};
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Redirect};
use axum::{Extension, Json};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use systemprompt_core_logging::LogService;
use systemprompt_core_system::{AppContext, RequestContext};

#[derive(Debug, Deserialize)]
pub struct GenerateLinkRequest {
    pub target_url: String,
    pub link_type: String,
    pub campaign_id: Option<String>,
    pub campaign_name: Option<String>,
    pub source_content_id: Option<String>,
    pub source_page: Option<String>,
    pub utm_source: Option<String>,
    pub utm_medium: Option<String>,
    pub utm_campaign: Option<String>,
    pub utm_term: Option<String>,
    pub utm_content: Option<String>,
    pub link_text: Option<String>,
    pub link_position: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct GenerateLinkResponse {
    pub link_id: String,
    pub short_code: String,
    pub redirect_url: String,
    pub full_url: String,
}

#[derive(Debug, Deserialize)]
pub struct ListLinksQuery {
    pub campaign_id: Option<String>,
    pub source_content_id: Option<String>,
}

#[derive(Debug, Deserialize, Copy, Clone)]
pub struct AnalyticsQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

pub async fn redirect_handler(
    State(ctx): State<AppContext>,
    Extension(req_ctx): Extension<RequestContext>,
    Path(short_code): Path<String>,
) -> impl IntoResponse {
    let link_gen_service = LinkGenerationService::new(ctx.db_pool().clone());
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());
    let logger = LogService::new(ctx.db_pool().clone(), req_ctx.log_context());

    let link = match link_gen_service.get_link_by_short_code(&short_code).await {
        Ok(Some(link)) => link,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Link not found"})),
            )
                .into_response();
        },
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response();
        },
    };

    if let Err(e) = analytics_service
        .track_click(
            &link.id,
            req_ctx.request.session_id.as_str(),
            Some(req_ctx.auth.user_id.to_string()),
            Some(req_ctx.execution.context_id.to_string()),
            req_ctx.execution.task_id.as_ref().map(ToString::to_string),
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .await
    {
        logger
            .error(
                "link_tracking",
                &format!("Failed to track click for link_id {}: {}", link.id, e),
            )
            .await
            .ok();
    }

    let target_url = link.get_full_url();
    Redirect::temporary(&target_url).into_response()
}

pub async fn generate_link_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Json(payload): Json<GenerateLinkRequest>,
) -> impl IntoResponse {
    let link_type = match payload.link_type.as_str() {
        "redirect" => LinkType::Redirect,
        "utm" => LinkType::Utm,
        "both" => LinkType::Both,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid link_type. Must be 'redirect', 'utm', or 'both'"})),
            )
                .into_response();
        },
    };

    let utm_params = if payload.utm_source.is_some()
        || payload.utm_medium.is_some()
        || payload.utm_campaign.is_some()
    {
        Some(UtmParams {
            source: payload.utm_source,
            medium: payload.utm_medium,
            campaign: payload.utm_campaign,
            term: payload.utm_term,
            content: payload.utm_content,
        })
    } else {
        None
    };

    let link_gen_service = LinkGenerationService::new(ctx.db_pool().clone());

    match link_gen_service
        .generate_link(
            &payload.target_url,
            link_type,
            payload.campaign_id,
            payload.campaign_name,
            payload.source_content_id,
            payload.source_page,
            utm_params,
            payload.link_text,
            payload.link_position,
            payload.expires_at,
        )
        .await
    {
        Ok(link) => {
            let base_url =
                std::env::var("BASE_URL").unwrap_or_else(|_| "http://localhost:8080".to_string());
            let redirect_url = LinkGenerationService::build_trackable_url(&link, &base_url);
            let full_url = link.get_full_url();

            Json(GenerateLinkResponse {
                link_id: link.id,
                short_code: link.short_code,
                redirect_url,
                full_url,
            })
            .into_response()
        },
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_link_performance_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Path(link_id): Path<String>,
) -> impl IntoResponse {
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());

    match analytics_service.get_link_performance(&link_id).await {
        Ok(Some(performance)) => Json(performance).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Link not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_campaign_performance_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Path(campaign_id): Path<String>,
) -> impl IntoResponse {
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());

    match analytics_service
        .get_campaign_performance(&campaign_id)
        .await
    {
        Ok(Some(performance)) => Json(performance).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Campaign not found"})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn get_content_journey_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());

    match analytics_service
        .get_content_journey_map(query.limit, query.offset)
        .await
    {
        Ok(journey) => Json(journey).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

pub async fn list_links_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Query(query): Query<ListLinksQuery>,
) -> impl IntoResponse {
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());

    if let Some(campaign_id) = query.campaign_id {
        match analytics_service.get_links_by_campaign(&campaign_id).await {
            Ok(links) => Json(links).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        }
    } else if let Some(source_content_id) = query.source_content_id {
        match analytics_service
            .get_links_by_source_content(&source_content_id)
            .await
        {
            Ok(links) => Json(links).into_response(),
            Err(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
                .into_response(),
        }
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Must provide either campaign_id or source_content_id"})),
        )
            .into_response()
    }
}

pub async fn get_link_clicks_handler(
    State(ctx): State<AppContext>,
    Extension(_req_ctx): Extension<RequestContext>,
    Path(link_id): Path<String>,
    Query(query): Query<AnalyticsQuery>,
) -> impl IntoResponse {
    let analytics_service = LinkAnalyticsService::new(ctx.db_pool().clone());

    match analytics_service
        .get_link_clicks(&link_id, query.limit, query.offset)
        .await
    {
        Ok(clicks) => Json(clicks).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}
