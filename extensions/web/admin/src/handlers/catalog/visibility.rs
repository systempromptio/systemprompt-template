//! Who reaches a catalog entity, as a badge.
//!
//! The badge answers one question at a glance: is this thing open to everyone,
//! or is it fenced to named groups and projects? The nearest declared ruleset
//! wins — a rule written directly against the plugin or skill overrides
//! whatever the marketplaces carrying it declare, because that is the order the
//! resolver applies them in. A row with no declaration anywhere is reported as
//! restricted rather than open: silence is not a grant.

use std::collections::BTreeSet;

use serde::Serialize;

use crate::repositories::marketplace::manifests::MarketplaceConfigSummary;
use crate::types::access_control::{AccessControlRule, AccessDecision};

const EVERYONE_ROLE: &str = "user";

#[derive(Debug, Clone, Default, Serialize)]
pub(crate) struct VisibilityView {
    pub is_public: bool,
    pub label: String,
    pub roles: Vec<String>,
    pub groups: Vec<String>,
    pub projects: Vec<String>,
    pub has_scope: bool,
}

#[derive(Debug, Default)]
struct Bands {
    roles: BTreeSet<String>,
    groups: BTreeSet<String>,
    projects: BTreeSet<String>,
}

impl Bands {
    fn is_empty(&self) -> bool {
        self.roles.is_empty() && self.groups.is_empty() && self.projects.is_empty()
    }

    fn add(&mut self, rule_type: &str, value: &str) {
        match rule_type {
            "role" => self.roles.insert(value.to_owned()),
            "group" => self.groups.insert(value.to_owned()),
            "project" => self.projects.insert(value.to_owned()),
            _ => false,
        };
    }
}

fn direct_bands(rules: &[AccessControlRule], entity_type: &str, entity_id: &str) -> Bands {
    let mut bands = Bands::default();
    for rule in rules.iter().filter(|r| {
        r.entity_type == entity_type
            && r.entity_id == entity_id
            && matches!(r.access, AccessDecision::Allow)
    }) {
        bands.add(rule.rule_type.as_str(), &rule.rule_value);
    }
    bands
}

fn manifest_bands(manifests: &[&MarketplaceConfigSummary]) -> Bands {
    let mut bands = Bands::default();
    for manifest in manifests {
        for role in &manifest.access.roles {
            bands.add("role", role);
        }
        for band in &manifest.access.bands {
            if band.access != "allow" {
                continue;
            }
            for value in &band.values {
                bands.add(&band.rule_type, value);
            }
        }
        if manifest.access.default_included {
            bands.add("role", EVERYONE_ROLE);
        }
    }
    bands
}

fn to_view(bands: Bands) -> VisibilityView {
    let is_public = bands.roles.contains(EVERYONE_ROLE);
    let scope_count = bands.groups.len() + bands.projects.len();
    let label = if is_public {
        "All users".to_owned()
    } else if scope_count > 0 {
        format!(
            "{scope_count} scope{}",
            if scope_count == 1 { "" } else { "s" }
        )
    } else if bands.roles.is_empty() {
        "Restricted".to_owned()
    } else {
        bands.roles.iter().cloned().collect::<Vec<_>>().join(", ")
    };
    VisibilityView {
        is_public,
        label,
        has_scope: scope_count > 0,
        roles: bands.roles.into_iter().collect(),
        groups: bands.groups.into_iter().collect(),
        projects: bands.projects.into_iter().collect(),
    }
}

// Why: The badge for one entity. `carriers` are the marketplaces that ship it,
// consulted only when nothing is declared against the entity itself.
pub(crate) fn visibility_for(
    rules: &[AccessControlRule],
    entity_type: &str,
    entity_id: &str,
    carriers: &[&MarketplaceConfigSummary],
) -> VisibilityView {
    let direct = direct_bands(rules, entity_type, entity_id);
    if direct.is_empty() {
        return to_view(manifest_bands(carriers));
    }
    to_view(direct)
}

// Why: The marketplaces that carry a given plugin.
pub(crate) fn carriers_of_plugin<'a>(
    manifests: &'a [MarketplaceConfigSummary],
    plugin_id: &str,
) -> Vec<&'a MarketplaceConfigSummary> {
    manifests
        .iter()
        .filter(|m| m.plugins.iter().any(|p| p == plugin_id))
        .collect()
}

// Why: The marketplaces that carry a skill, via the plugins that ship it.
pub(crate) fn carriers_of_skill<'a>(
    manifests: &'a [MarketplaceConfigSummary],
    plugin_ids: &[String],
) -> Vec<&'a MarketplaceConfigSummary> {
    manifests
        .iter()
        .filter(|m| m.plugins.iter().any(|p| plugin_ids.contains(p)))
        .collect()
}
