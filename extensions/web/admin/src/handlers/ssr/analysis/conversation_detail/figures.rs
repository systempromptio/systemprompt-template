//! The KPI tiles and the per-turn charts of the conversation detail page.

use super::views::ConversationKpiView;
use crate::handlers::ssr::analysis::tone::{
    cache_tone, deny_tone, error_rate_tone, latency_tone, percent, safety_tone,
};
use crate::handlers::ssr::format::{format_cost, format_duration_ms, format_token_total};
use crate::handlers::ssr::types::{Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis};
use crate::repositories::analysis::conversations::ConversationFactRow;
use crate::repositories::analysis::conversations::planes::ConversationTurnRow;

const fn kpi(
    head: (&'static str, &'static str),
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
) -> ConversationKpiView {
    ConversationKpiView {
        label: head.0,
        icon: head.1,
        value,
        note,
        tone,
        hint,
    }
}

fn usage_kpis(f: &ConversationFactRow) -> Vec<ConversationKpiView> {
    let cache = f.cache_read_tokens + f.cache_creation_tokens;
    vec![
        kpi(
            ("Turns", "bolt"),
            f.turn_count.to_string(),
            format!(
                "{} side calls · {} hook prompts",
                f.side_call_count, f.prompt_count
            ),
            "accent",
            "Human prompts answered; side calls are the harness's own title and summary requests",
        ),
        kpi(
            ("Tokens", "token"),
            format_token_total(f.total_tokens),
            format!(
                "{} in · {} out · {} reasoning",
                format_token_total(f.input_tokens),
                format_token_total(f.output_tokens),
                format_token_total(f.reasoning_tokens)
            ),
            "accent",
            "Input plus output tokens across every request",
        ),
        kpi(
            ("Cache", "cache"),
            percent(f.cache_read_tokens, f.cache_read_tokens + f.input_tokens),
            format!(
                "{} read · {} written",
                format_token_total(f.cache_read_tokens),
                format_token_total(f.cache_creation_tokens)
            ),
            cache_tone(cache, f.input_tokens),
            "Share of everything the models read that came from prompt cache",
        ),
        kpi(
            ("Cost", "coins"),
            format_cost(f.cost_microdollars),
            format!(
                "{} on side calls · {} per turn",
                format_cost(f.side_call_cost_microdollars),
                format_cost(f.cost_microdollars / f.turn_count.max(1))
            ),
            "accent",
            "Priced spend on every request",
        ),
        kpi(
            ("Latency", "gauge"),
            f.p95_latency_ms
                .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
            format!(
                "p50 {} · max {}",
                f.p50_latency_ms
                    .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms))),
                f.max_latency_ms
                    .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms)))
            ),
            latency_tone(f.p95_latency_ms.map(f64::from)),
            "95th percentile of turn latency at the gateway",
        ),
    ]
}

fn health_kpis(f: &ConversationFactRow) -> Vec<ConversationKpiView> {
    vec![
        kpi(
            ("Tools", "wrench"),
            format!(
                "{}/{}",
                f.tool_calls_executed,
                f.tool_calls_intended.max(f.tool_calls_executed)
            ),
            format!("executed / asked · {} failed", f.tool_calls_failed),
            if f.tool_calls_failed > 0 {
                "err"
            } else {
                "accent"
            },
            "Executed over requested tool calls; the ledger joins the model's intent to the execution and its result",
        ),
        kpi(
            ("Artifacts", "file"),
            f.artifact_count.to_string(),
            format!(
                "{} files · {} cards, UI, bodies",
                f.artifact_files, f.artifact_cards
            ),
            if f.artifact_count > 0 {
                "accent"
            } else {
                "muted"
            },
            "Tool calls that produced something viewable — a file edited, written or read, a typed card, a UI resource or a retained body",
        ),
        kpi(
            ("Errors", "alert"),
            f.error_count.to_string(),
            format!(
                "{} rejected · {} streaming",
                f.rejected_count, f.streaming_count
            ),
            error_rate_tone(f.error_count, f.request_count),
            "Failed requests; rejected ones never reached a model",
        ),
        kpi(
            ("Governance", "shield"),
            f.gov_deny.to_string(),
            format!("denied · {} warned · {} allowed", f.gov_warn, f.gov_allow),
            deny_tone(f.gov_deny),
            "Decisions the governance chain took on this conversation's tool calls",
        ),
        kpi(
            ("Safety", "alert"),
            f.safety_findings.to_string(),
            format!("findings · {} blocked", f.safety_blocked),
            safety_tone(f.safety_findings, f.safety_blocked),
            "Gateway safety scanner findings on requests and responses",
        ),
        kpi(
            ("Skills", "skill"),
            f.skill_invocations.to_string(),
            format!("invocations · {} distinct", f.skills.len()),
            if f.skill_invocations > 0 {
                "accent"
            } else {
                "muted"
            },
            "Skill invocations the harness hooks reported for this session",
        ),
    ]
}

