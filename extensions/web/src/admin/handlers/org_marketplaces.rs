use std::sync::Arc;

use crate::admin::repositories;
use crate::admin::types::marketplaces::{CreateOrgMarketplaceRequest, UpdateOrgMarketplaceRequest};
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::PgPool;

use crate::admin::repositories::github_sync;
use crate::admin::types::UserContext;

pub(crate) async fn list_org_marketplaces_handler(
    State(pool): State<Arc<PgPool>>,
) -> impl IntoResponse {
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

pub(crate) async fn create_org_marketplace_handler(
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

pub(crate) async fn update_org_marketplace_handler(
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

pub(crate) async fn delete_org_marketplace_handler(
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
pub(crate) struct SetPluginsRequest {
    pub plugin_ids: Vec<String>,
}

pub(crate) async fn set_marketplace_plugins_handler(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
    Json(req): Json<SetPluginsRequest>,
) -> impl IntoResponse {
    match repositories::org_marketplaces::get_org_marketplace(&pool, &id).await {
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

pub(crate) async fn sync_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let marketplace = match repositories::org_marketplaces::get_org_marketplace(&pool, &id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Marketplace not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to look up marketplace");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to look up marketplace"})),
            )
                .into_response();
        }
    };

    let Some(ref repo_url) = marketplace.github_repo_url else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Marketplace has no GitHub repository URL configured"})),
        )
            .into_response();
    };

    match github_sync::sync_marketplace_from_github(&pool, &id, repo_url, &user_ctx.user_id).await {
        Ok(result) => (
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
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, marketplace_id = %id, "Marketplace sync failed");
            let _ = repositories::org_marketplaces::insert_sync_log(
                &pool,
                &id,
                "sync",
                "error",
                None,
                0i64,
                1i64,
                Some(&e.to_string()),
                user_ctx.user_id.as_str(),
                None::<i64>,
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Sync failed: {e}")})),
            )
                .into_response()
        }
    }
}

pub(crate) async fn publish_marketplace_handler(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let marketplace = match repositories::org_marketplaces::get_org_marketplace(&pool, &id).await {
        Ok(Some(m)) => m,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({"error": "Marketplace not found"})),
            )
                .into_response();
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to look up marketplace");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": "Failed to look up marketplace"})),
            )
                .into_response();
        }
    };

    let Some(ref repo_url) = marketplace.github_repo_url else {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({"error": "Marketplace has no GitHub repository URL configured"})),
        )
            .into_response();
    };

    match github_sync::publish_marketplace_to_github(&pool, &id, repo_url, &user_ctx.user_id).await
    {
        Ok(result) => (
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
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, marketplace_id = %id, "Marketplace publish failed");
            let _ = repositories::org_marketplaces::insert_sync_log(
                &pool,
                &id,
                "publish",
                "error",
                None,
                0i64,
                1i64,
                Some(&e.to_string()),
                user_ctx.user_id.as_str(),
                None::<i64>,
            )
            .await;
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({"error": format!("Publish failed: {e}")})),
            )
                .into_response()
        }
    }
}
