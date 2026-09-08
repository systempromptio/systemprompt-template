//! Which rules the YAML on disk declares, so the ledger can say which ones it
//! does not.
//!
//! `access_control_rules` has no provenance column and cannot grow one
//! usefully: the bootstrap loaders upsert, so the last writer of a row is not
//! the only author of it. What is decidable is the other direction — rebuild
//! the set the loaders would write from the four declarative sources and
//! treat a rule outside that set as an edit made in this instance's database.
//! A rule inside it may still have been re-saved from the dashboard; the
//! column says "declared in YAML", not "never touched here".
//!
//! The four sources are the ones
//! [`crate::repositories::config::acl_yaml_loader`] reads: `access-control/
//! roles.yaml`, the member-grant and link-gate files beside it, each
//! marketplace's own `access` block, and every Slack app's
//! `authz.allowed_roles`.

use std::collections::HashSet;
use std::path::Path;

use serde::Deserialize;

/// The declarative rule set, keyed for lookup by the ledger.
#[derive(Debug, Default, Clone)]
pub struct DeclaredRules {
    exact: HashSet<(String, String, String, String)>,
    globs: Vec<(String, String, String, String)>,
}

impl DeclaredRules {
    // Why: Whether YAML declares this (entity type, entity, subject kind, subject).
    #[must_use]
    pub fn declares(
        &self,
        entity_type: &str,
        entity_id: &str,
        rule_type: &str,
        rule_value: &str,
    ) -> bool {
        let key = (
            entity_type.to_owned(),
            entity_id.to_owned(),
            rule_type.to_owned(),
            rule_value.to_owned(),
        );
        if self.exact.contains(&key) {
            return true;
        }
        self.globs.iter().any(|(kind, pattern, rt, rv)| {
            kind == entity_type
                && rt == rule_type
                && rv == rule_value
                && glob_matches(pattern, entity_id)
        })
    }

    fn push(&mut self, entity_type: &str, target: &Target, rule_type: &str, rule_value: &str) {
        let row = (
            entity_type.to_owned(),
            target.value().to_owned(),
            rule_type.to_owned(),
            rule_value.to_owned(),
        );
        match target {
            Target::Id(_) => {
                self.exact.insert(row);
            },
            Target::Glob(_) => self.globs.push(row),
        }
    }
}

enum Target {
    Id(String),
    Glob(String),
}

impl Target {
    fn value(&self) -> &str {
        match self {
            Self::Id(v) | Self::Glob(v) => v,
        }
    }
}

// Why: `*` is the only wildcard roles.yaml uses and the only one core's
// expansion supports, so the matcher is a split on it rather than a regex
// dependency for one character.
fn glob_matches(pattern: &str, value: &str) -> bool {
    let parts: Vec<&str> = pattern.split('*').collect();
    let Some((first, tail)) = parts.split_first() else {
        return false;
    };
    if tail.is_empty() {
        return pattern == value;
    }
    let Some(mut rest) = value.strip_prefix(first) else {
        return false;
    };
    let (last, middle) = tail.split_last().unwrap_or((&"", &[]));
    for part in middle {
        let Some(at) = rest.find(part) else {
            return false;
        };
        rest = &rest[at + part.len()..];
    }
    last.is_empty() || rest.ends_with(last)
}

// Why: Read every declarative source under `services_path`.
//
// Unreadable or malformed files are skipped rather than failed: a provenance
// column that cannot render is worth less than a page that does not load,
// and every one of these files is validated for real by the bootstrap loader
// at startup.
#[must_use]
pub fn load_declared_rules(services_path: &Path) -> DeclaredRules {
    let mut out = DeclaredRules::default();
    read_roles_file(services_path, &mut out);
    read_member_grants(
        services_path,
        "access-control/groups.yaml",
        "group",
        &mut out,
    );
    read_member_grants(
        services_path,
        "access-control/projects.yaml",
        "project",
        &mut out,
    );
    read_link_gates(
        services_path,
        "access-control/salesforce.yaml",
        "salesforce",
        &mut out,
    );
    read_marketplaces(services_path, &mut out);
    read_slack_apps(services_path, &mut out);
    out
}

fn parse<T: for<'de> Deserialize<'de>>(path: &Path) -> Option<T> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_yaml::from_str::<T>(&text).ok()
}

