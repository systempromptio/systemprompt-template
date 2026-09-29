//! The fixed vocabularies the judge writes and the small views above the
//! conversations table: the filter state and the breakdown rows. The KPI
//! tiles are in `kpis`, the charts in `charts`.

use serde::Serialize;

use crate::handlers::ssr::analysis::tone::{
    completion_tone, deny_tone, error_rate_tone, percent, score_display,
};
use crate::handlers::ssr::format::{format_cost, format_token_total};
use crate::repositories::analysis::conversations::ConversationBucketRow;

pub(crate) const CATEGORIES: [(&str, &str); 7] = [
    ("development", "Development"),
    ("business-analysis", "Business analysis"),
    ("operations", "Operations"),
    ("admin-config", "Admin & config"),
    ("writing-comms", "Writing & comms"),
    ("research-learning", "Research & learning"),
    ("other", "Other"),
];

pub(crate) const OUTCOMES: [(&str, &str); 4] = [
    ("achieved", "Achieved"),
    ("partial", "Partial"),
    ("abandoned", "Abandoned"),
    ("unclear", "Unclear"),
];

#[must_use]
pub(crate) fn category_label(value: &str) -> &'static str {
    CATEGORIES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("Unjudged", |(_, label)| label)
}

pub(crate) fn outcome_label(value: &str) -> &'static str {
    OUTCOMES
        .iter()
        .find(|(v, _)| *v == value)
        .map_or("Unclear", |(_, label)| label)
}

// Why: the badge tones the console already defines; an outcome reads at a
// glance without a legend.
pub(crate) fn outcome_tone(value: &str) -> &'static str {
    match value {
        "achieved" => "ok",
        "partial" => "warn",
        "abandoned" => "err",
        _ => "muted",
    }
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationAnalysisFilterView {
    pub since: &'static str,
    pub q: String,
    pub category: String,
    pub outcome: String,
    pub skill: String,
    pub judged: String,
    pub model: String,
    pub client: String,
    pub flag: String,
    pub user_key: String,
    pub group: String,
    pub project: String,
    pub by: &'static str,
    pub sort: &'static str,
    pub dir: &'static str,
    // Why: the whole query string, for the judge-all form to carry back.
    pub query: String,
}

#[derive(Debug, Serialize)]
pub(crate) struct ConversationBucketView {
    pub label: String,
    pub href: Option<String>,
    pub export_href: Option<String>,
    // Why: the bucket's own query for the export dialog, so the icon opens on
    // this bucket's rows rather than the page's whole filter set.
    pub export_query: Option<String>,
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub tool_calls: i64,
    pub artifacts: i64,
    pub tokens_display: String,
    pub cost_display: String,
    pub cost_share_pct: i64,
    pub errors: i64,
    pub errors_tone: &'static str,
    pub denied: i64,
    pub denied_tone: &'static str,
    pub achieved_display: String,
    pub judged: i64,
    pub completion_display: String,
    pub completion_tone: &'static str,
    pub top_category: &'static str,
}

impl ConversationBucketView {
    pub(crate) fn new(
        row: &ConversationBucketRow,
        href: Option<String>,
        export: Option<(String, String)>,
        label: Option<&str>,
        total_cost: i64,
    ) -> Self {
        let (export_href, export_query) = export.unzip();
        Self {
            label: label.map_or_else(|| row.label.clone(), str::to_owned),
            href,
            export_href,
            export_query,
            conversations: row.conversations,
            users: row.users,
            turns: row.turns,
            tool_calls: row.tool_calls,
            artifacts: row.artifacts,
            tokens_display: format_token_total(row.total_tokens),
            cost_display: format_cost(row.total_cost_microdollars),
            cost_share_pct: if total_cost > 0 {
                (row.total_cost_microdollars * 100 / total_cost).clamp(0, 100)
            } else {
                0
            },
            errors: row.errors,
            errors_tone: error_rate_tone(row.errors, row.turns.max(row.conversations)),
            denied: row.denied,
            denied_tone: deny_tone(row.denied),
            achieved_display: percent(row.achieved, row.conversations),
            judged: row.judged,
            completion_display: score_display(row.completion_avg),
            completion_tone: completion_tone(row.completion_avg),
            top_category: row.top_category.as_deref().map_or("—", category_label),
        }
    }
}
