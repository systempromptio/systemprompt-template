//! The bands of one entity as the page words them: the ladder's labels and
//! order, the allow/deny chips, and the resolver's outcome in one sentence.

use super::view::BandChipsView;
use crate::repositories::access_control::rules::LedgerRuleRow;
use crate::types::access_control::AccessDecision;

// Why: the ladder's own words for each band, so the page and the docs agree.
pub(super) fn band_label(rule_type: &str) -> &'static str {
    match rule_type {
        "user" => "person",
        "project" => "project",
        "group" => "group",
        "connector" => "connected server",
        "role" => "role",
        _ => "band",
    }
}

// Why: ladder order, narrowest first — the order the resolver consults them.
pub(super) fn band_rank(rule_type: &str) -> u8 {
    match rule_type {
        "user" => 0,
        "project" => 1,
        "group" => 2,
        "connector" => 3,
        "role" => 4,
        _ => 5,
    }
}

pub(super) fn chips(rows: &[&LedgerRuleRow], access: AccessDecision) -> Vec<BandChipsView> {
    let mut bands: Vec<BandChipsView> = Vec::new();
    for row in rows.iter().filter(|r| r.access == access) {
        match bands.iter_mut().find(|b| b.band == row.rule_type) {
            Some(band) => band.values.push(row.rule_value.clone()),
            None => bands.push(BandChipsView {
                band: row.rule_type.clone(),
                label: band_label(&row.rule_type),
                values: vec![row.rule_value.clone()],
            }),
        }
    }
    bands.sort_by_key(|b| band_rank(&b.band));
    for band in &mut bands {
        band.values.sort();
    }
    bands
}

fn phrase(band: &BandChipsView) -> String {
    let list = band.values.join(", ");
    match band.band.as_str() {
        "user" => format!("is {list}"),
        "project" => format!("is on project {list}"),
        "group" => format!("is in group {list}"),
        "connector" => format!("has connected {list}"),
        "role" => format!("holds role {list}"),
        _ => format!("matches {} {list}", band.band),
    }
}

// Why: the resolver's outcome in one sentence, built from the bands in ladder
// order. It is deliberately literal: "or" between allow bands because any one
// suffices, the deny clause first because a deny inside a band wins.
pub(super) fn resolution(allow: &[BandChipsView], deny: &[BandChipsView], open: bool) -> String {
    let mut out = String::new();
    if !deny.is_empty() {
        let d: Vec<String> = deny.iter().map(phrase).collect();
        out.push_str(&format!("Refused to anyone who {}. ", d.join(" or ")));
    }
    if allow.is_empty() {
        out.push_str(if open {
            "No allow rule: everyone else reaches it because the entity is open."
        } else {
            "No allow rule and the entity is closed: nobody reaches it directly."
        });
        return out;
    }
    let a: Vec<String> = allow.iter().map(phrase).collect();
    out.push_str(&format!(
        "A person reaches this if they {}. ",
        a.join(", or ")
    ));
    out.push_str(if open {
        "Everyone else reaches it too — the entity is open."
    } else {
        "Everyone else is refused — the entity is closed."
    });
    out
}
