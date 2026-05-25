//! Wire types for the bootstrap ACL YAML loader.
//!
//! Schema invariants that cannot be expressed in serde's derive (exactly-one
//! of `entity_id` / `entity_match`) are enforced through a custom
//! `Deserialize` impl, so a malformed YAML file fails at parse time rather
//! than producing a runtime error during the apply phase.

use serde::{Deserialize, Deserializer};
use systemprompt_security::authz::{Access, EntityKind};

#[derive(Debug)]
pub enum RuleTarget {
    Id(String),
    Match(String),
}

#[derive(Debug)]
pub struct YamlRule {
    pub entity_type: EntityKind,
    pub target: RuleTarget,
    pub access: Access,
    pub default_included: bool,
    pub roles: Vec<String>,
    pub departments: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct YamlRuleWire {
    entity_type: EntityKind,
    #[serde(default)]
    entity_id: Option<String>,
    #[serde(default)]
    entity_match: Option<String>,
    #[serde(default = "default_allow")]
    access: Access,
    #[serde(default)]
    default_included: bool,
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    departments: Vec<String>,
}

const fn default_allow() -> Access {
    Access::Allow
}

impl<'de> Deserialize<'de> for YamlRule {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = YamlRuleWire::deserialize(d)?;
        let target = match (w.entity_id, w.entity_match) {
            (Some(id), None) => RuleTarget::Id(id),
            (None, Some(pattern)) => RuleTarget::Match(pattern),
            (Some(_), Some(_)) => {
                return Err(serde::de::Error::custom(format!(
                    "rule for entity_type={} sets both entity_id and entity_match; pick one",
                    w.entity_type.as_str()
                )));
            }
            (None, None) => {
                return Err(serde::de::Error::custom(format!(
                    "rule for entity_type={} sets neither entity_id nor entity_match",
                    w.entity_type.as_str()
                )));
            }
        };
        Ok(Self {
            entity_type: w.entity_type,
            target,
            access: w.access,
            default_included: w.default_included,
            roles: w.roles,
            departments: w.departments,
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepartmentsDoc {
    #[serde(default)]
    pub departments: Vec<YamlDepartment>,
    #[serde(default)]
    pub rules: Vec<YamlRule>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RolesDoc {
    #[serde(default)]
    pub rules: Vec<YamlRule>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct YamlDepartment {
    pub name: String,
    #[serde(default)]
    pub description: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LoadReport {
    pub departments_upserted: usize,
    pub rules_upserted: usize,
    pub rules_skipped: usize,
    pub glob_rules_expanded: usize,
    pub glob_entities_matched: usize,
}

/// Tiny `*`-glob matcher: `"*"`, `"prefix*"`, `"*suffix"`, `"a*b"`. Pulling
/// in `globset` for this single call site is overkill.
pub fn glob_matches(pattern: &str, candidate: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    if parts.len() == 1 {
        return pattern == candidate;
    }
    let first = parts[0];
    let last = parts[parts.len() - 1];
    if !candidate.starts_with(first) || !candidate.ends_with(last) {
        return false;
    }
    if first.len() + last.len() > candidate.len() {
        return false;
    }
    let mut cursor = first.len();
    let end = candidate.len() - last.len();
    for part in &parts[1..parts.len() - 1] {
        if part.is_empty() {
            continue;
        }
        match candidate[cursor..end].find(part) {
            Some(pos) => cursor += pos + part.len(),
            None => return false,
        }
    }
    true
}
