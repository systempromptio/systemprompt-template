//! Reference-free judge run over live gateway traffic.
//!
//! Split from `mod.rs` so the module stays under the size ceiling; the run
//! lifecycle (open/close, ids, verdict parsing) lives in [`super::lifecycle`].

use sqlx::PgPool;

use crate::repositories::evals::sampling::EvalCandidate;
use crate::repositories::evals::{EvalRunKind, EvalVerdict, results, sampling};

use super::deterministic::PrePass;
use super::lifecycle::{OpenRunParams, RunTally, close_run, new_id, open_run, parse_verdict};
use super::{
    EXCERPT_CHARS, EvalError, EvalRunOutcome, EvalRunRequest, MAX_JUDGE_CHARS, MAX_SAMPLE_SIZE,
    deterministic, extract, judge,
};
use judge::JudgeConfig;
use sqlx::types::Json;

use crate::repositories::evals::results::DimensionScores;

pub(crate) async fn run_judge_eval(
    pool: &PgPool,
    request: &EvalRunRequest,
) -> Result<EvalRunOutcome, EvalError> {
    let run_id = new_id("evrun");
    let config = request.judge_config(&run_id);
    let sample_size = request.sample_size.clamp(1, MAX_SAMPLE_SIZE);

    let mut filter = request.filter.clone();
    // Why: never re-score what this judge model has already scored — a second
    // run over the same window should extend coverage, not duplicate it.
    filter.skip_judged_by = Some(config.model.clone());

    let candidates =
        sampling::list_eval_candidates(pool, &filter, request.range, sample_size).await?;
    if candidates.is_empty() {
        return Err(EvalError::NoCandidates);
    }

    open_run(OpenRunParams {
        pool,
        run_id: &run_id,
        kind: EvalRunKind::Judge,
        config: &config,
        request,
        sample_size: candidates.len(),
    })
    .await?;

    let mut tally = RunTally::default();
    for candidate in candidates {
        score_one(ScoreParams {
            pool,
            config: &config,
            run_id: &run_id,
            candidate: &candidate,
            tally: &mut tally,
        })
        .await?;
    }

    close_run(pool, &run_id, tally.scored, tally.failed, tally.cost).await?;

    Ok(EvalRunOutcome {
        run_id,
        scored: tally.scored,
        failed: tally.failed,
        cost_microdollars: tally.cost,
    })
}

struct ScoreParams<'a> {
    pool: &'a PgPool,
    config: &'a JudgeConfig,
    run_id: &'a str,
    candidate: &'a EvalCandidate,
    tally: &'a mut RunTally,
}

async fn score_one(params: ScoreParams<'_>) -> Result<(), sqlx::Error> {
    let candidate = params.candidate;
    let pre = deterministic::run_pre_pass(candidate);
    let excerpts = Excerpts::from_pre_pass(&pre);

    if let Some((verdict, rationale)) = pre.short_circuit {
        insert_row(
            params.pool,
            RowParams {
                run_id: params.run_id,
                candidate,
                excerpts: &excerpts,
                verdict,
                overall_score: matches!(verdict, EvalVerdict::Fail).then_some(1),
                dimension_scores: Json(DimensionScores::default()),
                rationale: &rationale,
                flags: &pre.flags,
                judge_cost: 0,
            },
        )
        .await?;
        params.tally.scored += 1;
        return Ok(());
    }

    let (Some(prompt), Some(answer)) = (pre.prompt.as_deref(), pre.answer.as_deref()) else {
        params.tally.failed += 1;
        return Ok(());
    };

    let judged = judge::judge_answer(
        params.pool,
        params.config,
        &extract::truncate_for_judge(prompt, MAX_JUDGE_CHARS),
        &extract::truncate_for_judge(answer, MAX_JUDGE_CHARS),
    )
    .await;

    let Some(judged) = judged else {
        params.tally.failed += 1;
        return Ok(());
    };

    params.tally.cost += judged.cost_microdollars;

    insert_row(
        params.pool,
        RowParams {
            run_id: params.run_id,
            candidate,
            excerpts: &excerpts,
            verdict: parse_verdict(&judged.verdict.verdict),
            overall_score: Some(i32::from(judged.verdict.overall_score)),
            dimension_scores: judged.verdict.dimension_scores(),
            rationale: &judged.verdict.rationale,
            flags: &merge_flags(&pre.flags, &judged.verdict.flags),
            judge_cost: judged.cost_microdollars,
        },
    )
    .await?;
    params.tally.scored += 1;
    Ok(())
}

fn merge_flags(pre: &[String], judged: &[String]) -> Vec<String> {
    let mut flags = pre.to_vec();
    for f in judged {
        if !flags.contains(f) {
            flags.push(f.clone());
        }
    }
    flags
}

struct Excerpts {
    prompt: Option<String>,
    response: Option<String>,
}

impl Excerpts {
    fn from_pre_pass(pre: &PrePass) -> Self {
        Self {
            prompt: pre
                .prompt
                .as_deref()
                .map(|p| extract::excerpt(p, EXCERPT_CHARS)),
            response: pre
                .answer
                .as_deref()
                .map(|a| extract::excerpt(a, EXCERPT_CHARS)),
        }
    }
}

struct RowParams<'a> {
    run_id: &'a str,
    candidate: &'a EvalCandidate,
    excerpts: &'a Excerpts,
    verdict: EvalVerdict,
    overall_score: Option<i32>,
    dimension_scores: Json<DimensionScores>,
    rationale: &'a str,
    flags: &'a [String],
    judge_cost: i64,
}

async fn insert_row(pool: &PgPool, params: RowParams<'_>) -> Result<(), sqlx::Error> {
    let candidate = params.candidate;
    results::insert_result(
        pool,
        results::InsertResultParams {
            id: &new_id("evres"),
            run_id: params.run_id,
            ai_request_id: Some(candidate.ai_request_id.as_str()),
            case_id: None,
            user_id: Some(&candidate.user_id),
            session_id: candidate.session_id.as_ref(),
            provider: &candidate.provider,
            model: &candidate.model,
            overall_score: params.overall_score,
            dimension_scores: params.dimension_scores,
            verdict: params.verdict,
            rationale: Some(params.rationale),
            flags: params.flags,
            prompt_excerpt: params.excerpts.prompt.as_deref(),
            response_excerpt: params.excerpts.response.as_deref(),
            latency_ms: candidate.latency_ms,
            cost_microdollars: candidate.cost_microdollars,
            judge_cost_microdollars: params.judge_cost,
        },
    )
    .await
}
