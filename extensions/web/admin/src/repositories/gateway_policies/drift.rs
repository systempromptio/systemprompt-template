//! Declared policies against the rows: one difference per policy, and
//! which fields disagree.
//!
//! The difference is field-level so the row names what moved —
//! `quota_windows` against `safety.block_categories` is the difference
//! between a spend and a scanner decision, without diffing two JSON blobs.
//!
//! Comparison is on the normalised spec (monthly windows folded to their
//! sentinel), so the daily rewrite is invisible here; a console edit to a
//! ceiling or a scanner list is not. A row the file does not name is an
//! orphan whatever wrote it — the console is the only other writer, and
//! *Overwrite from code* deletes it exactly as the boot seed would.

use serde::Serialize;
use systemprompt::ai::{GatewayPolicyEntry, GatewayPolicySpec};

use super::declared::DeclaredPolicies;
use super::month_window::normalise_spec;
use super::rows::PolicyRow;

#[derive(Debug, Clone, Serialize)]
pub struct ChangedPolicy {
    pub name: String,
    pub fields: Vec<String>,
    pub in_code: String,
    pub in_db: String,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PolicyDrift {
    pub missing_in_db: Vec<String>,
    pub only_in_db: Vec<String>,
    pub changed: Vec<ChangedPolicy>,
}

impl PolicyDrift {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.missing_in_db.is_empty() && self.only_in_db.is_empty() && self.changed.is_empty()
    }
}

fn windows_summary(spec: &GatewayPolicySpec) -> String {
    if spec.quota_windows.is_empty() {
        return "no windows".to_owned();
    }
    spec.quota_windows
        .iter()
        .map(|w| {
            let mut parts = vec![format!("{}/{}s", w.subject, w.window_seconds)];
            if let Some(n) = w.max_requests {
                parts.push(format!("{n} req"));
            }
            if let Some(n) = w.max_cost_microdollars {
                parts.push(systemprompt_web_shared::format::format_cost(n));
            }
            parts.join(" ")
        })
        .collect::<Vec<_>>()
        .join("; ")
}

#[must_use]
pub fn summarise_policy(entry_enabled: bool, priority: i32, spec: &GatewayPolicySpec) -> String {
    format!(
        "{} · p{priority} · quota {:?} · {} · safety {:?} [{}] block {:?}/{:?}",
        if entry_enabled { "on" } else { "off" },
        spec.quota_mode,
        windows_summary(spec),
        spec.safety.mode,
        spec.safety.scanners.join(","),
        spec.safety.block_categories,
        spec.safety.block_response_categories,
    )
    .to_lowercase()
}

fn changed_fields(declared: &GatewayPolicyEntry, row: &PolicyRow) -> Vec<String> {
    let db = normalise_spec(&row.spec);
    let d = &declared.spec;
    [
        ("enabled", declared.enabled != row.enabled),
        ("priority", declared.priority != row.priority),
        ("quota_mode", d.quota_mode != db.quota_mode),
        (
            "quota_windows",
            comparison_key(&d.quota_windows) != comparison_key(&db.quota_windows),
        ),
        ("safety.mode", d.safety.mode != db.safety.mode),
        ("safety.scanners", d.safety.scanners != db.safety.scanners),
        (
            "safety.block_categories",
            d.safety.block_categories != db.safety.block_categories,
        ),
        (
            "safety.block_response_categories",
            d.safety.block_response_categories != db.safety.block_response_categories,
        ),
        ("safety.history", d.safety.history != db.safety.history),
        (
            "safety.heuristic",
            comparison_key(&d.safety.heuristic) != comparison_key(&db.safety.heuristic),
        ),
    ]
    .into_iter()
    .filter(|(_, differs)| *differs)
    .map(|(field, _)| field.to_owned())
    .collect()
}

// JSON: variable-shape comparison key, never leaves this module
fn comparison_key<T: Serialize>(v: &T) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

fn changed_policy(entry: &GatewayPolicyEntry, row: &PolicyRow) -> Option<ChangedPolicy> {
    let fields = changed_fields(entry, row);
    if fields.is_empty() {
        return None;
    }
    Some(ChangedPolicy {
        name: entry.name.clone(),
        fields,
        in_code: summarise_policy(entry.enabled, entry.priority, &entry.spec),
        in_db: summarise_policy(row.enabled, row.priority, &normalise_spec(&row.spec)),
    })
}

#[must_use]
pub fn compute_policy_drift(declared: &DeclaredPolicies, rows: &[PolicyRow]) -> PolicyDrift {
    let mut drift = PolicyDrift::default();
    for entry in &declared.entries {
        let Some(row) = rows.iter().find(|r| r.name == entry.name) else {
            drift.missing_in_db.push(entry.name.clone());
            continue;
        };
        drift.changed.extend(changed_policy(entry, row));
    }
    drift.only_in_db = rows
        .iter()
        .filter(|row| declared.find(&row.name).is_none())
        .map(|row| row.name.clone())
        .collect();
    drift
}
