//! Row assembly for runs, results, and golden-set cases on the Evals page.
//!
//! Split from `view.rs` to keep both under the module size ceiling; shared
//! numeric and string formatting lives in [`super::format`].

use crate::repositories::evals::EvalRunKind;
use crate::repositories::evals::cases::EvalCaseRow;
use crate::repositories::evals::results::{DimensionScores, EvalResultRow};
use crate::repositories::evals::runs::EvalRunRow;

use super::BASE_URL;
use super::context_runs::{CaseRowView, DimensionView, ResultRowView, RunRowView};
use super::format::{format_cost, local_time, score_pct, short_id};

pub(super) fn run_rows(runs: &[EvalRunRow]) -> Vec<RunRowView> {
    runs.iter().map(run_row).collect()
}

// Why: Runs of one kind only, for the tabs that show a single eval type.
pub(super) fn run_rows_of_kind(runs: &[EvalRunRow], kind: EvalRunKind) -> Vec<RunRowView> {
    runs.iter()
        .filter(|r| r.kind == kind.as_str())
        .map(run_row)
        .collect()
}

pub(super) fn run_row(r: &EvalRunRow) -> RunRowView {
    RunRowView {
        id: r.id.clone(),
        short_id: short_id(&r.id),
        kind: r.kind.clone(),
        status: r.status.clone(),
        is_running: r.status == "running",
        is_failed: r.status == "failed",
        judge_model: r.judge_model.clone(),
        sample_size: r.sample_size,
        scored_count: r.scored_count,
        failed_count: r.failed_count,
        mean_score_display: r
            .mean_score
            .map_or_else(|| "—".to_owned(), |m| format!("{m:.2}")),
        cost_display: format_cost(r.cost_microdollars),
        created_by: r.created_by.clone(),
        created_at_local: local_time(r.created_at),
        detail_url: format!("{BASE_URL}/runs/{}", r.id),
    }
}

pub(super) fn result_rows(results: &[EvalResultRow]) -> Vec<ResultRowView> {
    results.iter().map(result_row).collect()
}

pub(super) fn result_row(r: &EvalResultRow) -> ResultRowView {
    let score = r.overall_score.unwrap_or(0);
    ResultRowView {
        id: r.id.clone(),
        run_id: r.run_id.clone(),
        ai_request_id: r.ai_request_id.clone(),
        case_id: r.case_id.clone(),
        model: r.model.clone(),
        provider: r.provider.clone(),
        score_display: r
            .overall_score
            .map_or_else(|| "—".to_owned(), |s| format!("{s}/5")),
        score_pct: score_pct(f64::from(score)),
        verdict: r.verdict.clone(),
        is_pass: r.verdict == "pass",
        is_partial: r.verdict == "partial",
        is_fail: r.verdict == "fail",
        rationale: r.rationale.clone().unwrap_or_default(),
        flags: r.flags.clone(),
        has_flags: !r.flags.is_empty(),
        dimensions: dimension_views(&r.dimension_scores),
        prompt_excerpt: r.prompt_excerpt.clone().unwrap_or_default(),
        response_excerpt: r.response_excerpt.clone().unwrap_or_default(),
        latency_ms: r.latency_ms,
        created_at_local: local_time(r.created_at),
        promote_id: r.ai_request_id.clone(),
    }
}

fn dimension_views(scores: &DimensionScores) -> Vec<DimensionView> {
    scores
        .labelled()
        .into_iter()
        .filter_map(|(label, score)| {
            let score = i64::from(score?);
            Some(DimensionView {
                label,
                score,
                pct: score_pct(score as f64),
            })
        })
        .collect()
}

pub(super) fn case_rows(cases: &[EvalCaseRow]) -> Vec<CaseRowView> {
    cases
        .iter()
        .map(|c| CaseRowView {
            id: c.id.clone(),
            name: c.name.clone(),
            baseline_model: c.baseline_model.clone().unwrap_or_else(|| "—".to_owned()),
            expectation: c.expectation.clone().unwrap_or_default(),
            has_expectation: c.expectation.is_some(),
            created_at_local: local_time(c.created_at),
        })
        .collect()
}
