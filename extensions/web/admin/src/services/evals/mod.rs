//! Evaluation engine.
//!
//! Three run kinds, one shared judge:
//!
//! - [`run_judge_eval`] scores real gateway traffic reference-free.
//! - [`run_replay_eval`] re-sends the golden set and scores the fresh answers.
//! - [`run_pairwise_eval`] puts two models on the same case and picks a winner.
//!
//! Every run writes an `eval_runs` row first and closes it out at the end, so
//! a crashed run is visible as `running` with no completion rather than
//! silently absent. Judge calls go through `AiService`, which means they are
//! themselves governed, audited, and costed — and are excluded from future
//! candidate pools by [`crate::repositories::evals::sampling`].

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

pub(crate) mod deterministic;
pub(crate) mod extract;
pub(crate) mod gateway_client;
pub(crate) mod judge;
pub(crate) mod judge_run;
mod lifecycle;
pub(crate) mod pairwise;
pub(crate) mod replay;
pub(crate) mod rubric;

use crate::repositories::evals::sampling::CandidateFilter;
use crate::repositories::evals::{EvalRunKind, cases, sampling};
use crate::util::time_range::TimeRange;

pub(crate) use judge_run::run_judge_eval;
pub(crate) use lifecycle::{OpenRunParams, RunTally, close_run, new_id, open_run, parse_verdict};

use gateway_client::GatewayCredential;
use judge::JudgeConfig;

pub(crate) const MAX_JUDGE_CHARS: usize = 8_000;
pub(crate) const EXCERPT_CHARS: usize = 240;
// Why: hard ceiling on judge spend — a caller-supplied sample size is clamped
// to this, whatever the form asked for.
pub(crate) const MAX_SAMPLE_SIZE: i64 = 200;

// Why: a bare model id is not enough to place a call, so both halves are
// carried rather than re-deriving the provider from a naming convention.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModelRef {
    pub provider: String,
    pub model: String,
}

impl ModelRef {
    // Why: a slashless input is rejected outright — silently attributing a bare
    // model id to some default provider would bill the wrong upstream.
    #[must_use]
    pub(crate) fn parse(s: &str) -> Option<Self> {
        let (provider, model) = s.split_once('/')?;
        (!provider.is_empty() && !model.is_empty()).then(|| Self {
            provider: provider.to_owned(),
            model: model.to_owned(),
        })
    }

    #[must_use]
    pub(crate) fn as_value(&self) -> String {
        format!("{}/{}", self.provider, self.model)
    }
}

// Why: serialised onto `eval_runs.filter` so a run can be read back later
// without guessing what it covered.
#[derive(Debug, Clone)]
pub(crate) struct EvalRunRequest {
    pub kind: EvalRunKind,
    pub range: TimeRange,
    pub filter: CandidateFilter,
    pub sample_size: i64,
    pub actor: UserId,
    // Why: replay uses the first entry; pairwise requires two distinct entries.
    pub compare_models: Vec<ModelRef>,
    // Why: every model call an eval makes travels under the operator's own
    // gateway credential, so a run can never exceed what its operator could
    // have asked for directly.
    pub credential: GatewayCredential,
    pub judge: ModelRef,
}

