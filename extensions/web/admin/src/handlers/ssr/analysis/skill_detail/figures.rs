//! The KPI tiles and the four day-by-day charts of the skill detail page.

use super::views::SkillKpiView;
use crate::handlers::ssr::analysis::tone::{
    completion_tone, deny_tone, error_rate_tone, latency_tone, percent, score_display,
};
use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::handlers::ssr::types::{Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis};
use crate::repositories::analysis::skills::{SkillDayRow, SkillFactRow};

fn usage_kpis(f: &SkillFactRow, entitled: usize) -> Vec<SkillKpiView> {
    let kpi = |label, icon, value, note, tone, hint| SkillKpiView {
        label,
        icon,
        value,
        note,
        tone,
        hint,
    };
    let tokens = f.input_tokens + f.output_tokens;
    vec![
        kpi(
            "Invocations",
            "bolt",
            f.invocations.to_string(),
            format!(
                "{} slash · {} tool · {} attributed",
                f.slash,
                f.tool,
                percent(f.attributed, f.invocations)
            ),
            "accent",
            "Hook-reported invocations in the window",
        ),
        kpi(
            "People",
            "people",
            f.users.to_string(),
            format!("of {entitled} entitled · {} installed", f.installs),
            "accent",
            "Distinct people who invoked the skill; entitlement from the access-control rules, installs from verified receipts",
        ),
        kpi(
            "Conversations",
            "chat",
            f.conversations.to_string(),
            format!("{} turns · {} sessions", f.turns, f.sessions),
            "accent",
            "Gateway conversations whose harness session invoked the skill",
        ),
        kpi(
            "Tokens",
            "token",
            format_token_total(tokens),
            format!(
                "{} in · {} out · {} cached",
                format_token_total(f.input_tokens),
                format_token_total(f.output_tokens),
                format_token_total(f.cache_tokens)
            ),
            "accent",
            "Tokens those conversations ran up",
        ),
        kpi(
            "Cost",
            "coins",
            format_cost(f.cost_microdollars),
            format!(
                "{} per invocation · {} per conversation",
                format_cost(f.cost_microdollars / f.invocations.max(1)),
                format_cost(f.cost_microdollars / f.conversations.max(1))
            ),
            "accent",
            "Priced spend of those conversations",
        ),
    ]
}

fn health_kpis(f: &SkillFactRow) -> Vec<SkillKpiView> {
    let kpi = |label, icon, value, note, tone, hint| SkillKpiView {
        label,
        icon,
        value,
        note,
        tone,
        hint,
    };
    vec![
        kpi(
            "Tools",
            "wrench",
            f.tool_calls.to_string(),
            format!("{} failed · {} artifacts", f.tool_calls_failed, f.artifacts),
            if f.tool_calls_failed > 0 {
                "err"
            } else {
                "accent"
            },
            "Tool calls the model asked for inside those conversations",
        ),
        kpi(
            "Errors",
            "alert",
            f.errors.to_string(),
            format!("failed requests · {} denied", f.denied),
            if f.denied > 0 {
                deny_tone(f.denied)
            } else {
                error_rate_tone(f.errors, f.requests)
            },
            "Failed requests and governance denials",
        ),
        kpi(
            "p95 latency",
            "gauge",
            f.p95_latency_ms.map_or_else(
                || "—".to_owned(),
                |ms| format_duration_ms(ms.round() as i64),
            ),
            "over the conversations' own p95".to_owned(),
            latency_tone(f.p95_latency_ms),
            "95th percentile of turn latency",
        ),
        kpi(
            "AI score",
            "sparkle",
            score_display(f.completion_avg),
            format!(
                "{} judged · {} achieved",
                f.judged,
                percent(f.achieved, f.conversations.max(1))
            ),
            completion_tone(f.completion_avg),
            "Mean of the judge's one completion score over judged conversations that used this skill",
        ),
    ]
}

