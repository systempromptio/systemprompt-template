//! Super-admin publication, distribution, installation and rollback review.

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::evaluation_experiments::require_write_origin;
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::{Extension, Form, State};
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::sync::Arc;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};
use systemprompt::marketplace::managed::{PublicationAction, PublicationRequest};

#[derive(Debug, FromRow, Serialize)]
struct LifecycleRow {
    resource_id: String,
    resource_key: String,
    publication_id: String,
    review_id: String,
    generation: i64,
    action: String,
    revision_id: Option<String>,
    bundle_digest: Option<String>,
    reviewer_id: String,
    limitations: String,
    distribution_status: String,
    installation_receipts: i64,
}

#[derive(Serialize)]
struct DistributionView {
    publication_id: String,
    generation: i64,
    status: String,
    claimed_at: chrono::DateTime<chrono::Utc>,
    error: Option<String>,
}

#[derive(Serialize)]
struct InstallationReceiptView {
    id: String,
    installation_id: String,
    publication_id: String,
    generation: i64,
    bundle_digest: String,
    installed_manifest: String,
    client_evidence: String,
    verified_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Serialize)]
struct LifecycleContext {
    page: &'static str,
    title: &'static str,
    rows: Vec<LifecycleRow>,
    withdrawals: Vec<systemprompt::marketplace::managed::WithdrawalProposal>,
    distributions: Vec<DistributionView>,
    receipts: Vec<InstallationReceiptView>,
    can_manage: bool,
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(managed): Extension<Arc<ManagedState>>,
    State(pool): State<Arc<PgPool>>,
) -> AdminHtmlResult<Response> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for shared publication evidence".to_owned(),
        )
        .into());
    }
    let rows = sqlx::query_as!(LifecycleRow, r#"SELECT m.id AS resource_id,m.resource_key,p.id AS publication_id,p.review_id,p.generation,p.action,p.revision_id,p.bundle_digest,r.reviewer_id,r.limitations,COALESCE(d.status,CASE WHEN o.delivered_at IS NOT NULL THEN 'distributed' ELSE 'approved' END) AS "distribution_status!",count(i.id)::BIGINT AS "installation_receipts!" FROM managed_publications p JOIN managed_resources m ON m.id=p.resource_id JOIN managed_publication_reviews r ON r.id=p.review_id LEFT JOIN managed_distribution_outbox o ON o.publication_id=p.id LEFT JOIN managed_distribution_deliveries d ON d.outbox_id=o.id LEFT JOIN managed_installation_receipts i ON i.publication_id=p.id WHERE p.owner_id=$1 GROUP BY m.id,m.resource_key,p.id,p.review_id,p.generation,p.action,p.revision_id,p.bundle_digest,r.reviewer_id,r.limitations,d.status,o.delivered_at ORDER BY p.created_at DESC"#,
        managed.owner.as_str()).fetch_all(&*pool).await?;
    let withdrawals = managed
        .repository
        .list_withdrawal_proposals(&managed.owner)
        .await?;
    let distributions = managed
        .repository
        .list_distribution_status(&managed.owner)
        .await?
        .into_iter()
        .map(|row| DistributionView {
            publication_id: row.publication_id,
            generation: row.generation,
            status: row.status,
            claimed_at: row.claimed_at,
            error: row.error,
        })
        .collect();
    let receipts = managed
        .repository
        .list_installation_receipts(&managed.owner, None)
        .await?
        .into_iter()
        .map(|row| InstallationReceiptView {
            id: row.id.to_string(),
            installation_id: row.installation_id,
            publication_id: row.publication_id.to_string(),
            generation: row.generation,
            bundle_digest: row.bundle_digest.as_str().to_owned(),
            installed_manifest: serde_json::to_string_pretty(&row.installed_manifest)
                .unwrap_or_else(|_| "[]".to_owned()),
            client_evidence: serde_json::to_string_pretty(&row.client_evidence)
                .unwrap_or_else(|_| "{}".to_owned()),
            verified_at: row.verified_at,
        })
        .collect();
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-publications",
        &LifecycleContext {
            page: "analysis-publications",
            title: "Publication and installation",
            rows,
            withdrawals,
            distributions,
            receipts,
            can_manage: user.is_admin,
        },
        &user,
        &marketplace,
    ))
}

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
    Ok(Redirect::to("/admin/analysis/publications"))
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
    Extension(evaluations): Extension<Arc<EvaluationState>>,
    headers: HeaderMap,
    Form(form): Form<ReviewForm>,
) -> AdminHtmlResult<impl IntoResponse> {
    if !user.is_admin {
        return Err(AdminError::Forbidden("Administrator review required".to_owned()).into());
    }
    require_write_origin(&headers)?;
    let action = match form.action.as_str() {
        "initial_adoption" => PublicationAction::InitialAdoption,
        "publish_improvement" => PublicationAction::PublishImprovement,
        "withdraw" => PublicationAction::Withdraw,
        "rollback" => PublicationAction::Rollback,
        _ => return Err(AdminError::BadRequest("Unknown publication action".to_owned()).into()),
    };
    let comparison_evidence = serde_json::from_str(&form.comparison_evidence)
        .map_err(|_error| AdminError::BadRequest("Comparison evidence must be JSON".to_owned()))?;
    let decision = managed
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
    crate::handlers::managed_resources::workspace::register_published_workspace(
        &managed,
        &evaluations,
        &managed.owner,
        &decision,
    )
    .await?;
    Ok(Redirect::to("/admin/analysis/publications"))
}
