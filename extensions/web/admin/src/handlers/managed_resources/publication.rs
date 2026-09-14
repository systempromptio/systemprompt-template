//! Candidate comparison, publication, distribution and installation handlers.

use std::sync::Arc;

use axum::extract::{Path, Query};
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::Deserialize;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};
use systemprompt::marketplace::managed::{
    DistributionClaim, InstallationReceipt, InstallationReceiptRequest,
};

use super::super::evaluation_experiments::require_write_origin;
use crate::error::AdminResult;
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;

pub(crate) async fn create_candidate(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
    headers: HeaderMap,
    Json(input): Json<systemprompt::marketplace::managed::TextCandidate>,
) -> AdminResult<(StatusCode, Json<ResourceRevisionId>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .create_text_candidate(&state.owner, &id, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn comparison(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((baseline, candidate)): Path<(ResourceRevisionId, ResourceRevisionId)>,
) -> AdminResult<Json<systemprompt::marketplace::managed::RevisionComparison>> {
    Ok(Json(
        state
            .repository
            .compare_revisions(&state.owner, &baseline, &candidate)
            .await?,
    ))
}

pub(crate) async fn bundle(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminResult<axum::response::Response> {
    use axum::response::IntoResponse;
    use systemprompt::marketplace::managed::AssetDigest;
    let bundle = state
        .repository
        .get_revision_bundle(&state.owner, &id)
        .await?;
    let bytes = bundle.canonical_bytes()?;
    let digest = AssetDigest::of(&bytes);
    Ok((
        [
            ("content-type", "application/json".to_owned()),
            (
                "content-disposition",
                "attachment; filename=revision-bundle.json".to_owned(),
            ),
            ("cache-control", "no-store".to_owned()),
            ("x-content-type-options", "nosniff".to_owned()),
            ("x-content-sha256", digest.as_str().to_owned()),
        ],
        bytes,
    )
        .into_response())
}

pub(crate) async fn publish(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Extension(evaluations): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<systemprompt::marketplace::managed::PublicationRequest>,
) -> AdminResult<(
    StatusCode,
    Json<systemprompt::marketplace::managed::PublicationDecision>,
)> {
    if !user.is_admin {
        return Err(crate::error::AdminError::Forbidden(
            "Administrator publication review required".to_owned(),
        ));
    }
    require_write_origin(&headers)?;
    let decision = state
        .repository
        .review_and_publish(&state.owner, &user.user_id, &input)
        .await?;
    super::workspace::register_published_workspace(&state, &evaluations, &state.owner, &decision)
        .await?;
    Ok((StatusCode::CREATED, Json(decision)))
}

pub(crate) async fn resolution(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((kind, key)): Path<(systemprompt::marketplace::managed::ResourceKind, String)>,
) -> AdminResult<Json<systemprompt::marketplace::managed::ManagedResolution>> {
    let resolver =
        systemprompt::marketplace::managed::ManagedResourceResolver::new(state.repository.clone());
    Ok(Json(
        resolver.resolve_state(&state.owner, kind, &key).await?,
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct PublicationBundleQuery {
    digest: String,
}

pub(crate) async fn publication_bundle(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((resource, generation)): Path<(ManagedResourceId, i64)>,
    Query(query): Query<PublicationBundleQuery>,
) -> AdminResult<axum::response::Response> {
    use axum::response::IntoResponse;
    use systemprompt::marketplace::managed::AssetDigest;
    let digest = AssetDigest::try_from(query.digest)?;
    let bundle = state
        .repository
        .get_publication_bundle(&state.owner, &resource, generation, &digest)
        .await?;
    let bytes = bundle.canonical_bytes()?;
    Ok((
        [
            ("content-type", "application/json".to_owned()),
            (
                "content-disposition",
                format!("attachment; filename=managed-bundle-{generation}.json"),
            ),
            ("cache-control", "private, no-store".to_owned()),
            ("x-content-type-options", "nosniff".to_owned()),
            ("x-content-sha256", digest.as_str().to_owned()),
            ("x-publication-generation", generation.to_string()),
        ],
        bytes,
    )
        .into_response())
}

pub(crate) async fn publication_history(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(resource): Path<ManagedResourceId>,
) -> AdminResult<Json<Vec<systemprompt::marketplace::managed::PublicationHistoryEntry>>> {
    Ok(Json(
        state
            .repository
            .list_publication_history(&state.owner, &resource)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ClaimDistribution {
    claim_token: String,
}

pub(crate) async fn claim_distribution(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<ClaimDistribution>,
) -> AdminResult<Json<Option<DistributionClaim>>> {
    require_write_origin(&headers)?;
    Ok(Json(
        state
            .repository
            .claim_distribution(&state.owner, &input.claim_token)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CompleteDistribution {
    claim: DistributionClaim,
    delivered: bool,
    error: Option<String>,
}

pub(crate) async fn complete_distribution(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<CompleteDistribution>,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .repository
        .complete_distribution(
            &state.owner,
            &input.claim,
            input.delivered,
            input.error.as_deref(),
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn installation_receipt(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Json(input): Json<InstallationReceiptRequest>,
) -> AdminResult<(StatusCode, Json<InstallationReceipt>)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .repository
                .record_installation(&state.owner, &input)
                .await?,
        ),
    ))
}

pub(crate) async fn distribution_status(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
) -> AdminResult<Json<Vec<systemprompt::marketplace::managed::DistributionStatus>>> {
    Ok(Json(
        state
            .repository
            .list_distribution_status(&state.owner)
            .await?,
    ))
}

#[derive(Debug, Deserialize)]
pub(crate) struct ReceiptQuery {
    resource_id: Option<ManagedResourceId>,
}

pub(crate) async fn installation_receipts(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Query(query): Query<ReceiptQuery>,
) -> AdminResult<Json<Vec<InstallationReceipt>>> {
    Ok(Json(
        state
            .repository
            .list_installation_receipts(&state.owner, query.resource_id.as_ref())
            .await?,
    ))
}