pub(super) fn kpis(f: Option<&SkillFactRow>, entitled: usize) -> Vec<SkillKpiView> {
    let Some(f) = f else {
        return Vec::new();
    };
    let mut tiles = usage_kpis(f, entitled);
    tiles.extend(health_kpis(f));
    tiles
}

// Why: the day axis shared by the four charts: one label per day and the
// first/middle/last the gutters print.
struct DayAxis {
    labels: Vec<String>,
}

impl DayAxis {
    fn new(daily: &[SkillDayRow]) -> Self {
        Self {
            labels: daily
                .iter()
                .map(|d| d.day.format("%b %-d").to_string())
                .collect(),
        }
    }

    fn chart(&self, plot: Plot) -> SvgLineChartView {
        chart_on_axis(&self.labels, "Nothing in this window", plot)
    }
}

fn series(
    daily: &[SkillDayRow],
    label: &str,
    f: fn(&SkillDayRow) -> i64,
    total: String,
) -> SvgSeriesInput {
    SvgSeriesInput {
        label: label.to_owned(),
        values: daily.iter().map(f).collect(),
        value_display: total,
    }
}

fn sum(daily: &[SkillDayRow], f: fn(&SkillDayRow) -> i64) -> i64 {
    daily.iter().map(f).sum()
}

fn volume_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Invocations and people per day",
        format!("{} invocations", sum(daily, |d| d.invocations)),
        vec![
            series(
                daily,
                "Invocations",
                |d| d.invocations,
                sum(daily, |d| d.invocations).to_string(),
            ),
            series(
                daily,
                "People",
                |d| d.users,
                daily.iter().map(|d| d.users).max().unwrap_or(0).to_string(),
            ),
            series(
                daily,
                "Conversations",
                |d| d.conversations,
                sum(daily, |d| d.conversations).to_string(),
            ),
        ],
    ))
}

fn spend_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot::new(
        "Cost and tokens per day",
        format!(
            "{} · {} tokens",
            format_cost(sum(daily, |d| d.cost_microdollars)),
            format_token_total(sum(daily, |d| d.tokens))
        ),
        vec![
            series(
                daily,
                "Cost (µ$)",
                |d| d.cost_microdollars,
                format_cost(sum(daily, |d| d.cost_microdollars)),
            ),
            series(
                daily,
                "Tokens (k)",
                |d| d.tokens / 1000,
                format_token_total(sum(daily, |d| d.tokens)),
            ),
        ],
    ))
}

fn health_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    let worst = daily
        .iter()
        .filter_map(|d| d.p95_latency_ms)
        .fold(0.0f64, f64::max)
        .round();
    axis.chart(Plot {
        ref_lines: vec![(20_000, "20 s target".to_owned(), "warn")],
        ..Plot::new(
            "p95 latency and errors per day",
            "Gateway p95 per day against the 20 s target".to_owned(),
            vec![
                series(
                    daily,
                    "p95 (ms)",
                    |d| d.p95_latency_ms.map_or(0, |v| v.round() as i64),
                    worst.to_string(),
                ),
                series(
                    daily,
                    "Errors",
                    |d| d.errors,
                    sum(daily, |d| d.errors).to_string(),
                ),
            ],
        )
    })
}

fn completion_chart(axis: &DayAxis, daily: &[SkillDayRow]) -> SvgLineChartView {
    axis.chart(Plot {
        ref_lines: vec![
            (80, "good".to_owned(), "ok"),
            (50, "watch".to_owned(), "warn"),
        ],
        y_max: Some(100),
        ..Plot::new(
            "Completion per day",
            "Mean judge completion of the conversations that used the skill".to_owned(),
            vec![series(
                daily,
                "Completion",
                |d| d.completion_avg.map_or(0, |v| v.round() as i64),
                String::new(),
            )],
        )
    })
}

pub(super) fn charts(daily: &[SkillDayRow]) -> Vec<SvgLineChartView> {
    let axis = DayAxis::new(daily);
    vec![
        volume_chart(&axis, daily),
        spend_chart(&axis, daily),
        health_chart(&axis, daily),
        completion_chart(&axis, daily),
    ]
}
