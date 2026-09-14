//! Systemprompt campaign presentation delegates policy and persistence to core.

use axum::Extension;
use axum::extract::{Form, Path};
use axum::http::HeaderMap;
use axum::response::{Redirect, Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::evaluation::campaigns::repository::CampaignRecord;
use systemprompt::evaluation::campaigns::{CampaignPolicy, OptimizationObjective};
use systemprompt::identifiers::{
    EvalCampaignId, EvalExperimentId, ManagedResourceId, ResourceRevisionId,
};

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::evaluation_state::EvaluationState;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

#[derive(Serialize)]
struct CampaignDashboard {
    page: &'static str,
    title: &'static str,
    campaigns: Vec<CampaignView>,
    templates: Vec<systemprompt::evaluation::experiments::records::ExperimentRecord>,
    resources: Vec<systemprompt::marketplace::managed::ResourceSummary>,
    operation_key: String,
}

#[derive(Serialize)]
struct CampaignView {
    #[serde(flatten)]
    campaign: CampaignRecord,
    experiments: Vec<EvalExperimentId>,
    candidates: Vec<systemprompt::marketplace::managed::RevisionSummary>,
}

pub(crate) async fn page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Extension(managed): Extension<Arc<ManagedState>>,
) -> AdminHtmlResult<Response> {
    require_admin(&user)?;
    let mut campaigns = Vec::new();
    for campaign in state.campaigns.list(&state.owner, None).await? {
        campaigns.push(CampaignView {
            experiments: state
                .campaigns
                .list_experiments(&state.owner, &campaign.id)
                .await?,
            candidates: managed
                .repository
                .list_revisions(&managed.owner, &campaign.policy.resource_id, 0)
                .await?,
            campaign,
        });
    }
    let data = CampaignDashboard {
        page: "analysis-campaigns",
        title: "Optimization campaigns",
        campaigns,
        templates: state.experiments.list(&state.owner).await?,
        resources: managed.repository.list_resources(&managed.owner, 0).await?,
        operation_key: uuid::Uuid::new_v4().to_string(),
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-campaigns",
        &data,
        &user,
        &marketplace,
    ))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignLaunchForm {
    template_id: EvalExperimentId,
    candidate_revision_id: ResourceRevisionId,
    operation_key: String,
}

pub(crate) async fn launch(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalCampaignId>,
    headers: HeaderMap,
    Form(form): Form<CampaignLaunchForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    let campaign = state.campaigns.get(&state.owner, &id).await?;
    let mut spec = state
        .experiments
        .get(&state.owner, &form.template_id)
        .await?
        .experiment
        .spec;
    if spec.variants.len() != 2 {
        return Err(
            AdminError::BadRequest("Choose a paired experiment template".to_owned()).into(),
        );
    }
    spec.variants[0].skill_bundle_digest = state
        .optimization
        .register_workspace(&state.owner, &campaign.policy.baseline_revision_id)
        .await?;
    spec.variants[1].skill_bundle_digest = state
        .optimization
        .register_workspace(&state.owner, &form.candidate_revision_id)
        .await?;
    spec.claim_independent_improvement = false;
    let mut cases = Vec::new();
    for case in &spec.cases {
        if let systemprompt::evaluation::experiments::resources::ResourceContent::Case(content) =
            state.revisions.get(&state.owner, case).await?
            && content.partition
                == systemprompt::evaluation::experiments::resources::Partition::Development
        {
            cases.push(case.clone());
        }
    }
    spec.cases = cases;
    let experiment = state
        .optimization
        .launch(
            &state.owner,
            &user.user_id,
            &systemprompt::evaluation::repository::experiments::CampaignExperiment {
                campaign_id: id,
                idempotency_key: form.operation_key,
                spec,
            },
        )
        .await?;
    Ok(Redirect::to(&format!(
        "/admin/analysis/evaluations/{experiment}"
    )))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CampaignForm {
    name: String,
    baseline: String,
    budget_dollars: String,
    objective: OptimizationObjective,
    minimum_quality_milli: u32,
    minimum_pairs: u32,
    maximum_iterations: u32,
    #[serde(default)]
    automatic: bool,
    operation_key: String,
}

pub(crate) async fn create(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Extension(managed): Extension<Arc<ManagedState>>,
    headers: HeaderMap,
    Form(form): Form<CampaignForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    let (resource, baseline) = form
        .baseline
        .split_once('|')
        .ok_or_else(|| AdminError::BadRequest("Choose an imported baseline".to_owned()))?;
    let resource_id = ManagedResourceId::new(resource);
    let baseline_revision_id = ResourceRevisionId::new(baseline);
    if managed
        .repository
        .revision_resource(&managed.owner, &baseline_revision_id)
        .await?
        != resource_id
    {
        return Err(
            AdminError::BadRequest("Baseline does not belong to this resource".to_owned()).into(),
        );
    }
    let cap = dollars_to_microdollars(&form.budget_dollars)?;
    let budget_id = state
        .budgets
        .create_shared(
            &state.owner,
            &format!("campaign:{}", form.operation_key),
            cap,
        )
        .await?;
    let policy = CampaignPolicy {
        name: form.name,
        resource_id,
        baseline_revision_id,
        budget_id,
        objective: form.objective,
        minimum_quality_milli: form.minimum_quality_milli,
        minimum_pairs: form.minimum_pairs,
        maximum_iterations: form.maximum_iterations,
        automatic: form.automatic,
    };
    state
        .campaigns
        .create(&state.owner, &user.user_id, &form.operation_key, &policy)
        .await?;
    Ok(Redirect::to("/admin/analysis/campaigns"))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AttachForm {
    experiment_id: EvalExperimentId,
}

pub(crate) async fn attach(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path(id): Path<EvalCampaignId>,
    headers: HeaderMap,
    Form(form): Form<AttachForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user)?;
    crate::handlers::evaluation_experiments::require_write_origin(&headers)?;
    state
        .optimization
        .attach(&state.owner, &user.user_id, &id, &form.experiment_id)
        .await?;
    Ok(Redirect::to("/admin/analysis/campaigns"))
}

pub(crate) async fn report(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<EvaluationState>>,
    Path((id, experiment)): Path<(EvalCampaignId, EvalExperimentId)>,
) -> Result<impl axum::response::IntoResponse, AdminError> {
    require_admin(&user)?;
    let report = state
        .optimization
        .report(&state.owner, &id, &experiment)
        .await?;
    Ok((
        [
            ("cache-control", "no-store"),
            (
                "content-disposition",
                "attachment; filename=campaign-report.json",
            ),
        ],
        axum::Json(report),
    ))
}

fn require_admin(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    Ok(())
}

fn dollars_to_microdollars(value: &str) -> Result<i64, AdminError> {
    let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
    let invalid = || {
        AdminError::BadRequest(
            "Enter a positive USD budget with at most two decimal places".to_owned(),
        )
    };
    if whole.is_empty()
        || !whole.bytes().all(|byte| byte.is_ascii_digit())
        || fraction.len() > 2
        || !fraction.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(invalid());
    }
    let dollars = whole.parse::<i64>().map_err(|_error| invalid())?;
    let cents = format!("{fraction:0<2}")
        .parse::<i64>()
        .map_err(|_error| invalid())?;
    dollars
        .checked_mul(1_000_000)
        .and_then(|amount| amount.checked_add(cents * 10_000))
        .filter(|amount| *amount > 0)
        .ok_or_else(invalid)
}
