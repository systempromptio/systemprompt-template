//! The panel's rule list: one entity's rows from the ledger, grouped by band
//! in precedence order, each with its reason, where it was written and when
//! it stops applying.

use std::collections::HashMap;

use systemprompt_security::authz::{DASHBOARD_SOURCE, YAML_SOURCE};

use super::view::{BandRulesView, PanelRuleView};
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::repositories::sync::access_control_rows::band_label;
use crate::types::access_control::AccessDecision;

// Why: the resolver's ladder, narrowest first — the order a reader walks to
// find which rule decides.
const BAND_ORDER: [&str; 6] = [
    "user",
    "project",
    "group",
    "connector",
    "role",
    "organization",
];

fn rank(band: &str) -> usize {
    BAND_ORDER
        .iter()
        .position(|b| *b == band)
        .unwrap_or(BAND_ORDER.len())
}

#[must_use]
pub(super) fn source_words(source: &str) -> (String, &'static str) {
    match source {
        YAML_SOURCE => ("code".to_owned(), "muted"),
        DASHBOARD_SOURCE => ("console".to_owned(), "info"),
        s => s.strip_prefix("bundle:").map_or_else(
            || (s.to_owned(), "muted"),
            |kit| (format!("kit {kit}"), "muted"),
        ),
    }
}

pub(super) fn band_rules(
    rows: &[&LedgerRuleRow],
    names: &HashMap<(String, String), String>,
) -> Vec<BandRulesView> {
    let mut bands: Vec<BandRulesView> = Vec::new();
    let mut sorted: Vec<&&LedgerRuleRow> = rows.iter().collect();
    sorted.sort_by(|a, b| {
        (rank(&a.rule_type), &a.rule_value).cmp(&(rank(&b.rule_type), &b.rule_value))
    });
    for row in sorted {
        let (source_label, source_tone) = source_words(&row.source);
        let view = PanelRuleView {
            id: row.id.clone(),
            band: row.rule_type.clone(),
            subject: row.rule_value.clone(),
            subject_label: names
                .get(&(row.rule_type.clone(), row.rule_value.clone()))
                .cloned()
                .unwrap_or_else(|| row.rule_value.clone()),
            decision: match row.access {
                AccessDecision::Allow => "allow",
                AccessDecision::Deny => "deny",
            },
            why: row.justification.clone().unwrap_or_default(),
            source_label,
            source_tone,
            until: row.valid_until.map(|t| t.format("%Y-%m-%d").to_string()),
        };
        match bands.last_mut() {
            Some(last) if last.band == row.rule_type => last.rules.push(view),
            _ => bands.push(BandRulesView {
                band: row.rule_type.clone(),
                label: band_label(&row.rule_type),
                rules: vec![view],
            }),
        }
    }
    bands
}
