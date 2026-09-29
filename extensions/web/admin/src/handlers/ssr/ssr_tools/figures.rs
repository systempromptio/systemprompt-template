//! The KPI tiles and the charts above the Tools and Artifacts tables, each
//! figure with its glyph, its tone and the window's trend in a reserved slot.

use chrono::{DateTime, Utc};
use serde::Serialize;

use super::query::Lens;
use super::rows::format_bytes;
use crate::handlers::ssr::analysis::tone::{deny_tone, error_rate_tone, latency_tone, percent};
use crate::handlers::ssr::format::format_duration_ms;
use crate::handlers::ssr::types::{
    Plot, SparklineView, SvgLineChartView, SvgSeriesInput, chart_on_axis, sparkline_toned,
};
use crate::repositories::analysis::tools::{ToolActivityTotals, ToolSeriesPoint};

#[derive(Debug, Serialize)]
pub(crate) struct ToolTileView {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
    pub hint: &'static str,
    pub spark: SparklineView,
}

struct Tile {
    label: &'static str,
    icon: &'static str,
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
}

impl Tile {
    fn with_spark(
        self,
        series: &[ToolSeriesPoint],
        f: fn(&ToolSeriesPoint) -> i64,
    ) -> ToolTileView {
        let values: Vec<i64> = series.iter().map(f).collect();
        ToolTileView {
            spark: sparkline_toned(
                &values,
                self.tone,
                format!("{} per bucket over the window", self.label),
            ),
            label: self.label,
            icon: self.icon,
            value: self.value,
            note: self.note,
            tone: self.tone,
            hint: self.hint,
        }
    }
}

fn tools_tiles(t: &ToolActivityTotals, series: &[ToolSeriesPoint]) -> Vec<ToolTileView> {
    let p95 = t.p95_duration_ms.map_or_else(
        || "\u{2014}".to_owned(),
        |ms| format_duration_ms(ms.round() as i64),
    );
    vec![
        Tile { label: "Tool calls", icon: "wrench", value: t.calls.to_string(),
               note: format!("{} builtin · {} MCP", t.builtin, t.calls - t.builtin), tone: "accent",
               hint: "Every tool call the ledger holds in the window: the model's intent, the execution, or both" }
            .with_spark(series, |p| p.calls),
        Tile { label: "Executed", icon: "check", value: t.executed.to_string(),
               note: format!("{} intended only · {} unattested", t.intended, t.unattested), tone: "ok",
               hint: "Calls with an execution the ledger joined to a request; intended-only calls never ran, unattested ones ran without a request" }
            .with_spark(series, |p| p.executed),
        Tile { label: "Failed", icon: "cross", value: t.failed.to_string(),
               note: percent(t.failed, t.calls) + " of calls", tone: error_rate_tone(t.failed, t.calls),
               hint: "Executions that failed, timed out or returned an error result" }
            .with_spark(series, |p| p.failed),
        Tile { label: "Denied", icon: "shield", value: t.denied.to_string(),
               note: format!("{} warned", t.warned), tone: deny_tone(t.denied),
               hint: "Calls the governance chain denied, keyed to the call by tool_use_id" }
            .with_spark(series, |p| p.denied),
        Tile { label: "Tools", icon: "layers", value: t.tools.to_string(),
               note: format!("{} servers · {} people", t.servers, t.users), tone: "accent",
               hint: "Distinct tool names and servers seen in the window" }
            .with_spark(series, |p| p.calls),
        Tile { label: "p95 duration", icon: "gauge", value: p95,
               note: format!("{} conversations", t.conversations), tone: latency_tone(t.p95_duration_ms),
               hint: "95th percentile execution time over calls with a recorded run" }
            .with_spark(series, |p| p.executed),
        Tile { label: "Artifacts", icon: "file", value: t.artifacts.to_string(),
               note: format!("{} of calls produced one", percent(t.artifacts, t.calls)), tone: "accent",
               hint: "Calls whose result a person can view or retrieve: a file, a card, a UI resource or a retained body" }
            .with_spark(series, |p| p.artifacts),
        Tile { label: "Files", icon: "file", value: t.files.to_string(),
               note: format!("{} cards · {} UI · {} bodies", t.cards, t.ui, t.bodies), tone: "accent",
               hint: "Artifacts by kind" }
            .with_spark(series, |p| p.files),
    ]
}

