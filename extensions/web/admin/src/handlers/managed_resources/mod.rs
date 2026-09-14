//! Owner-authenticated source and revision authoring; no implicit activation.

use super::evaluation_experiments::require_write_origin;
use crate::error::AdminResult;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use std::sync::Arc;

use axum::{Extension, Json};
use serde::Deserialize;
use systemprompt::identifiers::{
    ManagedReconciliationId, ManagedResourceId, ManagedSourceId, ResourceRevisionId,
    SourceSnapshotId, WithdrawalProposalId,
};
use systemprompt::marketplace::managed::{
    ConflictDecision, GitSyncRequest, NewResource, NewRevision, ReconciliationRequest,
    RevisionFiles, RevisionManifest, SnapshotProvenance, SourceSpec,
};

mod publication;
pub(crate) mod workspace;
pub(crate) use publication::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSource {
    name: String,
    specification: SourceSpec,
}

pub(crate) async fn create_source(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<CreateSource>,
) -> AdminResult<(StatusCode, Json<ManagedSourceId>)> {
    require_write_origin(&headers)?;
    let id = state
        .repository
        .register_source(&state.owner, &input.name, &input.specification)
        .await?;
    Ok((StatusCode::CREATED, Json(id)))
}

pub(crate) async fn source(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
) -> AdminResult<Json<SourceSpec>> {
    Ok(Json(state.repository.get_source(&state.owner, &id).await?))
}

pub(crate) async fn capture_snapshot(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<SnapshotProvenance>,
) -> AdminResult<(StatusCode, Json<SourceSnapshotId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .capture_snapshot(&state.owner, &id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn sync_git(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<GitSyncRequest>,
) -> AdminResult<Json<systemprompt::marketplace::managed::GitSyncResult>> {
    require_write_origin(&headers)?;
    if input.source_id != id {
        return Err(crate::error::AdminError::Conflict(
            "Source path and body differ".to_owned(),
        ));
    }
    let specification = state.repository.get_source(&state.owner, &id).await?;
    let credential_reference = match specification {
        SourceSpec::Git {
            credential_reference,
            ..
        } => credential_reference,
        _ => None,
    };
    let credential = credential_reference
        .as_deref()
        .map(|name| {
            systemprompt::config::SecretsBootstrap::get()
                .map_err(crate::error::AdminError::internal)?
                .get(name)
                .cloned()
                .ok_or_else(|| {
                    crate::error::AdminError::Unavailable(format!(
                        "Git credential reference '{name}' is unresolved"
                    ))
                })
        })
        .transpose()?;
    Ok(Json(
        state
            .repository
            .sync_git_source_with_credential(&state.owner, &input, credential.as_deref())
            .await?,
    ))
}

pub(crate) async fn begin_reconciliation(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<ReconciliationRequest>,
) -> AdminResult<(
    StatusCode,
    Json<systemprompt::marketplace::managed::ReconciliationRecord>,
)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .begin_reconciliation(&state.owner, &input)
                .await?,
        ),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ResolveConflict {
    path: String,
    resolution: systemprompt::marketplace::managed::ConflictResolution,
    resolved_digest: Option<String>,
}

pub(crate) async fn resolve_conflict(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedReconciliationId>,
    headers: HeaderMap,
    Json(input): Json<ResolveConflict>,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .repository
        .resolve_reconciliation_conflict(
            &state.owner,
            &id,
            &ConflictDecision {
                path: &input.path,
                resolution: input.resolution,
                resolved_digest: input.resolved_digest.as_deref(),
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompleteReconciliation {
    revision_id: ResourceRevisionId,
}

pub(crate) async fn complete_reconciliation(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedReconciliationId>,
    headers: HeaderMap,
    Json(input): Json<CompleteReconciliation>,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .repository
        .complete_reconciliation(&state.owner, &user.user_id, &id, &input.revision_id)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn withdrawal_proposals(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
) -> AdminResult<Json<Vec<systemprompt::marketplace::managed::WithdrawalProposal>>> {
    Ok(Json(
        state
            .repository
            .list_withdrawal_proposals(&state.owner)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecideWithdrawal {
    approved: bool,
}

pub(crate) async fn decide_withdrawal(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<WithdrawalProposalId>,
    headers: HeaderMap,
    Json(input): Json<DecideWithdrawal>,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .repository
        .decide_withdrawal_proposal(&state.owner, &user.user_id, &id, input.approved)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn bind_resource(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewResource>,
) -> AdminResult<(StatusCode, Json<ManagedResourceId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(state.repository.bind_resource(&state.owner, &input).await?),
    ))
}

pub(crate) async fn create_revision(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewRevision>,
) -> AdminResult<(StatusCode, Json<ResourceRevisionId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .create_revision(&state.owner, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn revision(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<Json<RevisionManifest>> {
    Ok(Json(
        state.repository.get_revision(&state.owner, &id).await?,
    ))
}

pub(crate) async fn files(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<([(&'static str, &'static str); 2], Json<RevisionFiles>)> {
    Ok((
        [
            ("cache-control", "no-store"),
            ("x-content-type-options", "nosniff"),
        ],
        Json(
            state
                .repository
                .get_revision_files(&state.owner, &id)
                .await?,
        ),
    ))
}
