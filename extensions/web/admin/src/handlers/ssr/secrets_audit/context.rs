//! The KPI strip and the toolbar options for the secrets audit trail.
//!
//! Split from the handler because the tiles carry the page's argument and the
//! handler carries its I/O. The `third_party` tile is the one that earns the
//! page: every other number can be counted off the table, and "an
//! administrator acted on someone else's credential" cannot.

use serde::Serialize;

use super::{BASE_URL, SecretsQuery};
use crate::handlers::ssr::list_view::SelectOptionView;
use crate::repositories::governance::secret_audit_log::SecretAuditStats;

#[derive(Debug, Serialize)]
pub(super) struct SecretAuditKpiView {
    pub(super) label: &'static str,
    pub(super) value: String,
    pub(super) sub: &'static str,
    pub(super) tone: &'static str,
    pub(super) href: String,
    pub(super) active: bool,
    pub(super) hint: &'static str,
}

pub(super) fn url_for(query: &SecretsQuery, action: Option<&str>, page: i64) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(preset) = query.preset.as_deref().filter(|p| !p.is_empty()) {
        parts.push(format!("preset={}", urlencoding::encode(preset)));
    }
    if let Some(action) = action.filter(|a| !a.is_empty()) {
        parts.push(format!("action={}", urlencoding::encode(action)));
    }
    if let Some(search) = query.q.as_deref().filter(|q| !q.is_empty()) {
        parts.push(format!("q={}", urlencoding::encode(search)));
    }
    if page > 0 {
        parts.push(format!("page={page}"));
    }
    if parts.is_empty() {
        BASE_URL.to_owned()
    } else {
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

// Why: `third_party` is the tile that earns the page. Every other number can be
// counted off the table; "an administrator read someone else's credential"
// cannot, and it is the row an auditor came here to find.
pub(super) fn kpis(stats: &SecretAuditStats, query: &SecretsQuery) -> Vec<SecretAuditKpiView> {
    vec![
        SecretAuditKpiView {
            label: "Entries",
            value: stats.entries.to_string(),
            sub: "recorded actions",
            tone: "",
            href: url_for(query, None, 0),
            active: query.action.is_none(),
            hint: "Rows in the window, before any filter",
        },
        SecretAuditKpiView {
            label: "Variables",
            value: stats.variables.to_string(),
            sub: "distinct secrets touched",
            tone: "",
            href: url_for(query, None, 0),
            active: false,
            hint: "Distinct variable names appearing in the window",
        },
        SecretAuditKpiView {
            label: "Actors",
            value: stats.actors.to_string(),
            sub: "people who acted",
            tone: "",
            href: url_for(query, None, 0),
            active: false,
            hint: "Distinct actors, which is not the same as distinct owners",
        },
        SecretAuditKpiView {
            label: "Reads",
            value: stats.accesses.to_string(),
            sub: "decrypt events",
            tone: "warn",
            href: url_for(query, Some("accessed"), 0),
            active: query.action.as_deref() == Some("accessed"),
            hint: "Every time a stored secret was unsealed",
        },
        SecretAuditKpiView {
            label: "Rotations",
            value: stats.rotations.to_string(),
            sub: "credentials replaced",
            tone: "ok",
            href: url_for(query, Some("rotated"), 0),
            active: query.action.as_deref() == Some("rotated"),
            hint: "Rotation is the healthy signal on this page",
        },
        SecretAuditKpiView {
            label: "Third party",
            value: stats.third_party.to_string(),
            sub: "actor was not the owner",
            tone: if stats.third_party > 0 { "err" } else { "ok" },
            href: url_for(query, None, 0),
            active: false,
            hint: "Rows where an administrator acted on someone else's secret",
        },
    ]
}

pub(super) fn action_options(actions: &[String], selected: Option<&str>) -> Vec<SelectOptionView> {
    let mut out = vec![SelectOptionView {
        value: String::new(),
        label: "All actions".to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(actions.iter().map(|action| SelectOptionView {
        selected: selected == Some(action.as_str()),
        value: action.clone(),
        label: action.clone(),
    }));
    out
}
