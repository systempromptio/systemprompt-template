use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use crate::admin::repositories;
pub(crate) use super::marketplace_git_json::{
    marketplace_json_handler, org_marketplace_json_handler,
};

pub(crate) fn detect_platform(headers: &HeaderMap) -> &'static str {
    if let Some(ua) = headers
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
    {
        if ua.contains("windows") || ua.contains("Windows") {
            return "windows";
        }
    }
    "unix"
}

async fn resolve_org_repo(
    pool: &Arc<PgPool>,
    marketplace_id_raw: &str,
    headers: &HeaderMap,
) -> Result<PathBuf, Response> {
    let marketplace_id = marketplace_id_raw
        .strip_suffix(".git")
        .unwrap_or(marketplace_id_raw);
    let platform = detect_platform(headers);

    repositories::marketplace_git::get_or_generate_org_marketplace_repo(
        pool,
        marketplace_id,
        platform,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, marketplace_id = marketplace_id, "Failed to generate org marketplace repo");
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to generate marketplace: {e}"),
        )
            .into_response()
    })
}

async fn resolve_repo(
    pool: &Arc<PgPool>,
    user_id_raw: &str,
    headers: &HeaderMap,
) -> Result<PathBuf, Response> {
    let user_id = user_id_raw.strip_suffix(".git").unwrap_or(user_id_raw);
    let platform = detect_platform(headers);

    repositories::marketplace_git::get_or_generate_marketplace_repo(pool, user_id, platform)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = user_id, "Failed to generate marketplace repo");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Failed to generate marketplace: {e}"),
            )
                .into_response()
        })
}

fn resolve_content_type(file_path: &str) -> &'static str {
    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str());
    if file_path == "HEAD" || file_path == "info/refs" {
        "text/plain"
    } else if ext == Some("pack") {
        "application/x-git-packed-objects"
    } else if ext == Some("idx") {
        "application/x-git-packed-objects-toc"
    } else {
        "application/octet-stream"
    }
}

#[derive(serde::Deserialize, Default)]
pub(crate) struct InfoRefsQuery {
    service: Option<String>,
}

pub(crate) async fn org_marketplace_git_handler(
    State(pool): State<Arc<PgPool>>,
    Path((marketplace_id_raw, file_path)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo_path = match resolve_org_repo(&pool, &marketplace_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };

    if file_path == "info/refs" {
        if let Some(ref service) = query.service {
            if service == "git-upload-pack" {
                return smart_info_refs(&repo_path);
            }
        }
    }

    let target = repo_path.join(&file_path);
    if !target.starts_with(&repo_path) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Ok(content) = std::fs::read(&target) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, resolve_content_type(&file_path))],
        content,
    )
        .into_response()
}

pub(crate) async fn org_git_upload_pack_handler(
    State(pool): State<Arc<PgPool>>,
    Path((marketplace_id_raw, _path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_path = match resolve_org_repo(&pool, &marketplace_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    run_upload_pack(&repo_path, &body)
}

pub(crate) async fn marketplace_git_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, file_path)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo_path = match resolve_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };

    if file_path == "info/refs" {
        if let Some(ref service) = query.service {
            if service == "git-upload-pack" {
                return smart_info_refs(&repo_path);
            }
        }
    }

    let target = repo_path.join(&file_path);
    if !target.starts_with(&repo_path) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Ok(content) = std::fs::read(&target) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, resolve_content_type(&file_path))],
        content,
    )
        .into_response()
}

fn smart_info_refs(repo_path: &PathBuf) -> Response {
    let output = match std::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "--advertise-refs"])
        .arg(repo_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            tracing::error!(error = %e, "Failed to run git upload-pack --advertise-refs");
            return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(stderr = %stderr, "git upload-pack --advertise-refs failed");
        return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
    }

    let mut body = Vec::new();
    body.extend_from_slice(b"001e# service=git-upload-pack\n");
    body.extend_from_slice(b"0000");
    body.extend_from_slice(&output.stdout);

    (
        StatusCode::OK,
        [
            (
                header::CONTENT_TYPE,
                "application/x-git-upload-pack-advertisement",
            ),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        body,
    )
        .into_response()
}

pub(crate) async fn git_upload_pack_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, _path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_path = match resolve_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    run_upload_pack(&repo_path, &body)
}

fn run_upload_pack(repo_path: &PathBuf, body: &[u8]) -> Response {
    let mut child = match std::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc"])
        .arg(repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    {
        Ok(c) => c,
        Err(e) => {
            tracing::error!(error = %e, "Failed to spawn git upload-pack");
            return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
        }
    };

    if let Some(mut stdin) = child.stdin.take() {
        use std::io::Write;
        if let Err(e) = stdin.write_all(body) {
            tracing::error!(error = %e, "Failed to write to git upload-pack stdin");
            return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
        }
    }

    let output = match child.wait_with_output() {
        Ok(o) => o,
        Err(e) => {
            tracing::error!(error = %e, "Failed to wait for git upload-pack");
            return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(stderr = %stderr, "git upload-pack --stateless-rpc failed");
        return (StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed").into_response();
    }

    (
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/x-git-upload-pack-result"),
            (header::CACHE_CONTROL, "no-cache"),
        ],
        output.stdout,
    )
        .into_response()
}
