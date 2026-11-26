//! The Git source sync a console owner triggers, relayed in the console's
//! tagged outcome shape.

use axum::extract::Path;
use axum::http::HeaderMap;
use axum::{Extension, Json};
use serde::Serialize;
use systemprompt::identifiers::{
    ManagedReconciliationId, ManagedSourceId, ResourceRevisionId, SourceSnapshotId,
    WithdrawalProposalId,
};
use systemprompt::marketplace::managed::GitSyncRequest;

use crate::env::AdminEnv;
use crate::error::{AdminError, AdminResult};
use crate::handlers::shared::require_write_origin;
use crate::types::UserContext;

pub(crate) async fn sync_git(
    Extension(env): Extension<AdminEnv>,
    Extension(_user): Extension<UserContext>,
    Path(id): Path<ManagedSourceId>,
    headers: HeaderMap,
    Json(input): Json<GitSyncRequest>,
) -> AdminResult<Json<GitSyncOutcome>> {
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
    Ok(Json(match synced {
        systemprompt::traits::GitSourceSynced::Incoming {
            snapshot_id,
            revision_id,
            commit,
            reconciliation_id,
        } => GitSyncOutcome::Incoming {
            snapshot_id,
            revision_id,
            commit,
            reconciliation_id,
        },
        systemprompt::traits::GitSourceSynced::WithdrawalProposed {
            snapshot_id,
            proposal_id,
            commit,
        } => GitSyncOutcome::WithdrawalProposed {
            snapshot_id,
            proposal_id,
            commit,
        },
    }))
}

#[derive(Debug, Serialize)]
#[serde(tag = "result", rename_all = "snake_case")]
pub(crate) enum GitSyncOutcome {
    Incoming {
        snapshot_id: SourceSnapshotId,
        revision_id: ResourceRevisionId,
        commit: String,
        reconciliation_id: Option<ManagedReconciliationId>,
    },
    WithdrawalProposed {
        snapshot_id: SourceSnapshotId,
        proposal_id: WithdrawalProposalId,
        commit: String,
    },
}