impl EvalRunRequest {
    pub(crate) fn judge_config(&self, run_id: &str) -> JudgeConfig {
        JudgeConfig {
            provider: self.judge.provider.clone(),
            model: self.judge.model.clone(),
            actor_user_id: self.actor.clone(),
            run_id: run_id.to_owned(),
            credential: self.credential.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct EvalRunOutcome {
    pub run_id: String,
    pub scored: i32,
    pub failed: i32,
    pub cost_microdollars: i64,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum EvalError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("no candidates matched the requested window and filters")]
    NoCandidates,
    #[error("golden set is empty — promote a request into it first")]
    NoCases,
    #[error("pairwise runs need two distinct models")]
    NeedTwoModels,
}

pub(crate) async fn run_replay_eval(
    pool: &PgPool,
    request: &EvalRunRequest,
) -> Result<EvalRunOutcome, EvalError> {
    let run_id = new_id("evrun");
    let config = request.judge_config(&run_id);

    let case_rows = cases::list_cases(pool, true).await?;
    if case_rows.is_empty() {
        return Err(EvalError::NoCases);
    }

    let target = request
        .compare_models
        .first()
        .cloned()
        .unwrap_or_else(|| request.judge.clone());

    open_run(OpenRunParams {
        pool,
        run_id: &run_id,
        kind: EvalRunKind::Replay,
        config: &config,
        request,
        sample_size: case_rows.len(),
    })
    .await?;

    let outcome = replay::execute_replay(replay::ReplayParams {
        pool,
        config: &config,
        run_id: &run_id,
        cases: &case_rows,
        target: &target,
    })
    .await?;

    close_run(pool, &run_id, outcome.scored, outcome.failed, outcome.cost).await?;

    Ok(EvalRunOutcome {
        run_id,
        scored: outcome.scored,
        failed: outcome.failed,
        cost_microdollars: outcome.cost,
    })
}

pub(crate) async fn run_pairwise_eval(
    pool: &PgPool,
    request: &EvalRunRequest,
) -> Result<EvalRunOutcome, EvalError> {
    if request.compare_models.len() < 2 || request.compare_models[0] == request.compare_models[1] {
        return Err(EvalError::NeedTwoModels);
    }

    let run_id = new_id("evrun");
    let config = request.judge_config(&run_id);

    let case_rows = cases::list_cases(pool, true).await?;
    if case_rows.is_empty() {
        return Err(EvalError::NoCases);
    }

    open_run(OpenRunParams {
        pool,
        run_id: &run_id,
        kind: EvalRunKind::Pairwise,
        config: &config,
        request,
        sample_size: case_rows.len(),
    })
    .await?;

    let outcome = pairwise::execute_pairwise(pairwise::PairwiseParams {
        pool,
        config: &config,
        run_id: &run_id,
        cases: &case_rows,
        model_a: &request.compare_models[0],
        model_b: &request.compare_models[1],
    })
    .await?;

    close_run(pool, &run_id, outcome.scored, outcome.failed, outcome.cost).await?;

    Ok(EvalRunOutcome {
        run_id,
        scored: outcome.scored,
        failed: outcome.failed,
        cost_microdollars: outcome.cost,
    })
}

pub(crate) async fn promote_case(
    pool: &PgPool,
    ai_request_id: &str,
    name: Option<&str>,
    expectation: Option<&str>,
    actor: &UserId,
) -> Result<String, EvalError> {
    let Some(candidate) = sampling::find_candidate_by_id(pool, ai_request_id).await? else {
        return Err(EvalError::NoCandidates);
    };

    let prompt = extract::final_user_prompt(candidate.request_body.as_ref())
        .or_else(|| candidate.request_excerpt.clone())
        .unwrap_or_default();
    let derived_name = name
        .map(str::to_owned)
        .filter(|n| !n.trim().is_empty())
        .unwrap_or_else(|| extract::excerpt(&prompt, 80));

    let case_id = new_id("evcase");
    let body = candidate
        .request_body
        .clone()
        .unwrap_or_else(|| serde_json::json!({ "messages": [] }));

    cases::insert_case(
        pool,
        cases::InsertCaseParams {
            id: &case_id,
            name: &derived_name,
            prompt_body: body,
            source_ai_request_id: Some(candidate.ai_request_id.as_str()),
            expectation,
            baseline_response: baseline_body(&candidate),
            baseline_model: Some(&candidate.model),
            tags: &[],
            created_by: actor.as_str(),
        },
    )
    .await?;

    Ok(case_id)
}

fn baseline_body(candidate: &sampling::EvalCandidate) -> Option<serde_json::Value> {
    if let Some(body) = candidate.response_body.clone() {
        return Some(body);
    }
    let streamed = extract::assistant_answer_from_sse(candidate.response_excerpt.as_deref()?)?;
    Some(serde_json::json!({
        "type": "message",
        "role": "assistant",
        "model": candidate.model,
        "content": [{ "type": "text", "text": streamed.text }],
    }))
}
