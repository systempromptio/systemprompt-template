//! Wire types for the bootstrap group/project definition loader
//! (`services/web/config/groups.yaml`).
//!
//! The file declares the groups and projects an installation ships with, and
//! the AD groups the directory maps into each. It defines *membership shape*,
//! not entitlement: which entity a member reaches is authored separately in
//! `services/access-control/{groups,projects}.yaml` and in each marketplace's
//! own `access.rules` block.

use serde::Deserialize;

/// One group or project this installation ships.
///
/// Both halves of the file share this shape — a group carries people and
/// marketplace entitlement, a project carries work attribution, and the
/// directory maps AD groups into either the same way.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemberSetDef {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    // Why: the AD group names that place a signing-in user here. Empty is
    // legitimate: a group an operator fills by hand from the dashboard.
    #[serde(default)]
    pub ad_groups: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GroupsDoc {
    #[serde(default)]
    pub groups: Vec<MemberSetDef>,
    #[serde(default)]
    pub projects: Vec<MemberSetDef>,
}

impl GroupsDoc {
    // Why: an id is a DB primary key with a CHECK constraint behind it, and a
    // duplicate id in the file would make the later definition silently win.
    // Both are caught here so a bad file fails the boot with a readable
    // message instead of a constraint violation.
    pub fn validate(&self) -> Result<(), String> {
        validate_set("groups", &self.groups)?;
        validate_set("projects", &self.projects)
    }
}

fn validate_set(label: &str, defs: &[MemberSetDef]) -> Result<(), String> {
    let mut seen: Vec<&str> = Vec::new();
    for def in defs {
        if !is_valid_id(&def.id) {
            return Err(format!(
                "{label}: id '{}' must be lowercase alphanumeric with '-' or '_', starting with a \
                 letter or digit, at most 64 characters",
                def.id
            ));
        }
        if def.name.trim().is_empty() {
            return Err(format!("{label}: '{}' must carry a name", def.id));
        }
        if seen.contains(&def.id.as_str()) {
            return Err(format!("{label}: id '{}' is declared twice", def.id));
        }
        seen.push(&def.id);
    }
    Ok(())
}

// Why: mirrors the `CHECK (id ~ '^[a-z0-9][a-z0-9_-]{0,63}$')` on `groups` and
// `projects`. The two must agree — the loader rejects anything this accepts
// but the constraint does not.
#[must_use]
pub fn is_valid_id(id: &str) -> bool {
    let mut chars = id.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    id.len() <= 64
        && (first.is_ascii_lowercase() || first.is_ascii_digit())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
}

#[derive(Debug, Default, Clone, Copy)]
pub struct GroupsLoadReport {
    pub groups: usize,
    pub projects: usize,
    pub mappings: usize,
}
