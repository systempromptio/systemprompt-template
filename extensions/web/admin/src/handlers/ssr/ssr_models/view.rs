//! Serializable view models for the `models.hbs` template.

use serde::Serialize;

use crate::handlers::ssr::types::BreadcrumbView;

#[derive(Debug, Serialize)]
pub(super) struct UserOptionView {
    pub id: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelRowView {
    pub route_id: String,
    pub model_pattern: String,
    pub provider: String,
    pub upstream_model: String,
    pub denied: bool,
    pub deny_rule_id: String,
    pub status_label: &'static str,
    pub status_tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct UsageRowView {
    pub request_id: String,
    pub created_at: String,
    pub model: String,
    pub provider: String,
    pub status: String,
    pub status_tone: &'static str,
    pub is_completed: bool,
    pub tokens: String,
    pub cost: String,
    pub latency_ms: i64,
    pub deny_count: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct UsageTotalsView {
    pub requests: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cost: String,
    pub denied_requests: i64,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelsKpiView {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
}

#[derive(Debug, Serialize)]
pub(super) struct ModelsPageData {
    pub page: &'static str,
    pub title: &'static str,
    pub breadcrumbs: Vec<BreadcrumbView>,
    pub base_url: &'static str,
    pub users: Vec<UserOptionView>,
    pub has_selection: bool,
    pub selected_user_id: String,
    pub selected_user_label: String,
    pub models: Vec<ModelRowView>,
    pub model_count: usize,
    pub search: String,
    pub has_search: bool,
    pub usage: Vec<UsageRowView>,
    pub usage_count: usize,
    pub has_usage: bool,
    pub usage_totals: UsageTotalsView,
    pub requests_link: String,
    pub kpis: Vec<ModelsKpiView>,
}

pub(super) fn build_user_options(
    all_users: &[crate::types::UserSummary],
    selected_id: Option<&str>,
) -> Vec<UserOptionView> {
    all_users
        .iter()
        .map(|u| {
            let id = u.user_id.to_string();
            let label = u
                .email
                .as_ref()
                .map_or_else(|| id.clone(), |e| e.as_ref().to_owned());
            UserOptionView {
                selected: Some(id.as_str()) == selected_id,
                id,
                label,
            }
        })
        .collect()
}

// Why: one band answers both questions the page is for — what the gateway
// exposes, and what the selected person did with it. Without a selection the
// usage tiles read zero rather than vanishing, so the band keeps its height.
pub(super) fn build_kpis(
    models: &[ModelRowView],
    totals: &UsageTotalsView,
    has_selection: bool,
) -> Vec<ModelsKpiView> {
    let denied_models = models.iter().filter(|m| m.denied).count();
    let scope = if has_selection {
        "for this user, last 24h"
    } else {
        "select a user to see usage"
    };
    vec![
        ModelsKpiView {
            label: "Models",
            value: models.len().to_string(),
            note: "gateway routes exposed".to_owned(),
            tone: "accent",
        },
        ModelsKpiView {
            label: "Disabled",
            value: denied_models.to_string(),
            note: if has_selection {
                "user-band denies for this person".to_owned()
            } else {
                "select a user to see access".to_owned()
            },
            tone: if denied_models > 0 { "warn" } else { "ok" },
        },
        ModelsKpiView {
            label: "Requests",
            value: totals.requests.to_string(),
            note: scope.to_owned(),
            tone: "accent",
        },
        ModelsKpiView {
            label: "Tokens in / out",
            value: format!("{} / {}", totals.input_tokens, totals.output_tokens),
            note: scope.to_owned(),
            tone: "accent",
        },
        ModelsKpiView {
            label: "Cost",
            value: totals.cost.clone(),
            note: scope.to_owned(),
            tone: "accent",
        },
        ModelsKpiView {
            label: "Not completed",
            value: totals.denied_requests.to_string(),
            note: "denied or failed calls, audited the same".to_owned(),
            tone: if totals.denied_requests > 0 {
                "err"
            } else {
                "ok"
            },
        },
    ]
}
