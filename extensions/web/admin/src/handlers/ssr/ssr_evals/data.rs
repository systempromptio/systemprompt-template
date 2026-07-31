//! Data collection for the Evals page.
//!
//! Same shape as the Inference Requests page: resolve the window (auto-widen
//! until it has rows), then fan every repository call out in parallel and
//! collapse each `Result` into a logged default, so one failing query degrades
//! a panel instead of the page.
//!
//! Only the active tab's queries run. The two summaries behind the KPI strip
//! are the exception — every tab shows one of them — and everything else is
//! gated on [`EvalsTab`], so opening the page costs three queries rather than
//! twelve. A tab that did not fetch a section leaves it empty, which is the
//! same state the template's `{{#if}}` guards already handle for a genuinely
//! empty window.

use std::sync::Arc;

use sqlx::PgPool;

use crate::repositories::analytics::request_stats::{
    LatencyBucket, RequestStats, TimeBucket, get_request_stats, list_latency_histogram,
    list_request_timeseries,
};
use crate::repositories::evals::cases::{EvalCaseRow, list_cases};
use crate::repositories::evals::distribution::{
    ModelDistributionRow, PromptTopicRow, UserDistributionRow, list_model_distribution,
    list_prompt_topics, list_user_distribution,
};
use crate::repositories::evals::results::{
    EvalPairRow, EvalResultRow, ResultFilter, list_recent_pairs, list_recent_results,
};
use crate::repositories::evals::runs::{EvalRunRow, list_recent_runs};
use crate::repositories::evals::scores::{
    EvalScoreSummary, ModelScoreRow, ModelWinRateRow, get_eval_score_summary, list_model_scores,
    list_model_win_rates,
};
use crate::util::time_range::{
    TimeRange, TimeRangePreset, TimeRangeQuery, count_requests_in_range, parse_time_range,
    preset_to_range,
};

use super::EvalsQuery;
use super::context::EvalsTab;

const USER_LIMIT: i64 = 15;
const TOPIC_LIMIT: i64 = 15;
const RUN_LIMIT: i64 = 15;
const RESULT_LIMIT: i64 = 50;
const PAIR_LIMIT: i64 = 30;

pub(super) async fn resolve_range(
    pool: &PgPool,
    query: &EvalsQuery,
) -> (TimeRange, Option<&'static str>) {
    let user_picked_range = query.preset.is_some() || (query.from.is_some() && query.to.is_some());
    let initial_range = parse_time_range(&TimeRangeQuery {
        from: query.from.clone(),
        to: query.to.clone(),
        preset: query.preset.clone(),
    });

    if user_picked_range {
        return (initial_range, None);
    }

    let mut chosen = initial_range;
    let mut widened: Option<&'static str> = None;
    for (label, preset) in [
        ("24h", TimeRangePreset::Hours24),
        ("7d", TimeRangePreset::Days7),
        ("30d", TimeRangePreset::Days30),
    ] {
        let candidate = preset_to_range(preset);
        let count = count_requests_in_range(pool, candidate).await.unwrap_or(0);
        if count > 0 {
            chosen = candidate;
            widened = if label == "24h" { None } else { Some(label) };
            break;
        }
    }
    (chosen, widened)
}

pub(super) fn range_from_strings(from: Option<&str>, to: Option<&str>) -> TimeRange {
    parse_time_range(&TimeRangeQuery {
        from: from.map(str::to_owned),
        to: to.map(str::to_owned),
        preset: None,
    })
}

#[derive(Default)]
pub(super) struct EvalsData {
    pub stats: RequestStats,
    pub hist: Vec<LatencyBucket>,
    pub series: Vec<TimeBucket>,
    pub models: Vec<ModelDistributionRow>,
    pub model_scores: Vec<ModelScoreRow>,
    pub users: Vec<UserDistributionRow>,
    pub topics: Vec<PromptTopicRow>,
    pub scores: EvalScoreSummary,
    pub win_rates: Vec<ModelWinRateRow>,
    pub pairs: Vec<EvalPairRow>,
    pub runs: Vec<EvalRunRow>,
    pub results: Vec<EvalResultRow>,
    pub cases: Vec<EvalCaseRow>,
}

pub(super) async fn fetch_evals_data(
    pool: &Arc<PgPool>,
    range: TimeRange,
    tab: EvalsTab,
    filter: &ResultFilter,
) -> EvalsData {
    let (stats, scores) = tokio::join!(
        get_request_stats(pool, range),
        get_eval_score_summary(pool, range),
    );

    let mut data = EvalsData {
        stats: unwrap_or_default(stats, "get_request_stats"),
        scores: unwrap_or_default(scores, "get_eval_score_summary"),
        ..EvalsData::default()
    };

    match tab {
        EvalsTab::Overview => {
            let (hist, series, runs) = tokio::join!(
                list_latency_histogram(pool, range),
                list_request_timeseries(pool, range),
                list_recent_runs(pool, range, RUN_LIMIT),
            );
            data.hist = unwrap_or_empty(hist, "list_latency_histogram");
            data.series = unwrap_or_empty(series, "list_request_timeseries");
            data.runs = unwrap_or_empty(runs, "list_recent_runs");
        },
        EvalsTab::Traffic => {
            let (models, model_scores, users, topics) = tokio::join!(
                list_model_distribution(pool, range),
                list_model_scores(pool, range),
                list_user_distribution(pool, range, USER_LIMIT),
                list_prompt_topics(pool, range, TOPIC_LIMIT),
            );
            data.models = unwrap_or_empty(models, "list_model_distribution");
            data.model_scores = unwrap_or_empty(model_scores, "list_model_scores");
            data.users = unwrap_or_empty(users, "list_user_distribution");
            data.topics = unwrap_or_empty(topics, "list_prompt_topics");
        },
        EvalsTab::Judge => {
            let (models, results) = tokio::join!(
                list_model_distribution(pool, range),
                list_recent_results(pool, range, RESULT_LIMIT, filter),
            );
            data.models = unwrap_or_empty(models, "list_model_distribution");
            data.results = unwrap_or_empty(results, "list_recent_results");
        },
        EvalsTab::HeadToHead => {
            let (models, win_rates, pairs, cases) = tokio::join!(
                list_model_distribution(pool, range),
                list_model_win_rates(pool, range),
                list_recent_pairs(pool, range, PAIR_LIMIT),
                list_cases(pool, false),
            );
            data.models = unwrap_or_empty(models, "list_model_distribution");
            data.win_rates = unwrap_or_empty(win_rates, "list_model_win_rates");
            data.pairs = unwrap_or_empty(pairs, "list_recent_pairs");
            data.cases = unwrap_or_empty(cases, "list_cases");
        },
        EvalsTab::GoldenSet => {
            let (models, cases, runs) = tokio::join!(
                list_model_distribution(pool, range),
                list_cases(pool, false),
                list_recent_runs(pool, range, RUN_LIMIT),
            );
            data.models = unwrap_or_empty(models, "list_model_distribution");
            data.cases = unwrap_or_empty(cases, "list_cases");
            data.runs = unwrap_or_empty(runs, "list_recent_runs");
        },
    }

    data
}

fn unwrap_or_empty<T>(res: Result<Vec<T>, sqlx::Error>, what: &str) -> Vec<T> {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "evals page query failed");
        Vec::new()
    })
}

fn unwrap_or_default<T: Default>(res: Result<T, sqlx::Error>, what: &str) -> T {
    res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, query = what, "evals page query failed");
        T::default()
    })
}
