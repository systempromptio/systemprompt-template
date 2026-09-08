//! View-model assembly for the Evals page.
//!
//! Pure functions from repository rows to the serde JSON the templates
//! consume. Formatting decisions live here and nowhere else: the templates do
//! no arithmetic, so what a reader sees on the page is exactly what a reader
//! of this file computes.

use std::collections::HashMap;


use crate::repositories::analytics::request_stats::RequestStats;
use crate::repositories::evals::distribution::{
    EvalModelDistributionRow, PromptTopicRow, UserDistributionRow,
};
use crate::repositories::evals::results::{EvalPairRow, ResultFilter};
use crate::repositories::evals::scores::{EvalScoreSummary, ModelScoreRow, ModelWinRateRow};

use super::format::{format_cost, local_time, score_pct, share_pct, truncate};

use super::context::{
    EvalModelMixRowView, EvalModelOptionView, FilterOptionView, PairRowView, ResultFilterView,
    ScoreSummaryView, TopicRowView, TrafficStatsView, UserRowView, WinRateView,
};

pub(super) fn traffic_stats(
    s: &RequestStats,
    models: &[EvalModelDistributionRow],
    users: &[UserDistributionRow],
) -> TrafficStatsView {
    TrafficStatsView {
        total: s.total,
        error_count: s.error_count,
        error_rate_pct: format!("{:.2}", s.error_rate * 100.0),
        p50_latency_ms: s.p50_latency_ms.round() as i64,
        p95_latency_ms: s.p95_latency_ms.round() as i64,
        total_cost_display: format_cost(s.total_cost_microdollars),
        user_count: users.len() as i64,
        model_count: models.len() as i64,
        // Why: The user and model counts are the lengths of the distribution lists,
        // which only the tabs that show those tables pay to fetch. Elsewhere a
        // zero would read as "no users", not "not counted" — so the strip is
        // told to leave the line off rather than print a number it never had.
        has_distribution: !models.is_empty() || !users.is_empty(),
    }
}

pub(super) fn score_summary(s: &EvalScoreSummary, traffic_total: i64) -> ScoreSummaryView {
    let coverage = if traffic_total > 0 {
        s.scored_count as f64 / traffic_total as f64 * 100.0
    } else {
        0.0
    };
    ScoreSummaryView {
        scored_count: s.scored_count,
        mean_score_display: if s.scored_count > 0 {
            format!("{:.2}", s.mean_score)
        } else {
            "—".to_owned()
        },
        mean_score_pct: score_pct(s.mean_score),
        pass_count: s.pass_count,
        partial_count: s.partial_count,
        fail_count: s.fail_count,
        flagged_count: s.flagged_count,
        judge_cost_display: format_cost(s.judge_cost_microdollars),
        coverage_pct: format!("{coverage:.1}"),
        has_scores: s.scored_count > 0,
    }
}


pub(super) fn model_rows(
    models: &[EvalModelDistributionRow],
    scores: &[ModelScoreRow],
    total_requests: i64,
) -> Vec<EvalModelMixRowView> {
    let by_model: HashMap<&str, &ModelScoreRow> =
        scores.iter().map(|s| (s.model.as_str(), s)).collect();

    models
        .iter()
        .map(|m| {
            let score = by_model.get(m.model.as_str());
            let cost_per_request = if m.request_count > 0 {
                m.cost_microdollars / m.request_count
            } else {
                0
            };
            EvalModelMixRowView {
                provider: m.provider.clone(),
                model: m.model.clone(),
                request_count: m.request_count,
                share_pct: share_pct(m.request_count, total_requests),
                user_count: m.user_count,
                error_count: m.error_count,
                tokens_total: m.input_tokens + m.output_tokens,
                cost_display: format_cost(m.cost_microdollars),
                cost_per_request_display: format_cost(cost_per_request),
                p50_latency_ms: m.p50_latency_ms.round() as i64,
                p95_latency_ms: m.p95_latency_ms.round() as i64,
                scored_count: score.map_or(0, |s| s.scored_count),
                mean_score_display: score
                    .filter(|s| s.scored_count > 0)
                    .map_or_else(|| "—".to_owned(), |s| format!("{:.2}", s.mean_score)),
                fail_count: score.map_or(0, |s| s.fail_count),
                has_score: score.is_some_and(|s| s.scored_count > 0),
            }
        })
        .collect()
}

