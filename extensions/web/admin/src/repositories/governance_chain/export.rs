//! Database → `governance/config.yaml`: the staged chain rendered as the
//! file core loads, and the drift between the two.

use serde_yaml::{Mapping, Value};

use super::declared::{DeclaredChain, entry_fingerprint};

#[derive(Debug, Clone)]
pub struct ChangedStage {
    pub id: String,
    pub fields: Vec<String>,
    pub in_code: String,
    pub in_db: String,
}

#[derive(Debug, Default, Clone)]
pub struct ChainDrift {
    pub settings_changed: Vec<String>,
    pub missing_in_db: Vec<String>,
    pub only_in_db: Vec<String>,
    pub changed: Vec<ChangedStage>,
    pub reordered: Option<(Vec<String>, Vec<String>)>,
}

impl ChainDrift {
    #[must_use]
    pub const fn is_clean(&self) -> bool {
        self.settings_changed.is_empty()
            && self.missing_in_db.is_empty()
            && self.only_in_db.is_empty()
            && self.changed.is_empty()
            && self.reordered.is_none()
    }

    #[must_use]
    pub fn total(&self) -> usize {
        self.settings_changed.len()
            + self.missing_in_db.len()
            + self.only_in_db.len()
            + self.changed.len()
            + usize::from(self.reordered.is_some())
    }
}

#[must_use]
pub fn summarise_stage(enabled: bool, mode: &str) -> String {
    format!("{} · {mode}", if enabled { "on" } else { "off" })
}

#[must_use]
pub fn compute_chain_drift(declared: &DeclaredChain, staged: &DeclaredChain) -> ChainDrift {
    let mut drift = ChainDrift::default();
    if declared.enabled != staged.enabled {
        drift.settings_changed.push("enabled".to_owned());
    }
    if declared.mode != staged.mode {
        drift.settings_changed.push("mode".to_owned());
    }
    for d in &declared.policies {
        match staged.find(&d.id) {
            None => drift.missing_in_db.push(d.id.clone()),
            Some(s) => {
                let mut fields = Vec::new();
                if d.enabled != s.enabled {
                    fields.push("enabled".to_owned());
                }
                if d.mode != s.mode {
                    fields.push("mode".to_owned());
                }
                if entry_fingerprint(&d.entry) != entry_fingerprint(&s.entry) {
                    fields.push("params".to_owned());
                }
                if !fields.is_empty() {
                    drift.changed.push(ChangedStage {
                        id: d.id.clone(),
                        fields,
                        in_code: summarise_stage(d.enabled, &d.mode),
                        in_db: summarise_stage(s.enabled, &s.mode),
                    });
                }
            },
        }
    }
    for s in &staged.policies {
        if declared.find(&s.id).is_none() {
            drift.only_in_db.push(s.id.clone());
        }
    }
    let shared = |chain: &DeclaredChain, other: &DeclaredChain| -> Vec<String> {
        chain
            .policies
            .iter()
            .filter(|p| other.find(&p.id).is_some())
            .map(|p| p.id.clone())
            .collect()
    };
    let (code, db) = (shared(declared, staged), shared(staged, declared));
    if code != db {
        drift.reordered = Some((code, db));
    }
    drift
}

fn header(exported_at: chrono::DateTime<chrono::Utc>) -> String {
    format!(
        "# Governance policy chain. Core builds the chain from this file at boot\n\
         # (systemprompt::security::policy::GovernanceConfig::load); the console stages\n\
         # the same chain in `governance_chain` and /admin/sync compares the two.\n\
         # A change here is enforced after the next restart.\n\
         #\n\
         # Exported from the database on {} by the console. The operational\n\
         # commentary in the committed file is the operator's to keep on merge.\n\n",
        exported_at.format("%Y-%m-%d")
    )
}

// Why: `enabled`/`mode` are written back into each entry from the row's
// columns, so a console edit to either lands in the file even though the
// stored entry still carries the value it was declared with.
#[must_use]
pub fn render_chain_export(
    chain: &DeclaredChain,
    exported_at: chrono::DateTime<chrono::Utc>,
) -> String {
    let mut governance = Mapping::new();
    governance.insert(Value::from("enabled"), Value::Bool(chain.enabled));
    governance.insert(Value::from("mode"), Value::from(chain.mode.clone()));
    let policies = chain
        .policies
        .iter()
        .map(|p| {
            let mut entry = p.entry.as_mapping().cloned().unwrap_or_default();
            entry.insert(Value::from("id"), Value::from(p.id.clone()));
            entry.insert(Value::from("enabled"), Value::Bool(p.enabled));
            entry.insert(Value::from("mode"), Value::from(p.mode.clone()));
            Value::Mapping(entry)
        })
        .collect();
    governance.insert(Value::from("policies"), Value::Sequence(policies));
    let mut root = Mapping::new();
    root.insert(Value::from("governance"), Value::Mapping(governance));
    // Why: discard-ok: a mapping of values that already parsed serialises
    let body = serde_yaml::to_string(&Value::Mapping(root)).unwrap_or_default();
    format!("{}{body}", header(exported_at))
}
