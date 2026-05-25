use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path as AxumPath, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::Response,
};
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::PluginId;
use systemprompt::models::AppPaths;

use crate::handlers::shared;

pub async fn handle(
    State(pool): State<Arc<PgPool>>,
    AxumPath((plugin_id, relative_path)): AxumPath<(String, String)>,
    headers: HeaderMap,
) -> Response {
    let user_id = match super::validate_bridge_jwt(&headers) {
        Ok(id) => id,
        Err(r) => return *r,
    };

    let plugin_id = PluginId::new(plugin_id);

    let _ = pool;
    let _ = user_id;

    let plugin_dir = match resolve_plugin_dir(&plugin_id) {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let resolved = match resolve_within(&plugin_dir, &relative_path) {
        Ok(p) => p,
        Err(reason) => {
            tracing::warn!(
                user_id = %user_id.as_str(),
                plugin_id = %plugin_id.as_str(),
                path = %relative_path,
                reason = %reason,
                "rejected plugin file request",
            );
            return shared::error_response(StatusCode::BAD_REQUEST, "Invalid path");
        }
    };

    let bytes = match std::fs::read(&resolved) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return shared::error_response(StatusCode::NOT_FOUND, "File not found");
        }
        Err(e) => {
            tracing::error!(error = %e, path = %resolved.display(), "plugin file read failed");
            return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "File read failed");
        }
    };

    build_file_response(&resolved, bytes)
}

fn resolve_plugin_dir(plugin_id: &PluginId) -> Result<PathBuf, Box<Response>> {
    ProfileBootstrap::get()
        .map_err(|e| e.to_string())
        .and_then(|profile| AppPaths::from_profile(&profile.paths).map_err(|e| e.to_string()))
        .map(|p| p.storage().files().join("plugins").join(plugin_id.as_str()))
        .map_err(|e| {
            tracing::error!(error = %e, "AppPaths derivation failed");
            Box::new(shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Service paths unavailable",
            ))
        })
}

fn build_file_response(resolved: &Path, bytes: Vec<u8>) -> Response {
    let content_type = guess_content_type(resolved);
    let mut response = Response::new(Body::from(bytes));
    *response.status_mut() = StatusCode::OK;
    if let Ok(value) = HeaderValue::from_str(content_type) {
        response.headers_mut().insert(header::CONTENT_TYPE, value);
    }
    response
}

pub fn resolve_within(base: &Path, relative: &str) -> Result<PathBuf, &'static str> {
    if relative.is_empty() {
        return Err("empty path");
    }
    let candidate = Path::new(relative);
    for comp in candidate.components() {
        match comp {
            Component::Normal(_) | Component::CurDir => {}
            _ => return Err("non-canonical component"),
        }
    }

    let canonical_base = base.canonicalize().map_err(|_| "base unavailable")?;
    let target = canonical_base.join(candidate);
    let canonical_target = target.canonicalize().map_err(|_| "target unavailable")?;

    if !canonical_target.starts_with(&canonical_base) {
        return Err("escapes base");
    }
    if !canonical_target.is_file() {
        return Err("not a file");
    }
    Ok(canonical_target)
}

fn guess_content_type(path: &Path) -> &'static str {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(str::to_ascii_lowercase);
    match ext.as_deref() {
        Some("md") => "text/markdown; charset=utf-8",
        Some("txt") => "text/plain; charset=utf-8",
        Some("json") => "application/json",
        Some("yaml" | "yml") => "application/yaml",
        Some("toml") => "application/toml",
        Some("html" | "htm") => "text/html; charset=utf-8",
        Some("css") => "text/css; charset=utf-8",
        Some("js") => "application/javascript",
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("svg") => "image/svg+xml",
        Some("wasm") => "application/wasm",
        _ => "application/octet-stream",
    }
}
