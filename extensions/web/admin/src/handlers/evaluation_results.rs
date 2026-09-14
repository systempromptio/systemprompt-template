//! Owner-scoped execution manifests and verified artifact downloads.

use super::evaluation_experiments::require_write_origin;
use crate::error::AdminResult;
use crate::routes::evaluation_state::EvaluationState;
use crate::types::UserContext;
use axum::extract::Path;
use axum::http::{HeaderMap, StatusCode};
use axum::response::IntoResponse;
use axum::{Extension, Json};
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::evaluation::repository::experiments::{
    ApprovalDecision, ApprovalVerdict, SuggestionRequest,
};
use systemprompt::identifiers::{EvalApprovalId, EvalExecutionId, EvalExperimentId};

pub(crate) async fn evidence(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExecutionId>,
) -> AdminResult<impl IntoResponse> {
    let evidence = state.evidence.get(&state.owner, &id).await?;
    Ok(([("cache-control", "no-store")], Json(evidence)))
}

pub(crate) async fn artifacts(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExecutionId>,
) -> AdminResult<impl IntoResponse> {
    let artifacts = state.evidence.get_artifacts(&state.owner, &id).await?;
    Ok(([("cache-control", "no-store")], Json(artifacts)))
}

pub(crate) async fn comparison(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExperimentId>,
) -> AdminResult<Json<systemprompt::evaluation::repository::experiments::ComparisonReport>> {
    Ok(Json(state.lifecycle.comparison(&state.owner, &id).await?))
}

pub(crate) async fn comparison_markdown(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExperimentId>,
) -> AdminResult<impl IntoResponse> {
    let report = state.lifecycle.comparison(&state.owner, &id).await?;
    let cost_per_success = report
        .cost_per_verified_success_microdollars
        .map_or_else(|| "undefined".to_owned(), |value| format!("{value} µ$"));
    let body = format!(
        "# Evaluation comparison {}\n\n- Attempted: {}\n- Completed: {}\n- Hard failures: {}\n- Unscored: {}\n- Verified successes: {}\n- Total failed-attempt-inclusive spend: {} µ$\n- Cost per verified success: {}\n- Accounting coverage: {}/{}\n\nThis bounded comparison is evidence for the recorded matrix only; before/after associations are not causal.\n",
        report.experiment_id,
        report.attempted,
        report.completed,
        report.hard_failures,
        report.unscored,
        report.verified_successes,
        report.attempted_cost_microdollars,
        cost_per_success,
        report.accounting_complete,
        report.accounting_total
    );
    Ok((
        [
            ("content-type", "text/markdown; charset=utf-8"),
            (
                "content-disposition",
                "attachment; filename=evaluation-comparison.md",
            ),
        ],
        body,
    ))
}

pub(crate) async fn suggestion(
    Extension(_user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Json(input): Json<SuggestionRequest>,
) -> AdminResult<(
    StatusCode,
    Json<systemprompt::identifiers::EvalSuggestionId>,
)> {
    require_write_origin(&headers)?;
    Ok((
        StatusCode::CREATED,
        Json(
            state
                .lifecycle
                .create_suggestion(&state.owner, &input)
                .await?,
        ),
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct DecideApproval {
    approve: bool,
    observed_precondition_digest: String,
}

pub(crate) async fn decide_approval(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalApprovalId>,
    headers: HeaderMap,
    Json(input): Json<DecideApproval>,
) -> AdminResult<StatusCode> {
    require_write_origin(&headers)?;
    let decision = if input.approve {
        ApprovalDecision::Approve
    } else {
        ApprovalDecision::Deny
    };
    state
        .lifecycle
        .decide_approval(
            &state.owner,
            &ApprovalVerdict {
                actor: &user.user_id,
                approval: &id,
                decision,
                observed_precondition: &input.observed_precondition_digest,
            },
        )
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
