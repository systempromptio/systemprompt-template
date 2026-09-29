//! The configuration archive endpoints: export every plane as one zip, stage
//! an uploaded zip, apply a staged plane, discard a stage.
//!
//! An upload writes nothing. It is unpacked under the archive rules, staged
//! in memory, and answered with the preview URL; the preview page reads the
//! stage and shows each plane's drift computed from the uploaded text. Only
//! `…/apply` reaches the database, one plane and one mode at a time (or
//! every plane in the archive at once), and each apply leaves an activity
//! row naming the stage it came from. The write tier and a same-origin
//! check guard everything but the export.

use std::sync::Arc;

use axum::Json;
use axum::body::Bytes;
use axum::extract::{Extension, Path, State};
use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::activity::{self, NewActivity, PlaneApply};
use crate::error::{AdminError, AdminResult};
use crate::handlers::shared::require_write_origin;
use crate::repositories::sync::archive::read::{classify, unpack_zip};
use crate::repositories::sync::archive::staging::{StagedArchive, StagingStore};
use crate::repositories::sync::archive::write::build_export_zip;
use crate::repositories::sync::plane::{
    Actor, ApplyOutcome, DeclarationSource, PlaneDrift, SyncMode,
};
use crate::repositories::sync::registry::{find_plane, planes};
use crate::types::UserContext;

fn preview_url(stage_id: &str) -> String {
    format!("/admin/sync/import/{stage_id}")
}

pub(crate) async fn export_zip_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
) -> AdminResult<Response> {
    let zip = build_export_zip(&pool, user_ctx.user_id.as_str()).await?;
    activity::record(
        &pool,
        NewActivity::configuration_exported(&user_ctx.user_id, zip.planes, zip.rows),
    )
    .await;
    let disposition = format!("attachment; filename=\"{}\"", zip.filename);
    Ok((
        StatusCode::OK,
        [
            (header::CONTENT_TYPE, "application/zip"),
            (header::CONTENT_DISPOSITION, disposition.as_str()),
        ],
        zip.bytes,
    )
        .into_response())
}

#[derive(Debug, Serialize)]
pub(crate) struct StageResponse {
    pub stage_id: String,
    pub preview_url: String,
    pub planes: Vec<&'static str>,
    pub other: usize,
    pub expires_at: String,
}

fn require_admin(user_ctx: &UserContext, headers: &HeaderMap) -> AdminResult<()> {
    if !user_ctx.is_admin {
        return Err(AdminError::Forbidden(
            "Importing configuration needs an administrator.".to_owned(),
        ));
    }
    require_write_origin(headers)
}

pub(crate) async fn stage_import_handler(
    Extension(user_ctx): Extension<UserContext>,
    headers: HeaderMap,
    body: Bytes,
) -> AdminResult<Response> {
    require_admin(&user_ctx, &headers)?;
    let store = StagingStore::global();
    if body.is_empty() {
        return Err(AdminError::BadRequest(
            "upload a zip archive as the request body".to_owned(),
        ));
    }
    let unpacked = unpack_zip(&body)?;
    let staged = classify(unpacked, &planes(), user_ctx.user_id.as_str())?;
    if staged.planes.is_empty() && staged.other.is_empty() {
        return Err(AdminError::Unprocessable(
            "the archive holds no configuration".to_owned(),
        ));
    }
    let response = StageResponse {
        stage_id: staged.id.clone(),
        preview_url: preview_url(&staged.id),
        planes: staged.planes.keys().copied().collect(),
        other: staged.other.len(),
        expires_at: staged.expires_at().to_rfc3339(),
    };
    store.put(staged);
    Ok(Json(response).into_response())
}

#[derive(Debug, Deserialize)]
pub(crate) struct ImportApplyRequest {
    // Why: a plane id, or `all` for every plane the archive holds.
    pub plane: String,
    pub mode: SyncMode,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportApplied {
    pub plane: &'static str,
    pub outcome: ApplyOutcome,
    pub drift_after: PlaneDrift,
}

#[derive(Debug, Serialize)]
pub(crate) struct ImportApplyResponse {
    pub stage_id: String,
    pub mode: SyncMode,
    pub applied: Vec<ImportApplied>,
    pub stage_closed: bool,
}

fn stage_or_404(store: &StagingStore, id: &str) -> AdminResult<StagedArchive> {
    store.get(id).ok_or_else(|| {
        AdminError::NotFound("no staged import with that id, or it expired".to_owned())
    })
}

pub(crate) async fn apply_import_handler(
    State(pool): State<Arc<PgPool>>,
    Extension(user_ctx): Extension<UserContext>,
    Path(stage_id): Path<String>,
    headers: HeaderMap,
    Json(body): Json<ImportApplyRequest>,
) -> AdminResult<Response> {
    require_admin(&user_ctx, &headers)?;
    let store = StagingStore::global();
    let stage = stage_or_404(store, &stage_id)?;
    let chosen: Vec<&'static str> = if body.plane == "all" {
        stage.planes.keys().copied().collect()
    } else {
        let id = stage
            .planes
            .keys()
            .copied()
            .find(|k| *k == body.plane)
            .ok_or_else(|| {
                AdminError::NotFound(format!("the archive holds no '{}' file", body.plane))
            })?;
        vec![id]
    };
    let mut applied = Vec::with_capacity(chosen.len());
    let mut stage_closed = false;
    for id in chosen {
        let plane =
            find_plane(id).ok_or_else(|| AdminError::NotFound(format!("no sync plane '{id}'")))?;
        let text = stage.planes.get(id).map(String::as_str).unwrap_or_default();
        let from = DeclarationSource::Text(text);
        let outcome = plane
            .apply_from(&pool, body.mode, Actor::User(&user_ctx.user_id), from)
            .await?;
        activity::record(
            &pool,
            NewActivity::configuration_imported(
                &user_ctx.user_id,
                &stage_id,
                PlaneApply {
                    plane: plane.id(),
                    label: plane.label(),
                    mode: body.mode,
                    outcome: &outcome,
                    reason: None,
                },
            ),
        )
        .await;
        // Why: after the apply the honest picture is the disk declaration
        // against the database — that is what the next page load shows.
        let drift_after = plane.drift(&pool).await?;
        stage_closed = store.drop_plane(&stage_id, id);
        applied.push(ImportApplied {
            plane: plane.id(),
            outcome,
            drift_after,
        });
    }
    Ok(Json(ImportApplyResponse {
        stage_id,
        mode: body.mode,
        applied,
        stage_closed,
    })
    .into_response())
}

pub(crate) async fn discard_import_handler(
    Extension(user_ctx): Extension<UserContext>,
    Path(stage_id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<Response> {
    require_admin(&user_ctx, &headers)?;
    let store = StagingStore::global();
    stage_or_404(store, &stage_id)?;
    store.remove(&stage_id);
    Ok(StatusCode::NO_CONTENT.into_response())
}
