use std::sync::Arc;

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use systemprompt::identifiers::UserId;

use crate::admin::activity::{self, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{MarketplaceChangelogEntry, MarketplaceUploadResponse, NewChangelogEntry};

fn extract_and_parse_archive(
    body: &Bytes,
    base_skills_dir: &std::path::Path,
) -> Result<Vec<crate::admin::types::ParsedSkill>, Box<Response>> {
    let tmp_dir = repositories::marketplace_sync::extract_archive(body).map_err(|e| {
        tracing::warn!(error = %e, "Failed to extract uploaded archive");
        Box::new(super::error_response(
            StatusCode::BAD_REQUEST,
            &e.to_string(),
        ))
    })?;

    repositories::marketplace_sync::parse_skills_from_directory(tmp_dir.path(), base_skills_dir)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to parse skills from archive");
            Box::new(super::error_response(
                StatusCode::BAD_REQUEST,
                &format!("Failed to parse skills: {e}"),
            ))
        })
}

pub async fn create_version_and_prune(
    pool: &PgPool,
    user_id: &UserId,
    version_type: &str,
    snapshot_path: &str,
    snapshot: &serde_json::Value,
) -> Result<(crate::admin::types::MarketplaceVersion, i32), Response> {
    let latest_version =
        repositories::marketplace_versions::get_latest_version_number(pool, user_id)
            .await
            .map_err(|e| {
                tracing::error!(error = %e, "Failed to get latest version number");
                super::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get version info",
                )
            })?;
    let new_version = latest_version + 1;

    let version = repositories::marketplace_versions::create_marketplace_version(
        pool,
        user_id,
        new_version,
        version_type,
        snapshot_path,
        snapshot,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to create version record");
        super::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create version",
        )
    })?;

    match repositories::marketplace_versions::prune_old_versions(pool, user_id, 3).await {
        Ok(old_paths) => {
            for path in old_paths {
                repositories::marketplace_sync::delete_snapshot_file(&path);
            }
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to prune old versions (non-fatal)");
        }
    }

    Ok((version, new_version))
}

fn build_changelog(
    diff: &crate::admin::types::SyncDiff,
    user_id: &UserId,
    version_id: &str,
) -> Vec<NewChangelogEntry> {
    let mut entries = Vec::new();
    for skill in &diff.added {
        entries.push(NewChangelogEntry {
            user_id: user_id.clone(),
            version_id: version_id.to_string(),
            action: "added".to_string(),
            skill_id: skill.skill_id.clone(),
            skill_name: skill.name.clone(),
            detail: "new skill added".to_string(),
        });
    }
    for (skill, detail) in &diff.updated {
        entries.push(NewChangelogEntry {
            user_id: user_id.clone(),
            version_id: version_id.to_string(),
            action: "updated".to_string(),
            skill_id: skill.skill_id.clone(),
            skill_name: skill.name.clone(),
            detail: detail.clone(),
        });
    }
    for (skill_id, skill_name) in &diff.deleted {
        entries.push(NewChangelogEntry {
            user_id: user_id.clone(),
            version_id: version_id.to_string(),
            action: "deleted".to_string(),
            skill_id: skill_id.clone(),
            skill_name: skill_name.clone(),
            detail: "skill removed".to_string(),
        });
    }
    entries
}

async fn apply_changes_and_respond(
    pool: &PgPool,
    user_id: &UserId,
    diff: &crate::admin::types::SyncDiff,
    changelog_entries: &[NewChangelogEntry],
    new_version: i32,
) -> Response {
    if let Err(e) = repositories::marketplace_sync::apply_sync_diff(pool, user_id, diff).await {
        tracing::error!(error = %e, "Failed to apply sync diff");
        return super::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to apply changes");
    }

    let changelog = insert_changelog(pool, changelog_entries).await;
    invalidate_cache(user_id);

    Json(MarketplaceUploadResponse {
        version_number: new_version,
        skills_added: diff.added.len(),
        skills_updated: diff.updated.len(),
        skills_deleted: diff.deleted.len(),
        changelog,
    })
    .into_response()
}

async fn insert_changelog(
    pool: &PgPool,
    entries: &[NewChangelogEntry],
) -> Vec<MarketplaceChangelogEntry> {
    repositories::marketplace_versions::insert_changelog_entries(pool, entries)
        .await
        .unwrap_or_else(|e| {
            tracing::warn!(error = %e, "Failed to insert changelog entries (non-fatal)");
            Vec::new()
        })
}

fn invalidate_cache(user_id: &UserId) {
    if let Err(e) = repositories::marketplace_sync::invalidate_git_cache(user_id) {
        tracing::warn!(error = %e, "Failed to invalidate git cache (non-fatal)");
    }
}

async fn snapshot_and_save(
    pool: &PgPool,
    user_id: &UserId,
    body: &Bytes,
) -> Result<(serde_json::Value, String), Response> {
    let snapshot = repositories::marketplace_sync::snapshot_current_skills(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to snapshot current skills");
            super::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to snapshot current state",
            )
        })?;

    let snapshot_path = repositories::marketplace_sync::save_upload_archive(user_id, 0, body)
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to save upload archive");
            super::error_response(StatusCode::INTERNAL_SERVER_ERROR, "Failed to save archive")
        })?;

    Ok((snapshot, snapshot_path))
}

pub async fn marketplace_upload_handler(
    State(pool): State<Arc<PgPool>>,
    Path(user_id_raw): Path<String>,
    headers: HeaderMap,
    body: Bytes,
) -> Response {
    let user_id_str = user_id_raw
        .strip_suffix(".git")
        .unwrap_or(&user_id_raw)
        .to_string();
    let user_id = UserId::new(user_id_str.clone());

    if let Err(r) = super::authenticate(&headers, &user_id_str) {
        return *r;
    }

    let services_path = match super::get_services_path() {
        Ok(p) => p,
        Err(r) => return *r,
    };

    let uploaded_skills = match extract_and_parse_archive(&body, &services_path.join("skills")) {
        Ok(s) => s,
        Err(r) => return *r,
    };

    let (snapshot, snapshot_path) = match snapshot_and_save(&pool, &user_id, &body).await {
        Ok(v) => v,
        Err(r) => return r,
    };

    let (version, new_version) = match create_version_and_prune(
        pool.as_ref(),
        &user_id,
        "upload",
        &snapshot_path,
        &snapshot,
    )
    .await
    {
        Ok(v) => v,
        Err(r) => return r,
    };

    let diff =
        match repositories::marketplace_sync::compute_skill_diff(&pool, &user_id, &uploaded_skills)
            .await
        {
            Ok(d) => d,
            Err(e) => {
                tracing::error!(error = %e, "Failed to compute skill diff");
                return super::error_response(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to compute changes",
                );
            }
        };

    let changelog_entries = build_changelog(&diff, &user_id, &version.id);

    let uid = user_id.clone();
    let ver = new_version;
    let p = Arc::clone(&pool);
    tokio::spawn(async move {
        activity::record(&p, NewActivity::marketplace_uploaded(&uid, ver)).await;
    });

    apply_changes_and_respond(
        pool.as_ref(),
        &user_id,
        &diff,
        &changelog_entries,
        new_version,
    )
    .await
}
