use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use super::resources::get_services_path;
use crate::admin::repositories;

pub(crate) async fn export_plugin_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id, plugin_id)): Path<(String, String)>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let (username, email, roles, department) =
        match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(error = %e, user_id = %user_id, "User not found for ZIP export");
                return error_response(StatusCode::NOT_FOUND, "User not found");
            }
        };

    let response = match repositories::generate_export_bundles(
        &services_path,
        &pool,
        &user_id,
        &username,
        &email,
        &roles,
        &department,
        "",
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles for ZIP");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let Some(bundle) = response.plugins.iter().find(|b| b.id == plugin_id) else {
        return error_response(
            StatusCode::NOT_FOUND,
            &format!("Plugin '{plugin_id}' not found in export"),
        );
    };

    let zip_data = match repositories::export_zip::build_plugin_zip(bundle) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(error = %e, plugin_id = %plugin_id, "Failed to build plugin ZIP");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ZIP generation failed: {e}"),
            );
        }
    };

    let filename = format!("{}.zip", bundle.id);
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        zip_data,
    )
        .into_response()
}

pub(crate) async fn export_marketplace_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id): Path<String>,
) -> Response {
    let services_path = match get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let (username, email, roles, department) =
        match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
            Ok(info) => info,
            Err(e) => {
                tracing::error!(error = %e, user_id = %user_id, "User not found for marketplace ZIP export");
                return error_response(StatusCode::NOT_FOUND, "User not found");
            }
        };

    let response = match repositories::generate_export_bundles(
        &services_path,
        &pool,
        &user_id,
        &username,
        &email,
        &roles,
        &department,
        "",
    )
    .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles for marketplace ZIP");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let zip_data = match repositories::export_zip::build_marketplace_zip(&response) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(error = %e, "Failed to build marketplace ZIP");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ZIP generation failed: {e}"),
            );
        }
    };

    let filename = format!("{username}-marketplace.zip");
    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/zip".to_string()),
            (
                header::CONTENT_DISPOSITION,
                format!("attachment; filename=\"{filename}\""),
            ),
        ],
        zip_data,
    )
        .into_response()
}

fn error_response(status: StatusCode, message: &str) -> Response {
    (status, Json(serde_json::json!({"error": message}))).into_response()
}
