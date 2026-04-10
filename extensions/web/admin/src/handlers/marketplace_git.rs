use std::path::PathBuf;
use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use crate::handlers::shared;
use crate::repositories;
use crate::types::{GIT_HEAD, GIT_INFO_REFS, GIT_UPLOAD_PACK};

fn detect_platform(headers: &HeaderMap) -> &'static str {
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

async fn resolve_repo(
    pool: &PgPool,
    user_id_raw: &str,
    headers: &HeaderMap,
) -> Result<PathBuf, Response> {
    let user_id_str = shared::normalize_user_id(user_id_raw);
    let user_id = UserId::new(user_id_str);
    let platform = detect_platform(headers);

    repositories::marketplace_git::get_or_generate_marketplace_repo(pool, &user_id, platform)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user_id, "Failed to generate marketplace repo");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate marketplace",
            )
        })
}

async fn resolve_cowork_repo(
    pool: &PgPool,
    user_id_raw: &str,
    headers: &HeaderMap,
) -> Result<PathBuf, Response> {
    let user_id_str = shared::normalize_user_id(user_id_raw);
    let user_id = UserId::new(user_id_str);
    let platform = detect_platform(headers);

    repositories::marketplace_git::get_or_generate_cowork_repo(pool, &user_id, platform)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, user_id = %user_id, "Failed to generate cowork marketplace repo");
            shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to generate cowork marketplace",
            )
        })
}

#[derive(serde::Deserialize, Default, Debug)]
pub struct InfoRefsQuery {
    service: Option<String>,
}

pub async fn marketplace_git_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, file_path)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo_path = match resolve_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    serve_git_file(&repo_path, &file_path, &query).await
}

async fn smart_info_refs(repo_path: &PathBuf) -> Response {
    let output = match tokio::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc", "--advertise-refs"])
        .arg(repo_path)
        .output()
        .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::error!(error = %e, "Failed to run git upload-pack --advertise-refs");
            return shared::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "git upload-pack failed",
            );
        }
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(stderr = %stderr, "git upload-pack --advertise-refs failed");
        return shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed");
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

pub async fn git_upload_pack_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, _path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_path = match resolve_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    run_upload_pack(&repo_path, &body).await
}

pub async fn cowork_git_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, file_path)): Path<(String, String)>,
    Query(query): Query<InfoRefsQuery>,
    headers: HeaderMap,
) -> Response {
    let repo_path = match resolve_cowork_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    serve_git_file(&repo_path, &file_path, &query).await
}

pub async fn cowork_upload_pack_handler(
    State(pool): State<Arc<PgPool>>,
    Path((user_id_raw, _path)): Path<(String, String)>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let repo_path = match resolve_cowork_repo(&pool, &user_id_raw, &headers).await {
        Ok(p) => p,
        Err(r) => return r,
    };
    run_upload_pack(&repo_path, &body).await
}

async fn serve_git_file(repo_path: &PathBuf, file_path: &str, query: &InfoRefsQuery) -> Response {
    if file_path == GIT_INFO_REFS {
        if let Some(ref service) = query.service {
            if service == GIT_UPLOAD_PACK {
                return smart_info_refs(repo_path).await;
            }
        }
    }

    let target = repo_path.join(file_path);

    if !target.starts_with(repo_path) {
        return StatusCode::FORBIDDEN.into_response();
    }

    let Ok(content) = std::fs::read(&target) else {
        return StatusCode::NOT_FOUND.into_response();
    };

    let ext = std::path::Path::new(file_path)
        .extension()
        .and_then(|e| e.to_str());
    let content_type = if file_path == GIT_HEAD || file_path == GIT_INFO_REFS {
        "text/plain"
    } else if ext == Some("pack") {
        "application/x-git-packed-objects"
    } else if ext == Some("idx") {
        "application/x-git-packed-objects-toc"
    } else {
        "application/octet-stream"
    };

    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, content_type)],
        content,
    )
        .into_response()
}

async fn run_upload_pack(repo_path: &PathBuf, body: &Bytes) -> Response {
    match run_upload_pack_inner(repo_path, body).await {
        Ok(stdout) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "application/x-git-upload-pack-result"),
                (header::CACHE_CONTROL, "no-cache"),
            ],
            stdout,
        )
            .into_response(),
        Err(r) => r,
    }
}

async fn run_upload_pack_inner(repo_path: &PathBuf, body: &Bytes) -> Result<Vec<u8>, Response> {
    let mut child = tokio::process::Command::new("git")
        .args(["upload-pack", "--stateless-rpc"])
        .arg(repo_path)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to spawn git upload-pack");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed")
        })?;

    if let Some(mut stdin) = child.stdin.take() {
        use tokio::io::AsyncWriteExt;
        stdin.write_all(body).await.map_err(|e| {
            tracing::error!(error = %e, "Failed to write to git upload-pack stdin");
            shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed")
        })?;
    }

    let output = child.wait_with_output().await.map_err(|e| {
        tracing::error!(error = %e, "Failed to wait for git upload-pack");
        shared::error_response(StatusCode::INTERNAL_SERVER_ERROR, "git upload-pack failed")
    })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::error!(stderr = %stderr, "git upload-pack --stateless-rpc failed");
        return Err(shared::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "git upload-pack failed",
        ));
    }

    Ok(output.stdout)
}
