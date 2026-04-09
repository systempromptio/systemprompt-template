use std::path::PathBuf;
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
use crate::admin::types::UserBasicInfo;

fn sanitize_filename(name: &str) -> String {
    let sanitized: String = name
        .to_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    sanitized
        .trim_matches('-')
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

fn zip_attachment_response(filename: &str, zip_data: Vec<u8>) -> Response {
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

struct UserExportContext {
    user_id: UserId,
    services_path: PathBuf,
    user_info: UserBasicInfo,
}

async fn prepare_export_context(
    pool: &PgPool,
    user_id_str: String,
    context_label: &str,
) -> Result<UserExportContext, Response> {
    let user_id = UserId::new(user_id_str);
    let services_path = shared::get_services_path().map_err(|r| *r)?;

    let user_info = repositories::marketplace_git::lookup_user_basic(pool, &user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user_id, "User not found for {context_label}");
            shared::error_response(StatusCode::NOT_FOUND, "User not found")
        })?;

    Ok(UserExportContext {
        user_id,
        services_path,
        user_info,
    })
}

async fn generate_bundles(
    ctx: &UserExportContext,
    pool: &PgPool,
    context_label: &str,
) -> Result<repositories::export::SyncPluginsResponse, Response> {
    let export_params = repositories::ExportParams {
        services_path: &ctx.services_path,
        pool,
        user_id: &ctx.user_id,
        username: &ctx.user_info.display_name,
        email: &ctx.user_info.email,
        roles: &ctx.user_info.roles,
    };

    repositories::generate_export_bundles(&export_params)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to generate export bundles for {context_label}");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                &format!("Export failed: {e}"),
            )
        })
}

pub async fn export_plugin_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_str, plugin_id)): Path<(String, String)>,
) -> Response {
    let ctx = match prepare_export_context(&pool, user_id_str, "ZIP export").await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let response = match generate_bundles(&ctx, &pool, "ZIP").await {
        Ok(r) => r,
        Err(r) => return r,
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
    zip_attachment_response(&filename, zip_data)
}

pub async fn export_marketplace_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_str): Path<String>,
) -> Response {
    let ctx = match prepare_export_context(&pool, user_id_str, "marketplace ZIP export").await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let response = match generate_bundles(&ctx, &pool, "marketplace ZIP").await {
        Ok(r) => r,
        Err(r) => return r,
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

    let filename = format!(
        "{}-marketplace.zip",
        sanitize_filename(&ctx.user_info.display_name)
    );
    zip_attachment_response(&filename, zip_data)
}

pub async fn export_cowork_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_str): Path<String>,
) -> Response {
    let ctx = match prepare_export_context(&pool, user_id_str, "Cowork ZIP export").await {
        Ok(c) => c,
        Err(r) => return r,
    };

    let platform_url = Config::get().map_or_else(|_| String::new(), |c| c.api_external_url.clone());

    let response = match generate_bundles(&ctx, &pool, "Cowork ZIP").await {
        Ok(r) => r,
        Err(r) => return r,
    };

    let cowork_params = repositories::export_zip::CoworkExportParams {
        response: &response,
        username: &ctx.user_info.display_name,
        email: &ctx.user_info.email,
        platform_url: &platform_url,
        user_id: &ctx.user_id,
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

    let filename = format!(
        "{}-cowork.zip",
        sanitize_filename(&ctx.user_info.display_name)
    );
    zip_attachment_response(&filename, zip_data)
}