pub(super) fn user_rows(users: &[UserDistributionRow], total_requests: i64) -> Vec<UserRowView> {
    users
        .iter()
        .map(|u| UserRowView {
            user_id: u.user_id.clone(),
            user_label: u
                .user_label
                .clone()
                .unwrap_or_else(|| u.user_id.as_str().to_owned()),
            request_count: u.request_count,
            share_pct: share_pct(u.request_count, total_requests),
            session_count: u.session_count,
            model_count: u.model_count,
            error_count: u.error_count,
            cost_display: format_cost(u.cost_microdollars),
            last_seen_local: local_time(u.last_seen),
        })
        .collect()
}

pub(super) fn topic_rows(topics: &[PromptTopicRow], total_requests: i64) -> Vec<TopicRowView> {
    topics
        .iter()
        .map(|t| TopicRowView {
            topic: t.topic.clone(),
            sample_excerpt: truncate(&t.sample_excerpt, 160),
            request_count: t.request_count,
            share_pct: share_pct(t.request_count, total_requests),
            distinct_models: t.distinct_models,
            cost_display: format_cost(t.cost_microdollars),
        })
        .collect()
}

pub(super) fn win_rate_rows(rows: &[ModelWinRateRow]) -> Vec<WinRateView> {
    rows.iter()
        .map(|r| {
            let decisive = r.wins + r.losses;
            WinRateView {
                model: r.model.clone(),
                comparisons: r.comparisons,
                wins: r.wins,
                losses: r.losses,
                ties: r.ties,
                win_rate_pct: if decisive > 0 {
                    ((r.wins as f64 / decisive as f64) * 100.0).round() as i64
                } else {
                    0
                },
            }
        })
        .collect()
}


pub(super) fn model_options(models: &[EvalModelDistributionRow]) -> Vec<EvalModelOptionView> {
    models
        .iter()
        .map(|m| EvalModelOptionView {
            value: format!("{}/{}", m.provider, m.model),
            label: format!("{} ({})", m.model, m.provider),
        })
        .collect()
}

pub(super) fn pair_rows(pairs: &[EvalPairRow]) -> Vec<PairRowView> {
    pairs
        .iter()
        .map(|p| PairRowView {
            model_a: p.model_a.clone(),
            model_b: p.model_b.clone(),
            winner_label: match p.winner.as_str() {
                "a" => p.model_a.clone(),
                "b" => p.model_b.clone(),
                _ => "tie".to_owned(),
            },
            is_tie: p.winner == "tie",
            order_swapped: p.order_swapped,
            rationale: p
                .rationale
                .clone()
                .unwrap_or_else(|| "No rationale recorded.".to_owned()),
            created_at_local: local_time(p.created_at),
        })
        .collect()
}

// Why: The Judge tab's filter selects, with the active choice pre-selected so
// the controls still describe what is on screen after the round trip.
pub(super) fn result_filter_view(
    filter: &ResultFilter,
    models: &[EvalModelOptionView],
) -> ResultFilterView {
    const VERDICTS: [(&str, &str); 4] = [
        ("", "Any verdict"),
        ("fail", "Fail"),
        ("partial", "Partial"),
        ("pass", "Pass"),
    ];

    let verdict = filter.verdict.clone().unwrap_or_default();
    let model = filter.model.clone().unwrap_or_default();

    let verdict_options = VERDICTS
        .iter()
        .map(|&(value, label)| FilterOptionView {
            value: value.to_owned(),
            label: label.to_owned(),
            is_selected: verdict == value,
        })
        .collect();

    // Why: The filter matches on the bare model name, while the run forms' options
    // are `provider/model` — so the value here is the model half only.
    let mut model_options = vec![FilterOptionView {
        value: String::new(),
        label: "Any model".to_owned(),
        is_selected: model.is_empty(),
    }];
    model_options.extend(models.iter().map(|m| {
        let bare = m.value.split_once('/').map_or(m.value.as_str(), |(_, m)| m);
        FilterOptionView {
            value: bare.to_owned(),
            label: m.label.clone(),
            is_selected: model == bare,
        }
    }));

    ResultFilterView {
        is_filtered: !verdict.is_empty() || !model.is_empty(),
        verdict,
        model,
        verdict_options,
        model_options,
    }
}
