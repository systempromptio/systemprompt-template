//! The Handlebars shapes of the Reports pages: the history row, the KPI strip
//! read from a report's digest, the theme cards and the recommendations.

use serde::Serialize;

use crate::handlers::ssr::types::BreadcrumbView;

use crate::handlers::ssr::analysis::help::HelpView;
use crate::handlers::ssr::analysis::tone::{completion_tone, score_display};
use crate::handlers::ssr::format::{format_cost, format_token_total, local_time, relative_time};
use crate::handlers::ssr::list_view::SelectOptionView;
use crate::repositories::analysis::reports::{AnalysisReportRow, DigestTotals, Evidence};

use super::REPORTS_URL;

#[derive(Debug, Serialize)]
pub(super) struct ReportRowView {
    pub href: String,
    pub scope_label: String,
    pub scope_kind: String,
    pub window_display: String,
    pub status: String,
    pub status_tone: &'static str,
    pub headline: String,
    pub assessment: String,
    pub assessment_label: String,
    pub assessment_tone: &'static str,
    pub model: String,
    pub cost_display: String,
    pub requested_by: String,
    pub created_display: String,
    pub created_relative: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub(super) struct ReportTileView {
    pub label: &'static str,
    pub icon: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
    pub hint: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ThemeView {
    pub kind: String,
    pub severity: &'static str,
    pub icon: &'static str,
    pub theme_title: String,
    pub detail: String,
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Serialize)]
pub(super) struct RecommendationView {
    pub action: String,
    pub rationale: String,
    pub priority: &'static str,
    pub priority_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ReportsPageContext {
    pub page: &'static str,
    pub title: &'static str,
    pub rows: Vec<ReportRowView>,
    pub has_rows: bool,
    pub marketplace_options: Vec<SelectOptionView>,
    pub help: HelpView,
}

#[derive(Debug, Serialize)]
pub(super) struct ReportPageContext {
    pub page: &'static str,
    pub title: String,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub report_id: String,
    // Why: what the loading widget says while the model reads the digest.
    pub digest_summary: String,
    pub status_url: String,
    pub scope_label: String,
    pub scope_kind: String,
    pub window_display: String,
    pub status: String,
    pub status_tone: &'static str,
    pub pending: bool,
    pub failed: bool,
    pub headline: String,
    pub assessment: String,
    pub assessment_label: String,
    pub assessment_tone: &'static str,
    pub assessment_icon: &'static str,
    pub generated_display: String,
    pub model: String,
    pub tokens_display: String,
    pub cost_display: String,
    pub requested_by: String,
    pub error: String,
    pub filter_query: Option<String>,
    pub tiles: Vec<ReportTileView>,
    pub themes: Vec<ThemeView>,
    pub recommendations: Vec<RecommendationView>,
    pub digest_text: String,
    pub regenerate_url: String,
    pub back_url: &'static str,
    pub help: HelpView,
}

pub(super) fn scope_label(row: &AnalysisReportRow) -> String {
    match row.scope_kind.as_str() {
        "global" => "Whole instance".to_owned(),
        "marketplace" => format!(
            "Marketplace {}",
            row.scope_label
                .as_deref()
                .or(row.scope_id.as_deref())
                .unwrap_or("?")
        ),
        "skill" => format!("Skill {}", row.scope_id.as_deref().unwrap_or("?")),
        _ => row
            .scope_label
            .clone()
            .or_else(|| row.inputs.filter_label.clone())
            .unwrap_or_else(|| "Filtered view".to_owned()),
    }
}

fn window_display(row: &AnalysisReportRow) -> String {
    format!(
        "{} → {}",
        row.window_start.format("%b %-d"),
        row.window_end.format("%b %-d")
    )
}

pub(super) fn status_tone(status: &str) -> &'static str {
    match status {
        "generated" => "ok",
        "failed" => "err",
        _ => "warn",
    }
}

pub(super) fn report_row(row: &AnalysisReportRow) -> ReportRowView {
    let findings = row.findings.as_ref();
    ReportRowView {
        href: format!("{REPORTS_URL}/{}", urlencoding::encode(&row.id)),
        scope_label: scope_label(row),
        scope_kind: row.scope_kind.clone(),
        window_display: window_display(row),
        status: row.status.clone(),
        status_tone: status_tone(&row.status),
        headline: findings.map(|f| f.headline.clone()).unwrap_or_default(),
        assessment: findings.map_or_else(String::new, |f| f.assessment.as_str().to_owned()),
        assessment_label: findings.map_or_else(String::new, |f| f.assessment.label().to_owned()),
        assessment_tone: findings.map_or("muted", |f| f.assessment.tone()),
        model: row.model.clone().unwrap_or_default(),
        cost_display: row.cost_microdollars.map_or_else(String::new, format_cost),
        requested_by: row.requested_by.clone(),
        created_display: local_time(row.created_at),
        created_relative: relative_time(row.created_at),
        error: row.last_error.clone().unwrap_or_default(),
    }
}

const fn tile(
    head: (&'static str, &'static str),
    value: String,
    note: String,
    tone: &'static str,
    hint: &'static str,
) -> ReportTileView {
    ReportTileView {
        label: head.0,
        icon: head.1,
        value,
        note,
        tone,
        hint,
    }
}

pub(super) fn tiles(t: &DigestTotals) -> Vec<ReportTileView> {
    vec![
        tile(
            ("Conversations", "chat"),
            t.conversations.to_string(),
            format!("{} people · {} turns", t.people, t.turns),
            "accent",
            "Conversations whose last activity fell in the window",
        ),
        tile(
            ("Cost", "coins"),
            format_cost(t.cost_microdollars),
            format!("{} tokens", format_token_total(t.tokens)),
            "accent",
            "Priced spend on every request in these conversations",
        ),
        tile(
            ("Tools", "wrench"),
            t.tool_calls.to_string(),
            format!("{} artifacts", t.artifacts),
            "accent",
            "Tool calls, and the subset that produced something viewable",
        ),
        tile(
            ("Errors", "alert"),
            t.errors.to_string(),
            format!("{} denied · {} safety", t.denied, t.safety_findings),
            if t.errors + t.denied > 0 {
                "warn"
            } else {
                "ok"
            },
            "Failed requests, denied tool calls and safety findings",
        ),
        tile(
            ("AI score", "sparkle"),
            score_display(t.completion_avg),
            format!("{} judged · {} achieved", t.judged, t.achieved),
            completion_tone(t.completion_avg),
            "Mean of the judge's completion score over the judged conversations",
        ),
        tile(
            ("Outcomes", "check"),
            format!("{}/{}/{}", t.achieved, t.partial, t.abandoned),
            "achieved / partial / abandoned".to_owned(),
            "muted",
            "The judge's outcome counts",
        ),
    ]
}

pub(super) fn severity_icon(severity: &str) -> &'static str {
    match severity {
        "err" => "cross",
        "warn" => "alert",
        _ => "check",
    }
}

pub(super) fn assessment_icon(assessment: &str) -> &'static str {
    match assessment {
        "degraded" => "cross",
        "watch" => "alert",
        "ok" => "check",
        _ => "sparkle",
    }
}