#[derive(Deserialize)]
struct RolesDoc {
    #[serde(default)]
    rules: Vec<RolesRule>,
}

#[derive(Deserialize)]
struct RolesRule {
    entity_type: String,
    #[serde(default)]
    entity_id: Option<String>,
    #[serde(default)]
    entity_match: Option<String>,
    #[serde(default)]
    roles: Vec<String>,
}

fn read_roles_file(services_path: &Path, out: &mut DeclaredRules) {
    let Some(doc) = parse::<RolesDoc>(&services_path.join("access-control/roles.yaml")) else {
        return;
    };
    for rule in doc.rules {
        let target = match (rule.entity_id, rule.entity_match) {
            (Some(id), _) => Target::Id(id),
            (None, Some(pattern)) => Target::Glob(pattern),
            (None, None) => continue,
        };
        for role in &rule.roles {
            out.push(&rule.entity_type, &target, "role", role);
        }
    }
}

#[derive(Deserialize)]
struct GrantsDoc {
    #[serde(default)]
    grants: Vec<Grant>,
}

#[derive(Deserialize)]
struct Grant {
    entity_type: String,
    entity_id: String,
    #[serde(default)]
    members: Vec<String>,
}

fn read_member_grants(services_path: &Path, file: &str, rule_type: &str, out: &mut DeclaredRules) {
    let Some(doc) = parse::<GrantsDoc>(&services_path.join(file)) else {
        return;
    };
    for grant in doc.grants {
        let target = Target::Id(grant.entity_id);
        for member in &grant.members {
            out.push(&grant.entity_type, &target, rule_type, member);
        }
    }
}

// Why: a link gate has one subject value — "linked" — written for every
// entity the file lists, so the file carries no value to read.
const LINKED_VALUE: &str = "linked";

fn read_link_gates(services_path: &Path, file: &str, rule_type: &str, out: &mut DeclaredRules) {
    let Some(doc) = parse::<GrantsDoc>(&services_path.join(file)) else {
        return;
    };
    for grant in doc.grants {
        let target = Target::Id(grant.entity_id);
        out.push(&grant.entity_type, &target, rule_type, LINKED_VALUE);
    }
}

#[derive(Deserialize)]
struct MarketplaceFile {
    marketplace: MarketplaceBody,
}

#[derive(Deserialize)]
struct MarketplaceBody {
    id: String,
    #[serde(default)]
    access: MarketplaceAccess,
}

#[derive(Default, Deserialize)]
struct MarketplaceAccess {
    #[serde(default)]
    roles: Vec<String>,
    #[serde(default)]
    rules: Vec<MarketplaceRule>,
}

#[derive(Deserialize)]
struct MarketplaceRule {
    rule_type: String,
    #[serde(default)]
    values: Vec<String>,
}

fn read_marketplaces(services_path: &Path, out: &mut DeclaredRules) {
    let Ok(entries) = std::fs::read_dir(services_path.join("marketplaces")) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(doc) = parse::<MarketplaceFile>(&entry.path().join("config.yaml")) else {
            continue;
        };
        let target = Target::Id(doc.marketplace.id);
        for role in &doc.marketplace.access.roles {
            out.push("marketplace", &target, "role", role);
        }
        for rule in &doc.marketplace.access.rules {
            for value in &rule.values {
                out.push("marketplace", &target, &rule.rule_type, value);
            }
        }
    }
}

#[derive(Deserialize)]
struct SlackDoc {
    #[serde(default)]
    slack_apps: std::collections::HashMap<String, SlackApp>,
}

#[derive(Deserialize)]
struct SlackApp {
    workspace_id: String,
    #[serde(default)]
    authz: SlackAuthz,
}

#[derive(Default, Deserialize)]
struct SlackAuthz {
    #[serde(default)]
    allowed_roles: Vec<String>,
}

fn read_slack_apps(services_path: &Path, out: &mut DeclaredRules) {
    let Ok(entries) = std::fs::read_dir(services_path.join("slack")) else {
        return;
    };
    for entry in entries.flatten() {
        let Some(doc) = parse::<SlackDoc>(&entry.path()) else {
            continue;
        };
        for app in doc.slack_apps.into_values() {
            let target = Target::Id(app.workspace_id);
            for role in &app.authz.allowed_roles {
                out.push("slack_workspace", &target, "role", role);
            }
        }
    }
}
