use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::repositories;

pub(crate) async fn marketplace_versions_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user_id = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();

    if let Err(r) = super::authenticate(&headers, &user_id) {
        return *r;
    }

    match repositories::marketplace_versions::list_marketplace_versions(pool.as_ref(), &user_id)
        .await
    {
        Ok(versions) => Json(versions).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list versions");
            super::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list versions")
        }
    }
}

pub(crate) async fn marketplace_changelog_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    headers: HeaderMap,
) -> Response {
    let user_id = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();

    if let Err(r) = super::authenticate(&headers, &user_id) {
        return *r;
    }

    match repositories::marketplace_versions::list_changelog(pool.as_ref(), &user_id, 50).await {
        Ok(entries) => Json(entries).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list changelog");
            super::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to list changelog",
            )
        }
    }
}

pub(crate) async fn get_base_skill_content_handler(
    Path(skill_id): Path<String>,
    headers: HeaderMap,
) -> Response {
    if super::super::users::extract_user_from_cookie(&headers).is_err() {
        return super::error_response(StatusCode::UNAUTHORIZED, "Authentication required");
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let skill_dir = services_path.join("skills").join(&skill_id);
    if !skill_dir.is_dir() {
        return super::error_response(StatusCode::NOT_FOUND, "Base skill not found");
    }

    let config_path = skill_dir.join("config.yaml");
    let content_path = skill_dir.join("index.md");

    let Some(config_str) = std::fs::read_to_string(&config_path)
        .map_err(|e| {
            tracing::warn!(error = %e, path = %config_path.display(), "Failed to read skill config");
        })
        .ok()
    else {
        return super::error_response(StatusCode::NOT_FOUND, "Base skill config not found");
    };

    let content_str = std::fs::read_to_string(&content_path).unwrap_or_else(|_| String::new());

    let config: serde_json::Value =
        serde_yaml::from_str(&config_str).unwrap_or(serde_json::json!({}));

    let name = config
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(&skill_id)
        .to_string();
    let description = config
        .get("description")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();

    Json(serde_json::json!({
        "skill_id": skill_id,
        "name": name,
        "description": description,
        "content": content_str,
        "config": config_str,
    }))
    .into_response()
}

pub(crate) async fn marketplace_version_detail_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, version_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let user_id = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();

    if let Err(r) = super::authenticate(&headers, &user_id) {
        return *r;
    }

    match repositories::marketplace_versions::get_marketplace_version(
        pool.as_ref(),
        &user_id,
        &version_id,
    )
    .await
    {
        Ok(Some(version)) => Json(version).into_response(),
        Ok(None) => super::error_response(StatusCode::NOT_FOUND, "Version not found"),
        Err(e) => {
            tracing::error!(error = %e, "Failed to get version detail");
            super::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get version")
        }
    }
}

pub(crate) async fn marketplace_all_versions_handler(State(pool): State<Arc<PgPool>>) -> Response {
    match repositories::marketplace_versions::list_all_versions_summary(pool.as_ref()).await {
        Ok(versions) => Json(versions).into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to list all versions");
            super::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list versions")
        }
    }
}
