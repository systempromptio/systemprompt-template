//! The two review writes behind the publication page: deciding an upstream
//! withdrawal proposal, and recording a manual review that publishes.

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::shared::require_write_origin;
use crate::routes::managed_state::ManagedState;
use crate::types::UserContext;
use axum::extract::{Extension, Form};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect};
use serde::Deserialize;
use std::sync::Arc;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};
use systemprompt::marketplace::managed::{PublicationAction, PublicationRequest};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct WithdrawalForm {
    proposal_id: String,
    decision: String,
}

pub(crate) async fn decide_withdrawal(
    Extension(user): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<WithdrawalForm>,
) -> AdminHtmlResult<impl IntoResponse> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator review required".to_owned()).into());
    }
    require_write_origin(&headers)?;
    managed
        .repository
        .decide_withdrawal_proposal(
            &managed.owner,
            &user.user_id,
            &systemprompt::identifiers::WithdrawalProposalId::new(form.proposal_id),
            form.decision == "approve",
        )
        .await?;
    Ok(Redirect::to("/admin/analysis/versions"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReviewForm {
    resource_id: ManagedResourceId,
    revision_id: Option<ResourceRevisionId>,
    action: String,
    expected_generation: i64,
    operation_key: String,
    limitations: String,
    comparison_evidence: String,
}

pub(crate) async fn review(
    Extension(user): Extension<UserContext>,
    Extension(managed): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<ReviewForm>,
) -> AdminHtmlResult<impl IntoResponse> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator review required".to_owned()).into());
    }
    require_write_origin(&headers)?;
    let action = match form.action.as_str() {
        "initial_adoption" => PublicationAction::InitialAdoption,
        "withdraw" => PublicationAction::Withdraw,
        "rollback" => PublicationAction::Rollback,
        _ => return Err(AdminError::BadRequest("Unknown publication action".to_owned()).into()),
    };
    let comparison_evidence = serde_json::from_str(&form.comparison_evidence)
        .map_err(|_error| AdminError::BadRequest("Comparison evidence must be JSON".to_owned()))?;
    managed
        .repository
        .review_and_publish(
            &managed.owner,
            &user.user_id,
            &PublicationRequest {
                resource_id: form.resource_id,
                revision_id: form.revision_id,
                action,
                expected_generation: form.expected_generation,
                operation_key: form.operation_key,
                comparison_evidence,
                limitations: form.limitations,
            },
        )
        .await?;
    Ok(Redirect::to("/admin/analysis/versions"))
}
