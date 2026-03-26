use std::sync::Arc;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use sqlx::PgPool;

use crate::admin::activity::{self, NewActivity};
use crate::admin::repositories;
use crate::admin::types::{MarketplaceRestoreResponse, NewChangelogEntry};

async fn restore_skills_and_log(
    pool: &PgPool,
    user_id: &str,
    target_version: &crate::admin::types::MarketplaceVersion,
    new_version_record_id: String,
) -> Result<usize, Response> {
    let skills_restored = repositories::marketplace_sync::restore_skills_from_snapshot(
        pool,
        user_id,
        &target_version.skills_snapshot,
    )
    .await
    .map_err(|e| {
        tracing::error!(error = %e, "Failed to restore skills from snapshot");
        super::error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to restore skills",
        )
    })?;

    let _ = repositories::marketplace_versions::insert_changelog_entries(
        pool,
        &[NewChangelogEntry {
            user_id: user_id.to_string(),
            version_id: new_version_record_id,
            action: "restored".to_string(),
            skill_id: "*".to_string(),
            skill_name: format!("Restored from version {}", target_version.version_number),
            detail: format!(
                "Restored {} skills from version {}",
                skills_restored, target_version.version_number
            ),
        }],
    )
    .await;

    if let Err(e) = repositories::marketplace_sync::invalidate_git_cache(user_id) {
        tracing::warn!(error = %e, "Failed to invalidate git cache (non-fatal)");
    }

    Ok(skills_restored)
}

async fn load_target_version(
    pool: &PgPool,
    user_id: &str,
    version_id: &str,
) -> Result<crate::admin::types::MarketplaceVersion, Response> {
    match repositories::marketplace_versions::get_marketplace_version(pool, user_id, version_id)
        .await
    {
        Ok(Some(v)) => Ok(v),
        Ok(None) => Err(super::error_response(
            StatusCode::NOT_FOUND,
            "Version not found",
        )),
        Err(e) => {
            tracing::error!(error = %e, "Failed to load version");
            Err(super::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to load version",
            ))
        }
    }
}

async fn snapshot_current(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<serde_json::Value, Response> {
    repositories::marketplace_sync::snapshot_current_skills(pool, user_id)
        .await
        .map_err(|e| {
            tracing::error!(error = %e, "Failed to snapshot current skills before restore");
            super::error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "Failed to snapshot current state",
            )
        })
}

pub(crate) async fn marketplace_restore_handler(
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

    let target_version = match load_target_version(pool.as_ref(), &user_id, &version_id).await {
        Ok(v) => v,
        Err(r) => return r,
    };

    let current_snapshot = match snapshot_current(&pool, &user_id).await {
        Ok(s) => s,
        Err(r) => return r,
    };

    let restore_path = format!("restore:v{}", target_version.version_number);
    let (new_version_record, new_version) = match super::upload::create_version_and_prune(
        pool.as_ref(),
        &user_id,
        "restore",
        &restore_path,
        &current_snapshot,
    )
    .await
    {
        Ok(v) => v,
        Err(r) => return r,
    };

    let skills_restored = match restore_skills_and_log(
        pool.as_ref(),
        &user_id,
        &target_version,
        new_version_record.id,
    )
    .await
    {
        Ok(count) => count,
        Err(r) => return r,
    };

    let uid = user_id.clone();
    let ver = target_version.version_number;
    let p = pool.clone();
    tokio::spawn(async move {
        activity::record(&p, NewActivity::marketplace_restored(&uid, ver)).await;
    });

    Json(MarketplaceRestoreResponse {
        restored_version: target_version.version_number,
        new_version,
        skills_restored,
    })
    .into_response()
}
