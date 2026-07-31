//! POST actions for the Evals page: launching a run and promoting a case.
//!
//! Both redirect back to the page with a notice rather than rendering, so the
//! browser lands on a GET and a refresh cannot re-fire the run.

use std::sync::Arc;

use axum::Form;
use axum::extract::{Extension, State};
use axum::http::HeaderMap;
use axum::response::Redirect;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::models::Config;

use crate::error::{AdminError, AdminHtmlResult};
use crate::repositories::evals::EvalRunKind;
use crate::repositories::evals::sampling::CandidateFilter;
use crate::services::evals::gateway_client::GatewayCredential;
use crate::services::evals::{self, EvalError, EvalRunOutcome, EvalRunRequest, ModelRef};
use crate::types::UserContext;

use super::context::EvalsTab;
use super::{DEFAULT_SAMPLE_SIZE, data, urls};

#[derive(Debug, Deserialize)]
pub(crate) struct RunEvalForm {
    pub kind: String,
    /// The tab the form was fired from, so the redirect lands back on it.
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
    pub sample_size: Option<i64>,
    pub model: Option<String>,
    pub model_a: Option<String>,
    pub model_b: Option<String>,
    pub judge_model: Option<String>,
}

pub(crate) async fn eval_run_action(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    headers: HeaderMap,
    Form(form): Form<RunEvalForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user_ctx)?;

    let range = data::range_from_strings(form.from.as_deref(), form.to.as_deref());
    let tab = EvalsTab::from_query(form.tab.as_deref()).as_str();

    let credential = match credential_from_request(&headers) {
        Ok(c) => c,
        Err(message) => {
            return Ok(Redirect::to(&urls::redirect_url(
                &range, tab, &message, true,
            )));
        },
    };

    let kind = EvalRunKind::from_str_opt(&form.kind).unwrap_or(EvalRunKind::Judge);
    let compare_models = [
        form.model_a.as_deref(),
        form.model_b.as_deref(),
        form.model.as_deref(),
    ]
    .into_iter()
    .flatten()
    .filter_map(ModelRef::parse)
    .collect::<Vec<_>>();

    let Some(judge) = form.judge_model.as_deref().and_then(ModelRef::parse) else {
        return Ok(Redirect::to(&urls::redirect_url(
            &range,
            tab,
            "Pick a judge model before running an evaluation.",
            true,
        )));
    };

    let request = EvalRunRequest {
        kind,
        range,
        filter: CandidateFilter::default(),
        sample_size: form.sample_size.unwrap_or(DEFAULT_SAMPLE_SIZE),
        actor: user_ctx.user_id.clone(),
        compare_models,
        credential,
        judge,
    };

    let outcome = match request.kind {
        EvalRunKind::Judge => evals::run_judge_eval(&pool, &request).await,
        EvalRunKind::Replay => evals::run_replay_eval(&pool, &request).await,
        EvalRunKind::Pairwise => evals::run_pairwise_eval(&pool, &request).await,
    };

    Ok(Redirect::to(&run_redirect(&range, tab, kind, outcome)))
}

fn credential_from_request(headers: &HeaderMap) -> Result<GatewayCredential, String> {
    let token = crate::handlers::extract_token_from_headers(headers)
        .map_err(|e| format!("Could not read your session token: {e}"))?;
    let claims = systemprompt::security::extract_user_context(&token)
        .map_err(|e| format!("Your session token could not be validated: {e}"))?;
    let config = Config::get().map_err(|e| format!("Configuration unavailable: {e}"))?;

    Ok(GatewayCredential {
        base_url: config.api_internal_url.clone(),
        token,
        session_id: claims.session_id,
    })
}

fn run_redirect(
    range: &crate::util::time_range::TimeRange,
    tab: &str,
    kind: EvalRunKind,
    outcome: Result<EvalRunOutcome, EvalError>,
) -> String {
    match outcome {
        Ok(o) => urls::redirect_url(
            range,
            tab,
            &format!(
                "{} run {} finished: {} scored, {} failed, judge cost ${:.4}.",
                kind.as_str(),
                o.run_id,
                o.scored,
                o.failed,
                o.cost_microdollars as f64 / 1_000_000.0,
            ),
            false,
        ),
        Err(e) => {
            tracing::warn!(error = %e, kind = kind.as_str(), "eval run failed");
            urls::redirect_url(range, tab, &format!("Eval run failed: {e}"), true)
        },
    }
}

#[derive(Debug, Deserialize)]
pub(crate) struct PromoteCaseForm {
    pub ai_request_id: String,
    pub name: Option<String>,
    pub expectation: Option<String>,
    pub tab: Option<String>,
    pub from: Option<String>,
    pub to: Option<String>,
}

pub(crate) async fn eval_promote_case_action(
    Extension(user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
    Form(form): Form<PromoteCaseForm>,
) -> AdminHtmlResult<Redirect> {
    require_admin(&user_ctx)?;

    let range = data::range_from_strings(form.from.as_deref(), form.to.as_deref());
    let tab = EvalsTab::from_query(form.tab.as_deref()).as_str();
    let outcome = evals::promote_case(
        &pool,
        &form.ai_request_id,
        form.name.as_deref(),
        form.expectation.as_deref().filter(|e| !e.trim().is_empty()),
        &user_ctx.user_id,
    )
    .await;

    let url = match outcome {
        Ok(_) => urls::redirect_url(&range, tab, "Added to the golden set.", false),
        Err(e) => {
            tracing::warn!(error = %e, "promoting eval case failed");
            urls::redirect_url(
                &range,
                tab,
                &format!("Could not add to the golden set: {e}"),
                true,
            )
        },
    };
    Ok(Redirect::to(&url))
}

pub(super) fn require_admin(user_ctx: &UserContext) -> Result<(), crate::error::AdminHtmlError> {
    if user_ctx.is_admin {
        Ok(())
    } else {
        Err(AdminError::Forbidden("Admin access required.".to_owned()).into())
    }
}
