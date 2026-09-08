//! The controls the rule ledger is driven by: its query string, its filter
//! menus, its sortable headers and its five totals.
//!
//! Split from [`super::rules`] along the line between the ledger itself and
//! the apparatus around it. The two share one vocabulary — [`RulesQuery`] and
//! the two source labels — which is declared here because everything that
//! reads a query parameter lives here.

use serde::{Deserialize, Serialize};

use crate::handlers::ssr::types::SortHeaderView;
use crate::repositories::access_control::rules::LedgerRuleRow;

pub(super) const BASE_URL: &str = "/admin/access-control";
pub(super) const PAGE_SIZE: i64 = 50;

// Why: the two answers the rule-source column gives. A rule the YAML declares
// survives a rebuild from source; one this database alone holds does not.
pub(super) const YAML: &str = "YAML";
pub(super) const MANUAL: &str = "This instance";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct RulesQuery {
    // Why: the editor's old deep link. A `?user=` on this page is answered by
    // a redirect to that person's Access tab, where the matrix now lives.
    pub user: Option<String>,
    pub subject_kind: Option<String>,
    pub entity_kind: Option<String>,
    pub source: Option<String>,
    pub q: Option<String>,
    pub page: Option<i64>,
    pub sort: Option<String>,
    pub dir: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcOptionView {
    pub value: String,
    pub label: String,
    pub selected: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct AcKpiView {
    pub label: &'static str,
    pub value: String,
    pub note: String,
    pub tone: &'static str,
}


fn field(value: Option<&String>) -> Option<&str> {
    value.map(String::as_str).filter(|v| !v.is_empty())
}

impl RulesQuery {
    pub(super) fn selected(&self, name: &str) -> Option<&str> {
        match name {
            "subject_kind" => field(self.subject_kind.as_ref()),
            "entity_kind" => field(self.entity_kind.as_ref()),
            "source" => field(self.source.as_ref()),
            "q" => field(self.q.as_ref()),
            _ => None,
        }
    }

    pub(super) fn sort_key(&self) -> &str {
        match self.sort.as_deref() {
            Some(key @ ("entity" | "subject_kind" | "subject" | "access" | "source")) => key,
            _ => "entity_kind",
        }
    }

    pub(super) fn descending(&self) -> bool {
        self.dir.as_deref() == Some("desc")
    }

    pub(super) fn any_applied(&self) -> bool {
        ["subject_kind", "entity_kind", "source", "q"]
            .iter()
            .any(|f| self.selected(f).is_some())
    }

    pub(super) fn url_with(&self, overrides: &[(&str, &str)]) -> String {
        let mut parts: Vec<String> = Vec::new();
        for name in [
            "subject_kind",
            "entity_kind",
            "source",
            "q",
            "sort",
            "dir",
            "page",
        ] {
            let value = match overrides.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => (*v).to_owned(),
                None => match name {
                    "sort" => self.sort_key().to_owned(),
                    "dir" => self.dir.clone().unwrap_or_default(),
                    "page" => String::new(),
                    other => self.selected(other).unwrap_or_default().to_owned(),
                },
            };
            if value.is_empty() || (name == "sort" && value == "entity_kind") {
                continue;
            }
            parts.push(format!("{name}={}", urlencoding::encode(&value)));
        }
        if parts.is_empty() {
            return BASE_URL.to_owned();
        }
        format!("{BASE_URL}?{}", parts.join("&"))
    }
}

pub(super) fn options(
    values: &[(String, String)],
    all_label: &str,
    selected: Option<&str>,
) -> Vec<AcOptionView> {
    let mut out = vec![AcOptionView {
        value: String::new(),
        label: all_label.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.iter().map(|(value, label)| AcOptionView {
        selected: selected == Some(value.as_str()),
        value: value.clone(),
        label: label.clone(),
    }));
    out
}

pub(super) fn distinct(
    rows: &[LedgerRuleRow],
    pick: fn(&LedgerRuleRow) -> &String,
) -> Vec<(String, String)> {
    let mut seen: Vec<String> = rows.iter().map(|r| pick(r).clone()).collect();
    seen.sort();
    seen.dedup();
    seen.into_iter()
        .map(|v| {
            let label = v.replace('_', " ");
            (v, label)
        })
        .collect()
}

pub(super) fn sort_headers(query: &RulesQuery) -> Vec<SortHeaderView> {
    let current = query.sort_key();
    let descending = query.descending();
    [
        ("entity_kind", "Entity kind", "sp-col-kind", "What the rule grants access to."),
        ("entity", "Entity", "sp-col-entity", "The id of that entity in the catalog."),
        (
            "subject_kind",
            "Subject kind",
            "sp-col-band",
            "The band the rule is written at: role, group, project or link.",
        ),
        ("subject", "Subject", "sp-col-subject", "The value at that band the rule names."),
        ("access", "Access", "sp-col-access", "A deny at any band beats every allow."),
        (
            "source",
            "Rule source",
            "sp-col-source",
            "Whether the YAML in the source repository declares this rule, or only this database holds it.",
        ),
    ]
    .into_iter()
    .map(|(key, label, class, hint)| {
        let active = current == key;
        let next = if active && !descending { "desc" } else { "" };
        SortHeaderView {
            label,
            class,
            hint,
            url: query.url_with(&[("sort", key), ("dir", next)]),
            active,
            aria_sort: match (active, descending) {
                (true, false) => "ascending",
                (true, true) => "descending",
                (false, _) => "none",
            },
            indicator: match (active, descending) {
                (true, false) => "▲",
                (true, true) => "▼",
                (false, _) => "",
            },
        }
    })
    .collect()
}

pub(super) fn kpis(rows: &[LedgerRuleRow], sources: &[&str], open_entities: i64) -> Vec<AcKpiView> {
    let manual = sources.iter().filter(|s| **s == MANUAL).count();
    let denies = rows
        .iter()
        .filter(|r| matches!(r.access, crate::types::access_control::AccessDecision::Deny))
        .count();
    let mut kinds: Vec<&str> = rows.iter().map(|r| r.rule_type.as_str()).collect();
    kinds.sort_unstable();
    kinds.dedup();
    let mut entities: Vec<(&str, &str)> = rows
        .iter()
        .map(|r| (r.entity_type.as_str(), r.entity_id.as_str()))
        .collect();
    entities.sort_unstable();
    entities.dedup();
    vec![
        AcKpiView {
            label: "Rules",
            value: rows.len().to_string(),
            note: format!("across {} entities", entities.len()),
            tone: "accent",
        },
        AcKpiView {
            label: "Subject bands in use",
            value: kinds.len().to_string(),
            note: kinds.join(", "),
            tone: "accent",
        },
        AcKpiView {
            label: "Denies",
            value: denies.to_string(),
            note: "a deny beats every allow".to_owned(),
            tone: if denies > 0 { "warn" } else { "ok" },
        },
        AcKpiView {
            label: "Only in this database",
            value: manual.to_string(),
            note: "not declared by any YAML file".to_owned(),
            tone: if manual > 0 { "warn" } else { "ok" },
        },
        AcKpiView {
            label: "Open by default",
            value: open_entities.to_string(),
            note: "entities everyone reaches unless denied".to_owned(),
            tone: if open_entities > 0 { "warn" } else { "ok" },
        },
    ]
}
