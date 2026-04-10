use std::sync::Arc;

use crate::repositories;
use crate::types::marketplaces::{
    CreateOrgMarketplaceRequest, OrgMarketplace, UpdateOrgMarketplaceRequest,
};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::repositories::github_sync;
use crate::types::UserContext;

pub async fn list_org_marketplaces_handler(State(pool): State<Arc<PgPool>>) -> impl IntoResponse {
    match repositories::org_marketplaces::list_org_marketplaces(&pool).await {
        Ok(marketplaces) => (StatusCode::OK, Json(json!(marketplaces))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list org marketplaces");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to list marketplaces"})),
            )
                .into_response()
        }
    }
}

pub async fn create_org_marketplace_handler(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Json(req): Json<CreateOrgMarketplaceRequest>,
) -> impl IntoResponse {
    match repositories::org_marketplaces::create_org_marketplace(&pool, &req).await {
        Ok(marketplace) => (StatusCode::CREATED, Json(json!(marketplace))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to create org marketplace");
            let msg = if e.to_string().contains("duplicate key") {
                "A marketplace with this ID already exists"
            } else {
                "Failed to create marketplace"
            };
            (StatusCode::BAD_REQUEST, Json(json!({"error": msg}))).into_response()
        }
    }
}

pub async fn update_org_marketplace_handler(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateOrgMarketplaceRequest>,
) -> impl IntoResponse {
    match repositories::org_marketplaces::update_org_marketplace(&pool, &id, &req).await {
        Ok(Some(marketplace)) => (StatusCode::OK, Json(json!(marketplace))).into_response(),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Marketplace not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to update org marketplace");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to update marketplace"})),
            )
                .into_response()
        }
    }
}

pub async fn delete_org_marketplace_handler(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    match repositories::org_marketplaces::delete_org_marketplace(&pool, &id).await {
        Ok(true) => (StatusCode::OK, Json(json!({"deleted": true}))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(json!({"error": "Marketplace not found"})),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to delete org marketplace");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to delete marketplace"})),
            )
                .into_response()
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SetPluginsRequest {
    pub plugin_ids: Vec<String>,
}

pub async fn set_marketplace_plugins_handler(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(req): Json<SetPluginsRequest>,
) -> impl IntoResponse {
    match repositories::org_marketplaces::find_org_marketplace(&pool, &id).await {
        Ok(Some(_)) => {}
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Marketplace not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to check marketplace");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to check marketplace"})),
            )
                .into_response();
        }
    }

    match repositories::org_marketplaces::set_marketplace_plugins(&pool, &id, &req.plugin_ids).await
    {
        Ok(()) => (StatusCode::OK, Json(json!({"updated": true}))).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to set marketplace plugins");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to set marketplace plugins"})),
            )
                .into_response()
        }
    }
}

pub async fn sync_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_marketplace, repo_url) = match find_marketplace_with_repo(&pool, &id).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };

    match github_sync::sync_marketplace_from_github(
        &pool,
        &id,
        &repo_url,
        user_ctx.user_id.as_str(),
    )
    .await
    {
        Ok(result) => build_sync_result_response(&result),
        Err(e) => {
            tracing::error!(error = %e, marketplace_id = %id, "Marketplace sync failed");
            log_sync_error(
                &pool,
                &id,
                "sync",
                &e.to_string(),
                user_ctx.user_id.as_str(),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Sync failed"})),
            )
                .into_response()
        }
    }
}

pub async fn publish_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let (_marketplace, repo_url) = match find_marketplace_with_repo(&pool, &id).await {
        Ok(pair) => pair,
        Err(resp) => return resp,
    };

    match github_sync::publish_marketplace_to_github(
        &pool,
        &id,
        &repo_url,
        user_ctx.user_id.as_str(),
    )
    .await
    {
        Ok(result) => build_sync_result_response(&result),
        Err(e) => {
            tracing::error!(error = %e, marketplace_id = %id, "Marketplace publish failed");
            log_sync_error(
                &pool,
                &id,
                "publish",
                &e.to_string(),
                user_ctx.user_id.as_str(),
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Publish failed"})),
            )
                .into_response()
        }
    }
}

async fn find_marketplace_with_repo(
    pool: &PgPool,
    id: &str,
) -> Result<(OrgMarketplace, String), Response> {
    let marketplace = match repositories::org_marketplaces::find_org_marketplace(pool, id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Marketplace not found"})),
            )
                .into_response());
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to look up marketplace");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to look up marketplace"})),
            )
                .into_response());
        }
    };

    let repo_url = marketplace.github_repo_url.clone().ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Marketplace has no GitHub repository URL configured"})),
        )
            .into_response()
    })?;

    Ok((marketplace, repo_url))
}

fn build_sync_result_response(result: &github_sync::SyncResult) -> Response {
    (
        StatusCode::OK,
        Json(json!({
            "success": true,
            "commit_hash": result.commit_hash,
            "plugins_synced": result.plugins_synced,
            "errors": result.errors,
            "changed": result.changed,
            "duration_ms": result.duration_ms,
        })),
    )
        .into_response()
}

async fn log_sync_error(
    pool: &PgPool,
    id: &str,
    operation: &str,
    error_msg: &str,
    triggered_by: &str,
) {
    let _ = repositories::org_marketplaces::insert_sync_log(
        pool,
        &repositories::org_marketplaces::SyncLogEntry {
            marketplace_id: id,
            operation,
            status: "error",
            commit_hash: None,
            plugins_synced: 0,
            errors: 1,
            error_message: Some(error_msg),
            triggered_by,
            duration_ms: None,
        },
    )
    .await;
}
