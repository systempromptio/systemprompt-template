//! The three charts above the conversations table — volume, spend and health
//! — over the same buckets, so a spike in one lines up with the others.

use chrono::{DateTime, Utc};

use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::handlers::ssr::types::{Plot, SvgLineChartView, SvgSeriesInput, chart_on_axis};
use crate::repositories::analysis::conversations::ConversationSeriesPoint;

fn bucket_label(t: DateTime<Utc>, hourly: bool) -> String {
    if hourly {
        t.format("%b %-d %H:00").to_string()
    } else {
        t.format("%b %-d").to_string()
    }
}

// Why: the shared x axis: one label per bucket and the three the gutters
// print, built once and handed to each chart.
struct Axis {
    labels: Vec<String>,
}

impl Axis {
    fn new(series: &[ConversationSeriesPoint], hourly: bool) -> Self {
        Self {
            labels: series
                .iter()
                .map(|p| bucket_label(p.bucket, hourly))
                .collect(),
        }
    }

    fn chart(&self, plot: Plot) -> SvgLineChartView {
        chart_on_axis(&self.labels, "Nothing in this window", plot)
    }
}

fn input(
    series: &[ConversationSeriesPoint],
    label: &str,
    f: fn(&ConversationSeriesPoint) -> i64,
    total_display: String,
) -> SvgSeriesInput {
    SvgSeriesInput {
        label: label.to_owned(),
        values: series.iter().map(f).collect(),
        value_display: total_display,
    }
}

fn sum(series: &[ConversationSeriesPoint], f: fn(&ConversationSeriesPoint) -> i64) -> i64 {
    series.iter().map(f).sum()
}

pub(crate) fn series_charts(
    series: &[ConversationSeriesPoint],
    hourly: bool,
) -> Vec<SvgLineChartView> {
    let axis = Axis::new(series, hourly);
    let peak_users = series.iter().map(|p| p.users).max().unwrap_or(0);
    vec![
        axis.chart(Plot::new(
            "Conversations and turns",
            format!(
                "{} conversations · {} turns",
                sum(series, |p| p.conversations),
                sum(series, |p| p.turns)
            ),
            vec![
                input(
                    series,
                    "Conversations",
                    |p| p.conversations,
                    sum(series, |p| p.conversations).to_string(),
                ),
                input(
                    series,
                    "Turns",
                    |p| p.turns,
                    sum(series, |p| p.turns).to_string(),
                ),
                input(series, "People", |p| p.users, peak_users.to_string()),
            ],
        )),
        axis.chart(Plot {
            y_unit: "µ$",
            y_display: format_cost,
            ..Plot::new(
                "Cost",
                format!(
                    "{} over the window",
                    format_cost(sum(series, |p| p.cost_microdollars))
                ),
                vec![input(
                    series,
                    "Cost",
                    |p| p.cost_microdollars,
                    format_cost(sum(series, |p| p.cost_microdollars)),
                )],
            )
        }),
        axis.chart(Plot::new(
            "Tokens, tool calls and errors",
            format!(
                "{} tokens · {} tool calls · {} failed requests",
                format_token_total(sum(series, |p| p.tokens)),
                sum(series, |p| p.tool_calls),
                sum(series, |p| p.errors)
            ),
            vec![
                input(
                    series,
                    "Tokens (k)",
                    |p| p.tokens / 1000,
                    format_token_total(sum(series, |p| p.tokens)),
                ),
                input(
                    series,
                    "Tool calls",
                    |p| p.tool_calls,
                    sum(series, |p| p.tool_calls).to_string(),
                ),
                input(
                    series,
                    "Errors",
                    |p| p.errors,
                    sum(series, |p| p.errors).to_string(),
                ),
            ],
        )),
    ]
}
