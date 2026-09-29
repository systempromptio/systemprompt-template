//! The five totals above the entities table and the filter menus beside it.
//!
//! Split from [`super::entities`] so the table builder stays inside the line
//! budget; nothing here is more than counting.

use super::entities::EntitiesInput;
use super::view::{AcKpiView, AcOptionView};
use crate::types::access_control::AccessDecision;

pub(super) fn kpis(input: &EntitiesInput<'_>, entities: usize) -> Vec<AcKpiView> {
    let denies = input
        .rows
        .iter()
        .filter(|r| r.access == AccessDecision::Deny)
        .count();
    let drift_total = input.drift.map_or(0, |d| d.counts().total);
    vec![
        AcKpiView {
            label: "Entities governed",
            value: entities.to_string(),
            note: format!("{} rules between them", input.rows.len()),
            tone: "accent",
        },
        AcKpiView {
            label: "Open by default",
            value: input.open_entities.to_string(),
            note: "reached by everyone the rules do not refuse".to_owned(),
            tone: if input.open_entities > 0 {
                "warn"
            } else {
                "ok"
            },
        },
        AcKpiView {
            label: "Denies",
            value: denies.to_string(),
            note: "a deny beats an allow in the same band".to_owned(),
            tone: if denies > 0 { "warn" } else { "ok" },
        },
        AcKpiView {
            label: "Drift",
            value: drift_total.to_string(),
            note: if input.drift.is_some() {
                "differences between rules.yaml and this database".to_owned()
            } else {
                "the declaration could not be compared; see the notice above".to_owned()
            },
            tone: if drift_total > 0 || input.drift.is_none() {
                "warn"
            } else {
                "ok"
            },
        },
        AcKpiView {
            label: "Audiences",
            value: input.audiences.to_string(),
            note: "roles, groups and projects on the Audience grid tab".to_owned(),
            tone: "accent",
        },
    ]
}

pub(super) fn options(
    values: Vec<(String, String)>,
    all: &str,
    selected: Option<&str>,
) -> Vec<AcOptionView> {
    let mut out = vec![AcOptionView {
        value: String::new(),
        label: all.to_owned(),
        selected: selected.is_none(),
    }];
    out.extend(values.into_iter().map(|(value, label)| AcOptionView {
        selected: selected == Some(value.as_str()),
        value,
        label,
    }));
    out
}
