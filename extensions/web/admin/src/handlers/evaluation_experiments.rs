//! Authenticated experiment authoring and owner-scoped evidence of queued work.

use crate::error::{AdminError, AdminResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::evaluation::experiments::ExperimentSpec;
use systemprompt::evaluation::experiments::records::{
    BudgetRecord, ExperimentDetail, ExperimentPreflight, ExperimentRecord,
};
use systemprompt::evaluation::experiments::resources::ResourceContent;
use systemprompt::identifiers::{EvalBudgetId, EvalExperimentId, EvalRevisionId};

pub(super) fn require_write_origin(headers: &HeaderMap) -> AdminResult<()> {
    let profile = systemprompt::config::ProfileBootstrap::get().map_err(AdminError::internal)?;
    let expected = url::Url::parse(&profile.server.api_external_url)
        .map_err(AdminError::internal)?
        .origin()
        .ascii_serialization();
    if headers.get("origin").and_then(|v| v.to_str().ok()) != Some(expected.as_str()) {
        return Err(AdminError::Forbidden(
            "Same-origin browser request required".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateExperiment {
    idempotency_key: String,
    budget_id: EvalBudgetId,
    spec: ExperimentSpec,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateBudget {
    idempotency_key: String,
    cap_microdollars: i64,
}

#[derive(Debug, Serialize)]
pub(crate) struct BudgetCreated {
    id: EvalBudgetId,
}

#[derive(Debug, Serialize)]
pub(crate) struct ExperimentCreated {
    id: EvalExperimentId,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CreateRevision {
    key: String,
    content: ResourceContent,
}

#[derive(Debug, Serialize)]
pub(crate) struct RevisionCreated {
    id: EvalRevisionId,
}

pub(crate) async fn list(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
) -> AdminResult<Json<Vec<ExperimentRecord>>> {
    Ok(Json(state.experiments.list(&state.owner).await?))
}

pub(crate) async fn show(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ExperimentDetail>> {
    Ok(Json(
        state
            .experiments
            .get(&state.owner, &EvalExperimentId::new(id))
            .await?,
    ))
}

pub(crate) async fn create(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<CreateExperiment>,
) -> AdminResult<(StatusCode, Json<ExperimentCreated>)> {
    require_write_origin(&headers)?;
    let estimate = state
        .experiments
        .preflight(&state.owner, &input.budget_id, &input.spec)
        .await?;
    if !estimate.affordable {
        return Err(AdminError::BadRequest(
            "The complete frozen matrix exceeds the shared budget".to_owned(),
        ));
    }
    let id = state
        .experiments
        .create_with_budget(
            &state.owner,
            &input.idempotency_key,
            &input.budget_id,
            &input.spec,
        )
        .await?;
    Ok((StatusCode::ACCEPTED, Json(ExperimentCreated { id })))
}

pub(crate) async fn preflight(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Json(input): Json<CreateExperiment>,
) -> AdminResult<Json<ExperimentPreflight>> {
    Ok(Json(
        state
            .experiments
            .preflight(&state.owner, &input.budget_id, &input.spec)
            .await?,
    ))
}

pub(crate) async fn create_budget(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<CreateBudget>,
) -> AdminResult<(StatusCode, Json<BudgetCreated>)> {
    require_write_origin(&headers)?;
    let id = state
        .budgets
        .create_shared(&state.owner, &input.idempotency_key, input.cap_microdollars)
        .await?;
    Ok((StatusCode::CREATED, Json(BudgetCreated { id })))
}

pub(crate) async fn get_budget(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
) -> AdminResult<Json<BudgetRecord>> {
    Ok(Json(
        state
            .budgets
            .get(&state.owner, &EvalBudgetId::new(id))
            .await?,
    ))
}

pub(crate) async fn cancel(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
    headers: HeaderMap,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    state
        .experiments
        .cancel(&state.owner, &EvalExperimentId::new(id))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}

pub(crate) async fn create_revision(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<CreateRevision>,
) -> AdminResult<(StatusCode, Json<RevisionCreated>)> {
    require_write_origin(&headers)?;
    let id = state
        .revisions
        .create(&state.owner, &input.key, &input.content)
        .await?;
    Ok((StatusCode::CREATED, Json(RevisionCreated { id })))
}

pub(crate) async fn get_revision(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<String>,
) -> AdminResult<Json<ResourceContent>> {
    Ok(Json(
        state
            .revisions
            .get(&state.owner, &EvalRevisionId::new(id))
            .await?,
    ))
}
