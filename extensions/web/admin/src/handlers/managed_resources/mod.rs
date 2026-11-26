//! Owner-authenticated source and revision authoring; no implicit activation.

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};
use crate::handlers::shared::require_write_origin;
use crate::handlers::ssr::page::Caller;
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
pub(crate) use publication::*;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateSource {
    name: String,
    specification: SourceSpec,
}

pub(crate) async fn create_source(
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<CreateSource>,
) -> AdminResult<(StatusCode, Json<ManagedSourceId>)> {
    require_write_origin(&env, &headers)?;
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
    caller: Caller,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<SnapshotProvenance>,
) -> AdminResult<(StatusCode, Json<SourceSnapshotId>)> {
    let Caller { env, .. } = caller;
    require_write_origin(&env, &headers)?;
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
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<GitSyncRequest>,
) -> AdminResult<Json<serde_json::Value>> {
    require_write_origin(&env, &headers)?;
    if input.source_id != id {
        return Err(AdminError::Conflict(
            "Source path and body differ".to_owned(),
        ));
    }
    let synced = env
        .git_sync()
        .sync(&systemprompt::traits::GitSourceSyncRequest {
            source_id: input.source_id,
            resource_id: input.resource_id,
            upstream_root: input.upstream_root,
            upstream_base_revision_id: input.upstream_base_revision_id,
        })
        .await?;
    // JSON: the sync outcome relayed in the console's tagged shape
    Ok(Json(match synced {
        systemprompt::traits::GitSourceSynced::Incoming {
            snapshot_id,
            revision_id,
            commit,
            reconciliation_id,
        } => serde_json::json!({
            "result": "incoming",
            "snapshot_id": snapshot_id,
            "revision_id": revision_id,
            "commit": commit,
            "reconciliation_id": reconciliation_id,
        }),
        systemprompt::traits::GitSourceSynced::WithdrawalProposed {
            snapshot_id,
            proposal_id,
            commit,
        } => serde_json::json!({
            "result": "withdrawal_proposed",
            "snapshot_id": snapshot_id,
            "proposal_id": proposal_id,
            "commit": commit,
        }),
    }))
}


pub(crate) async fn begin_reconciliation(
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<ReconciliationRequest>,
) -> AdminResult<(
    StatusCode,
    Json<systemprompt::marketplace::managed::ReconciliationRecord>,
)> {
    require_write_origin(&env, &headers)?;
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
    caller: Caller,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedReconciliationId>,
    headers: HeaderMap,
    Json(input): Json<ResolveConflict>,
) -> AdminResult<StatusCode> {
    let Caller { env, .. } = caller;
    require_write_origin(&env, &headers)?;
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
    caller: Caller,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ManagedReconciliationId>,
    headers: HeaderMap,
    Json(input): Json<CompleteReconciliation>,
) -> AdminResult<StatusCode> {
    let Caller { env, user } = caller;
    require_write_origin(&env, &headers)?;
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
    caller: Caller,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<WithdrawalProposalId>,
    headers: HeaderMap,
    Json(input): Json<DecideWithdrawal>,
) -> AdminResult<StatusCode> {
    let Caller { env, user } = caller;
    require_write_origin(&env, &headers)?;
    state
        .repository
        .decide_withdrawal_proposal(&state.owner, &user.user_id, &id, input.approved)
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn bind_resource(
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewResource>,
) -> AdminResult<(StatusCode, Json<ManagedResourceId>)> {
    require_write_origin(&env, &headers)?;
    Ok((
        StatusCode::CREATED,
        Json(state.repository.bind_resource(&state.owner, &input).await?),
    ))
}

pub(crate) async fn create_revision(
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<NewRevision>,
) -> AdminResult<(StatusCode, Json<ResourceRevisionId>)> {
    require_write_origin(&env, &headers)?;
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