fn artifact_tiles(t: &ToolActivityTotals, series: &[ToolSeriesPoint]) -> Vec<ToolTileView> {
    vec![
        Tile { label: "Artifacts", icon: "file", value: t.artifacts.to_string(),
               note: format!("{} people · {} conversations", t.users, t.conversations), tone: "accent",
               hint: "Tool results a person can view or retrieve afterwards" }
            .with_spark(series, |p| p.artifacts),
        Tile { label: "Files", icon: "file", value: t.files.to_string(),
               note: "edited, written or read".to_owned(), tone: "accent",
               hint: "Edit, Write, MultiEdit, NotebookEdit and Read calls naming a file path" }
            .with_spark(series, |p| p.files),
        Tile { label: "Cards", icon: "card", value: t.cards.to_string(),
               note: "typed by an MCP server".to_owned(), tone: "accent",
               hint: "Results an MCP server declared with a type — table, chart, report, presentation card" }
            .with_spark(series, |p| p.cards),
        Tile { label: "UI", icon: "ui", value: t.ui.to_string(),
               note: "MCP Apps resources".to_owned(), tone: "accent",
               hint: "Results carrying a ui:// resource a host can render" }
            .with_spark(series, |p| p.ui),
        Tile { label: "Bodies", icon: "body", value: t.bodies.to_string(),
               note: "retained, previewable".to_owned(), tone: "accent",
               hint: "Structured results whose body is retained in the content-addressed store" }
            .with_spark(series, |p| p.bodies),
        Tile { label: "Failed", icon: "cross", value: t.artifact_errors.to_string(),
               note: format!("{} secret redactions", t.redactions), tone: error_rate_tone(t.artifact_errors, t.artifacts),
               hint: "Artifacts whose tool reported an error" }
            .with_spark(series, |p| p.failed),
        Tile { label: "Stored", icon: "layers", value: format_bytes(t.bytes),
               note: "content-addressed bytes".to_owned(), tone: "accent",
               hint: "Payload bytes of the artifacts in the window" }
            .with_spark(series, |p| p.artifacts),
        Tile { label: "Producing tools", icon: "wrench", value: t.tools.to_string(),
               note: format!("{} servers", t.servers), tone: "accent",
               hint: "Distinct tools whose calls produced these artifacts" }
            .with_spark(series, |p| p.calls),
    ]
}

pub(crate) fn tiles(
    lens: Lens,
    t: &ToolActivityTotals,
    series: &[ToolSeriesPoint],
) -> Vec<ToolTileView> {
    match lens {
        Lens::Tools => tools_tiles(t, series),
        Lens::Artifacts => artifact_tiles(t, series),
    }
}

fn bucket_label(t: DateTime<Utc>, hourly: bool) -> String {
    if hourly {
        t.format("%b %-d %H:00").to_string()
    } else {
        t.format("%b %-d").to_string()
    }
}

fn input(
    series: &[ToolSeriesPoint],
    label: &str,
    f: fn(&ToolSeriesPoint) -> i64,
) -> SvgSeriesInput {
    SvgSeriesInput {
        label: label.to_owned(),
        values: series.iter().map(f).collect(),
        value_display: series.iter().map(f).sum::<i64>().to_string(),
    }
}

fn sum(series: &[ToolSeriesPoint], f: fn(&ToolSeriesPoint) -> i64) -> i64 {
    series.iter().map(f).sum()
}

pub(crate) fn charts(
    lens: Lens,
    series: &[ToolSeriesPoint],
    hourly: bool,
) -> Vec<SvgLineChartView> {
    let labels: Vec<String> = series
        .iter()
        .map(|p| bucket_label(p.bucket, hourly))
        .collect();
    let chart = |plot: Plot| chart_on_axis(&labels, "Nothing in this window", plot);
    let kinds = chart(Plot::new(
        "Artifacts by kind",
        format!(
            "{} files · {} cards · {} UI · {} bodies",
            sum(series, |p| p.files),
            sum(series, |p| p.cards),
            sum(series, |p| p.ui),
            sum(series, |p| p.bodies)
        ),
        vec![
            input(series, "Files", |p| p.files),
            input(series, "Cards", |p| p.cards),
            input(series, "UI", |p| p.ui),
            input(series, "Bodies", |p| p.bodies),
        ],
    ))
    .into_columns();
    match lens {
        Lens::Tools => vec![
            chart(Plot::new(
                "Tool calls",
                format!(
                    "{} calls · {} executed · {} failed",
                    sum(series, |p| p.calls),
                    sum(series, |p| p.executed),
                    sum(series, |p| p.failed)
                ),
                vec![
                    input(series, "Calls", |p| p.calls),
                    input(series, "Executed", |p| p.executed),
                    input(series, "Failed", |p| p.failed),
                ],
            )),
            kinds,
            chart(Plot::new(
                "Denied by governance",
                format!("{} denied", sum(series, |p| p.denied)),
                vec![input(series, "Denied", |p| p.denied)],
            )),
        ],
        Lens::Artifacts => vec![
            chart(Plot::new(
                "Artifacts",
                format!(
                    "{} of {} calls",
                    sum(series, |p| p.artifacts),
                    sum(series, |p| p.calls)
                ),
                vec![
                    input(series, "Artifacts", |p| p.artifacts),
                    input(series, "Calls", |p| p.calls),
                ],
            )),
            kinds,
        ],
    }
}