pub(super) fn kpis(f: &ConversationFactRow) -> Vec<ConversationKpiView> {
    let mut tiles = usage_kpis(f);
    tiles.extend(health_kpis(f));
    tiles
}

// Why: the per-turn x axis — one label per turn and the first/middle/last
// index the gutters print — built once for the three charts.
struct TurnAxis<'a> {
    turns: Vec<&'a ConversationTurnRow>,
    labels: Vec<String>,
}

impl<'a> TurnAxis<'a> {
    fn new(turns: &'a [ConversationTurnRow]) -> Self {
        let turns: Vec<&ConversationTurnRow> = turns
            .iter()
            .filter(|t| t.effective_kind == "turn")
            .collect();
        let labels = turns
            .iter()
            .enumerate()
            .map(|(i, t)| format!("Turn {} · {}", i + 1, t.created_at.format("%H:%M:%S")))
            .collect();
        Self { turns, labels }
    }

    fn series(
        &self,
        label: &str,
        f: fn(&ConversationTurnRow) -> i64,
        total: String,
    ) -> SvgSeriesInput {
        SvgSeriesInput {
            label: label.to_owned(),
            values: self.turns.iter().map(|t| f(t)).collect(),
            value_display: total,
        }
    }

    fn sum(&self, f: fn(&ConversationTurnRow) -> i64) -> i64 {
        self.turns.iter().map(|t| f(t)).sum()
    }

    fn chart(&self, plot: Plot) -> SvgLineChartView {
        chart_on_axis(&self.labels, "No turns recorded", plot)
    }
}

fn tokens_chart(axis: &TurnAxis<'_>) -> SvgLineChartView {
    let total = |f: fn(&ConversationTurnRow) -> i64| format_token_total(axis.sum(f));
    axis.chart(Plot {
        y_unit: "tok",
        ..Plot::new(
            "Tokens per turn",
            format!(
                "{} in · {} out · {} cached",
                total(|t| t.input_tokens),
                total(|t| t.output_tokens),
                total(|t| t.cache_read_tokens)
            ),
            vec![
                axis.series("Input", |t| t.input_tokens, total(|t| t.input_tokens)),
                axis.series("Output", |t| t.output_tokens, total(|t| t.output_tokens)),
                axis.series(
                    "Cache read",
                    |t| t.cache_read_tokens,
                    total(|t| t.cache_read_tokens),
                ),
            ],
        )
    })
    .into_columns()
}

fn cost_chart(axis: &TurnAxis<'_>) -> SvgLineChartView {
    let total = format_cost(axis.sum(|t| t.cost_microdollars));
    axis.chart(Plot {
        y_unit: "µ$",
        y_display: format_cost,
        ..Plot::new(
            "Cost per turn",
            format!("{total} across the turns"),
            vec![axis.series("Cost", |t| t.cost_microdollars, total)],
        )
    })
    .into_columns()
}

fn latency_chart(axis: &TurnAxis<'_>) -> SvgLineChartView {
    let max = axis
        .turns
        .iter()
        .filter_map(|t| t.latency_ms)
        .max()
        .map_or_else(|| "—".to_owned(), |ms| format_duration_ms(i64::from(ms)));
    axis.chart(Plot {
        y_unit: "ms",
        y_display: format_duration_ms,
        ref_lines: vec![(20_000, "20 s target".to_owned(), "warn")],
        ..Plot::new(
            "Latency per turn",
            "Gateway latency per turn against the 20 s target".to_owned(),
            vec![axis.series(
                "Latency (ms)",
                |t| i64::from(t.latency_ms.unwrap_or(0)),
                format!("max {max}"),
            )],
        )
    })
}

pub(super) fn charts(turns: &[ConversationTurnRow]) -> Vec<SvgLineChartView> {
    let axis = TurnAxis::new(turns);
    vec![tokens_chart(&axis), cost_chart(&axis), latency_chart(&axis)]
}
