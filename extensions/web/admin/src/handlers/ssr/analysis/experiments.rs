//! Owner-scoped experiment records expose progress without inferring quality.

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::{Form, Path};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::evaluation::experiments::records::{ExperimentRecord, ExperimentStatus};
use systemprompt::evaluation::repository::experiments::{ApprovalDecision, ApprovalVerdict};
use systemprompt::identifiers::{
    EvalApprovalId, EvalBudgetId, EvalExecutionId, EvalExperimentId, EvalRevisionId,
};

#[derive(Serialize)]
struct ExperimentView {
    id: EvalExperimentId,
    name: String,
    status: ExperimentStatus,
    cases: usize,
    variants: usize,
    repetitions: u32,
    cap: String,
    settled: String,
    reserved: String,
    frozen: bool,
}

impl From<ExperimentRecord> for ExperimentView {
    fn from(row: ExperimentRecord) -> Self {
        Self {
            id: row.id,
            name: row.spec.name,
            status: row.status,
            cases: row.spec.cases.len(),
            variants: row.spec.variants.len(),
            repetitions: row.spec.repetitions,
            cap: super::dollars(row.accounting.cap),
            settled: super::dollars(row.accounting.settled),
            reserved: super::dollars(row.accounting.reserved),
            frozen: row.accounting.frozen,
        }
    }
}

pub(crate) async fn list_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let experiments = state
        .experiments
        .list(&state.owner)
        .await
        .map_err(AdminError::from)?;
    let rendered = ExperimentIndexContext {
        page: "analysis-evaluations",
        title: "Evaluations",
        experiments: experiments.into_iter().map(ExperimentView::from).collect(),
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluations",
        &rendered,
        &user,
        &marketplace,
    ))
}

#[derive(Serialize)]
struct ExecutionView {
    id: EvalExecutionId,
    case_revision: EvalRevisionId,
    variant: i32,
    repetition: i32,
    status: systemprompt::evaluation::experiments::records::ExecutionStatus,
    summary: Option<String>,
}

pub(crate) async fn detail_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExperimentId>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let detail = state
        .experiments
        .get(&state.owner, &id)
        .await
        .map_err(AdminError::from)?;
    let specification =
        serde_json::to_string_pretty(&detail.experiment.spec).map_err(AdminError::internal)?;
    let approvals = sqlx::query_as!(ApprovalView, r#"SELECT a.id,a.execution_id AS "execution_id!: EvalExecutionId",a.status,a.precondition_digest,a.operation,a.expires_at FROM eval_execution_approvals a JOIN eval_executions x ON x.id=a.execution_id JOIN eval_experiments e ON e.id=x.experiment_id WHERE e.owner_id=$1 AND e.id=$2 ORDER BY a.requested_at"#,
        state.owner.as_str(), id.as_str()).fetch_all(&state.pool).await?;
    let rendered = ExperimentEvidenceContext {
        page: "analysis-evaluations",
        title: "Experiment evidence",
        experiment: detail.experiment.into(),
        specification,
        executions: detail
            .executions
            .into_iter()
            .map(|row| ExecutionView {
                id: row.id,
                case_revision: row.case_revision_id,
                variant: row.variant_index + 1,
                repetition: row.repetition + 1,
                status: row.status,
                summary: row.result.map(|result| result.summary),
            })
            .collect(),
        approvals,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-experiment",
        &rendered,
        &user,
        &marketplace,
    ))
}

fn require_console(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for shared evaluations".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct ExperimentIndexContext {
    page: &'static str,
    title: &'static str,
    experiments: Vec<ExperimentView>,
}

#[derive(Serialize)]
struct ExperimentEvidenceContext {
    page: &'static str,
    title: &'static str,
    experiment: ExperimentView,
    specification: String,
    executions: Vec<ExecutionView>,
    approvals: Vec<ApprovalView>,
}

#[derive(Serialize, sqlx::FromRow)]
struct ApprovalView {
    id: String,
    execution_id: EvalExecutionId,
    status: String,
    precondition_digest: String,
    // JSON: retained approval operations are deliberately arbitrary tool payloads.
    operation: serde_json::Value,
    expires_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LaunchForm {
    budget_id: EvalBudgetId,
    idempotency_key: String,
    spec_json: String,
}

#[derive(Serialize)]
struct PreflightContext {
    page: &'static str,
    title: &'static str,
    estimate: systemprompt::evaluation::experiments::records::ExperimentPreflight,
    budget_id: EvalBudgetId,
    idempotency_key: String,
    spec_json: String,
}

#[expect(
    clippy::too_many_arguments,
    reason = "Axum supplies six independent typed extractors"
)]
pub(crate) async fn preflight_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Form(form): Form<LaunchForm>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    super::super::super::evaluation_experiments::require_write_origin(&headers)?;
    // Why: lint-ok: error-adapt — malformed form JSON is a bad request.
    let spec = serde_json::from_str(&form.spec_json).map_err(|error| {
        AdminError::BadRequest(format!("Invalid experiment specification: {error}"))
    })?;
    let estimate = state
        .experiments
        .preflight(&state.owner, &form.budget_id, &spec)
        .await
        .map_err(AdminError::from)?;
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-evaluation-preflight",
        &PreflightContext {
            page: "analysis-evaluations",
            title: "Evaluation preflight",
            estimate,
            budget_id: form.budget_id,
            idempotency_key: form.idempotency_key,
            spec_json: form.spec_json,
        },
        &user,
        &marketplace,
    ))
}

pub(crate) async fn launch(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Form(form): Form<LaunchForm>,
) -> AdminHtmlResult<impl IntoResponse> {
    require_console(&user)?;
    super::super::super::evaluation_experiments::require_write_origin(&headers)?;
    // Why: lint-ok: error-adapt — malformed form JSON is a bad request.
    let spec = serde_json::from_str(&form.spec_json).map_err(|error| {
        AdminError::BadRequest(format!("Invalid experiment specification: {error}"))
    })?;
    let id = state
        .experiments
        .create_with_budget(&state.owner, &form.idempotency_key, &form.budget_id, &spec)
        .await
        .map_err(AdminError::from)?;
    Ok(Redirect::to(&format!("/admin/analysis/evaluations/{id}")))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ApprovalForm {
    approval_id: EvalApprovalId,
    decision: String,
    precondition_digest: String,
}

pub(crate) async fn decide_approval(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Form(form): Form<ApprovalForm>,
) -> AdminHtmlResult<impl IntoResponse> {
    require_console(&user)?;
    super::super::super::evaluation_experiments::require_write_origin(&headers)?;
    let decision = match form.decision.as_str() {
        "approve" => ApprovalDecision::Approve,
        "deny" => ApprovalDecision::Deny,
        _ => return Err(AdminError::BadRequest("Unknown approval decision".to_owned()).into()),
    };
    state
        .lifecycle
        .decide_approval(
            &state.owner,
            &ApprovalVerdict {
                actor: &user.user_id,
                approval: &form.approval_id,
                decision,
                observed_precondition: &form.precondition_digest,
            },
        )
        .await
        .map_err(AdminError::from)?;
    Ok(Redirect::to("/admin/analysis/evaluations"))
}

pub(crate) async fn cancel(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalExperimentId>,
    headers: HeaderMap,
) -> AdminHtmlResult<impl IntoResponse> {
    require_console(&user)?;
    super::super::super::evaluation_experiments::require_write_origin(&headers)?;
    state
        .experiments
        .cancel(&state.owner, &id)
        .await
        .map_err(AdminError::from)?;
    Ok(Redirect::to(&format!("/admin/analysis/evaluations/{id}")))
}
