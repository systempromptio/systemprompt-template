use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use super::marketplace_git::detect_platform;
use crate::admin::repositories;

pub(crate) async fn org_marketplace_json_handler(
    State(pool): State<Arc<PgPool>>,
    Path(marketplace_id_raw): Path<String>,
    headers: HeaderMap,
) -> Response {
    let marketplace_id = marketplace_id_raw
        .strip_suffix(".git")
        .unwrap_or(&marketplace_id_raw)
        .to_string();
    let platform = detect_platform(&headers);

    let services_path = match systemprompt::models::ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load profile"})),
            )
                .into_response();
        }
    };

    let response = match repositories::export::generate_org_marketplace_export_bundles(
        &services_path,
        &pool,
        &marketplace_id,
        platform,
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate org marketplace export");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Export failed: {e}"),
            )
                .into_response();
        }
    };

    let marketplace_json: serde_json::Value =
        match serde_json::from_str(&response.marketplace.content) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse marketplace JSON");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid marketplace JSON",
                )
                    .into_response();
            }
        };

    Json(marketplace_json).into_response()
}

pub(crate) async fn marketplace_json_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user_id = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();
    let platform = detect_platform(&headers);

    let persistent_path = std::path::PathBuf::from("storage/marketplace-versions")
        .join(&user_id)
        .join("marketplace.json");
    if persistent_path.is_file() {
        match std::fs::read_to_string(&persistent_path) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(json) => return Json(json).into_response(),
                Err(e) => {
                    tracing::warn!(error = %e, "Failed to parse persistent marketplace.json, falling back to on-demand generation");
                }
            },
            Err(e) => {
                tracing::warn!(error = %e, "Failed to read persistent marketplace.json, falling back to on-demand generation");
            }
        }
    }

    let services_path = match systemprompt::models::ProfileBootstrap::get() {
        Ok(profile) => std::path::PathBuf::from(&profile.paths.services),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get profile bootstrap");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to load profile"})),
            )
                .into_response();
        }
    };

    let (username, email, roles, _department) =
        match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(error = %e, user_id = %user_id, "User not found for marketplace");
                return (StatusCode::NOT_FOUND, "User not found").into_response();
            }
        };

    let _ = repositories::marketplace_sync_status::mark_user_dirty(&pool, &user_id).await;

    let uid = systemprompt::identifiers::UserId::new(&user_id);
    let export_params = repositories::ExportParams {
        services_path: &services_path,
        pool: &pool,
        user_id: &uid,
        username: &username,
        email: &email,
        roles: &roles,
    };
    let response = match repositories::generate_export_bundles(&export_params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Export failed: {e}"),
            )
                .into_response();
        }
    };

    let marketplace_json: serde_json::Value =
        match serde_json::from_str(&response.marketplace.content) {
            Ok(v) => v,
            Err(e) => {
                tracing::error!(error = %e, "Failed to parse marketplace JSON");
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Invalid marketplace JSON",
                )
                    .into_response();
            }
        };

    Json(marketplace_json).into_response()
}
