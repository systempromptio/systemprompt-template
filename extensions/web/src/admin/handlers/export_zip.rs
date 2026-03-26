use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{header, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;
use systemprompt::models::Config;

use crate::admin::handlers::shared;
use crate::admin::repositories;

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' { c } else { '-' })
        .collect();

    sanitized
        .trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

pub(crate) async fn export_plugin_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_str, plugin_id)): Path<(String, String)>,
) -> Response {
    let user_id = UserId::new(user_id_str);
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let user_info = match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "User not found for ZIP export");
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        }
    };

    let export_params = repositories::ExportParams {
        services_path: &services_path,
        pool: &pool,
        user_id: &user_id,
        username: &user_info.display_name,
        email: &user_info.email,
        roles: &user_info.roles,
    };

    let response = match repositories::generate_export_bundles(&export_params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles for ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let Some(bundle) = response.plugins.iter().find(|b| b.id == plugin_id) else {
        return shared::error_response(
            StatusCode::NOT_FOUND,
            &format!("Plugin '{plugin_id}' not found in export"),
        );
    };

    let zip_data = match repositories::export_zip::build_plugin_zip(bundle) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(error = %e, plugin_id = %plugin_id, "Failed to build plugin ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ZIP generation failed: {e}"),
            );
        }
    };

    let filename = format!("{}.zip", sanitize_filename(&bundle.id));
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
    Path(user_id_str): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id_str);
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let user_info = match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "User not found for marketplace ZIP export");
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        }
    };

    let export_params = repositories::ExportParams {
        services_path: &services_path,
        pool: &pool,
        user_id: &user_id,
        username: &user_info.display_name,
        email: &user_info.email,
        roles: &user_info.roles,
    };

    let response = match repositories::generate_export_bundles(&export_params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles for marketplace ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let zip_data = match repositories::export_zip::build_marketplace_zip(&response) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(error = %e, "Failed to build marketplace ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ZIP generation failed: {e}"),
            );
        }
    };

    let filename = format!("{}-marketplace.zip", sanitize_filename(&user_info.display_name));
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

pub(crate) async fn export_cowork_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_str): Path<String>,
) -> Response {
    let user_id = UserId::new(user_id_str);
    let services_path = match shared::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let user_info = match repositories::marketplace_git::lookup_user_basic(&pool, &user_id).await {
        Ok(info) => info,
        Err(e) => {
            tracing::error!(error = %e, user_id = %user_id, "User not found for Cowork ZIP export");
            return shared::error_response(StatusCode::NOT_FOUND, "User not found");
        }
    };

    let platform_url =
        Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let export_params = repositories::ExportParams {
        services_path: &services_path,
        pool: &pool,
        user_id: &user_id,
        username: &user_info.display_name,
        email: &user_info.email,
        roles: &user_info.roles,
    };

    let response = match repositories::generate_export_bundles(&export_params).await {
        Ok(r) => r,
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate export bundles for Cowork ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            );
        }
    };

    let cowork_params = repositories::export_zip::CoworkExportParams {
        response: &response,
        username: &user_info.display_name,
        email: &user_info.email,
        platform_url: &platform_url,
        user_id: &user_id,
    };

    let zip_data = match repositories::export_zip::build_cowork_plugin_zip(&cowork_params) {
        Ok(data) => data,
        Err(e) => {
            tracing::error!(error = %e, "Failed to build Cowork plugin ZIP");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("ZIP generation failed: {e}"),
            );
        }
    };

    let filename = format!("{}-cowork.zip", sanitize_filename(&user_info.display_name));
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
