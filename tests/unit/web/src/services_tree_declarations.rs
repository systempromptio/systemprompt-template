//! The shipped `services/` tree, read exactly as boot and the sync page read
//! it: every declaration parses, projects and round-trips, and none of them
//! names a person.
//!
//! These tests open the real files rather than fixtures, so a commit that
//! makes `rules.yaml` name a marketplace the tree does not define, or a
//! group `groups.yaml` no longer declares, fails here before it fails on the
//! access-control page. The catalog is loaded from the tree the way the page
//! loads the composed root, which is also how a dangling plugin include is
//! caught as a composition error rather than as an empty marketplace list.

use std::collections::BTreeSet;
use std::path::PathBuf;

use systemprompt::loader::ConfigLoader;
use systemprompt::models::services::ServicesConfig;
use systemprompt_security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_admin::repositories::access_control::declared::{
    DeclaredInputs, DeclaredSet, build_declared_set,
};
use systemprompt_web_admin::repositories::access_control::drift::{BandRuleRow, EntityDefaultRow};
use systemprompt_web_admin::repositories::access_control::export::{Owners, render_export};
use systemprompt_web_admin::repositories::config::gateway::dispatchable_route_ids;
use systemprompt_web_admin::repositories::config::rules_yaml_loader::{
    RULES_FILE, parse_rules_doc,
};
use systemprompt_web_admin::repositories::config::rules_yaml_types::RulesDoc;
use systemprompt_web_admin::repositories::gateway_policies::declared::{
    POLICIES_FILE, parse_declared_policies,
};
use systemprompt_web_admin::repositories::gateway_policies::export::render_policies_export;
use systemprompt_web_admin::repositories::gateway_policies::month_window::{
    MONTH_WINDOW_SECONDS, normalise_spec,
};
use systemprompt_web_admin::repositories::gateway_policies::rows::PolicyRow;
use systemprompt_web_admin::repositories::sync::groups_db::{
    parse_groups_doc, render_groups_export,
};
use systemprompt_web_admin::repositories::sync::groups_drift::{MappingRow, MemberSetRow};

use crate::support::repo_root;

fn services_dir() -> PathBuf {
    repo_root().join("services")
}

fn read(relative: &str) -> String {
    let path = services_dir().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("{}: {e}", path.display()))
}

// Why: the same load boot performs on the base tree, so a tree that does not
// compose fails this test with the composition error — not with a rules.yaml
// error one page later.
fn services() -> ServicesConfig {
    ConfigLoader::load_from_path(&services_dir().join("config/config.yaml"))
        .unwrap_or_else(|e| panic!("services tree could not be composed: {e}"))
}

fn catalog(services: &ServicesConfig) -> RegisteredEntities {
    RegisteredEntities::new()
        .with_kind(EntityKind::GatewayRoute, dispatchable_route_ids(services))
        .with_kind(EntityKind::Plugin, services.plugins.keys().cloned())
        .with_kind(EntityKind::Skill, services.skills.skills.keys().cloned())
        .with_kind(EntityKind::McpServer, services.mcp_servers.keys().cloned())
}

fn rules_doc() -> RulesDoc {
    parse_rules_doc(&read(RULES_FILE)).expect("rules.yaml parses and validates")
}

fn declared() -> (ServicesConfig, DeclaredSet) {
    let services = services();
    let routes = dispatchable_route_ids(&services);
    let marketplaces: Vec<String> = services
        .marketplaces
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect();
    let set = build_declared_set(
        &rules_doc(),
        &DeclaredInputs {
            gateway_routes: &routes,
            marketplace_ids: &marketplaces,
            registered: &catalog(&services),
        },
    )
    .unwrap_or_else(|e| panic!("rules.yaml does not project against the tree: {e}"));
    (services, set)
}

#[test]
fn rules_yaml_projects_against_the_tree_it_ships_with() {
    let (services, set) = declared();
    assert!(set.rule_count() > 0, "the file declares rules");
    let marketplaces: BTreeSet<&str> = services
        .marketplaces
        .keys()
        .map(systemprompt_web_shared::MarketplaceId::as_str)
        .collect();
    for (entity_type, entity_id) in set.entities.keys() {
        if entity_type == "marketplace" {
            assert!(
                marketplaces.contains(entity_id.as_str()),
                "marketplace/{entity_id} is declared but not defined"
            );
        }
    }
    for awaiting in &set.awaiting {
        assert!(
            awaiting.owner.starts_with("bundle:"),
            "{}/{} waits on a bundle, not on this tree",
            awaiting.entity_type,
            awaiting.entity_id
        );
        assert!(
            !marketplaces.contains(awaiting.entity_id.as_str()),
            "{} is both local and bundled",
            awaiting.entity_id
        );
    }
}

#[test]
fn every_group_and_project_the_rules_name_is_declared_in_groups_yaml() {
    let groups = parse_groups_doc(&read("web/config/groups.yaml")).expect("groups.yaml");
    let mut group_ids: BTreeSet<&str> = groups.groups.iter().map(|g| g.id.as_str()).collect();
    group_ids.insert("unassigned");
    let project_ids: BTreeSet<&str> = groups.projects.iter().map(|p| p.id.as_str()).collect();
    let (_, set) = declared();
    for rule in set.rules.values() {
        let known = match rule.key.rule_type.as_str() {
            "group" => &group_ids,
            "project" => &project_ids,
            _ => continue,
        };
        assert!(
            known.contains(rule.key.rule_value.as_str()),
            "{}/{} names {} '{}', which groups.yaml does not declare",
            rule.key.entity_type,
            rule.key.entity_id,
            rule.key.rule_type,
            rule.key.rule_value
        );
    }
}

#[test]
fn groups_yaml_round_trips_through_the_export() {
    let doc = parse_groups_doc(&read("web/config/groups.yaml")).expect("groups.yaml");
    let mut sets = Vec::new();
    let mut mappings = Vec::new();
    for (kind, defs) in [("group", &doc.groups), ("project", &doc.projects)] {
        for def in defs {
            sets.push(MemberSetRow {
                kind: kind.to_owned(),
                id: def.id.clone(),
                name: def.name.clone(),
                description: def.description.clone(),
                source: "yaml".to_owned(),
                is_system: false,
            });
            for ad_group in &def.ad_groups {
                mappings.push(MappingRow {
                    kind: kind.to_owned(),
                    ad_group: ad_group.clone(),
                    set_id: def.id.clone(),
                    source: "yaml".to_owned(),
                });
            }
        }
    }
    let exported = render_groups_export(&sets, &mappings);
    let back = parse_groups_doc(&exported).expect("the export parses as the file");
    let shape = |d: &systemprompt_web_admin::repositories::config::groups_yaml_types::GroupsDoc| {
        let mut out: Vec<(String, String, String, Vec<String>)> = Vec::new();
        for (kind, defs) in [("group", &d.groups), ("project", &d.projects)] {
            for def in defs {
                out.push((
                    kind.to_owned(),
                    def.id.clone(),
                    def.name.clone(),
                    def.ad_groups.clone(),
                ));
            }
        }
        out.sort();
        out
    };
    assert_eq!(shape(&back), shape(&doc));
}

#[test]
fn policies_yaml_parses_and_the_month_sentinel_survives_normalisation() {
    let declared = parse_declared_policies(&read(POLICIES_FILE)).expect("policies.yaml");
    assert!(!declared.entries.is_empty(), "the file declares policies");
    for entry in &declared.entries {
        let normalised = normalise_spec(&entry.spec);
        assert_eq!(
            serde_json::to_value(&normalised).expect("spec"),
            serde_json::to_value(&entry.spec).expect("spec"),
            "policy '{}' is already in declared form",
            entry.name
        );
        for window in &entry.spec.quota_windows {
            assert!(
                window.window_seconds <= MONTH_WINDOW_SECONDS
                    || window.window_seconds > 2 * MONTH_WINDOW_SECONDS,
                "policy '{}' hand-sets a window inside the live month range",
                entry.name
            );
        }
    }
}

// Why: people are the one thing this mechanism must never carry into code.
// Each export is a pure function over what the database holds, so it is fed
// the shapes a person would arrive in and must leave them all out.
#[test]
fn no_export_carries_a_person() {
    let rules = [
        BandRuleRow {
            id: "u1".to_owned(),
            entity_type: "mcp_server".to_owned(),
            entity_id: "systemprompt".to_owned(),
            rule_type: "user".to_owned(),
            rule_value: "someone@example.com".to_owned(),
            access: systemprompt_security::authz::Access::Allow,
            justification: Some("override".to_owned()),
            source: "dashboard".to_owned(),
            valid_until: None,
        },
        BandRuleRow {
            id: "g1".to_owned(),
            entity_type: "mcp_server".to_owned(),
            entity_id: "systemprompt".to_owned(),
            rule_type: "group".to_owned(),
            rule_value: "engineering".to_owned(),
            access: systemprompt_security::authz::Access::Allow,
            justification: Some("team".to_owned()),
            source: "yaml".to_owned(),
            valid_until: None,
        },
    ];
    let entities = [EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: "systemprompt".to_owned(),
        default_included: false,
        source: "yaml".to_owned(),
    }];
    let access = render_export(&rules, &entities, &Owners::default());
    assert!(!access.contains("someone@example.com"));
    assert!(!access.contains("user:"));
    assert!(access.contains("group: [engineering]") || access.contains("- engineering"));

    let sets = [MemberSetRow {
        kind: "group".to_owned(),
        id: "engineering".to_owned(),
        name: "Engineering".to_owned(),
        description: None,
        source: "dashboard".to_owned(),
        is_system: false,
    }];
    let groups = render_groups_export(&sets, &[]);
    for key in [
        "members",
        "group_members",
        "project_members",
        "user_id",
        "roles",
    ] {
        assert!(!groups.contains(key), "groups export carries `{key}`");
    }

    let policies = render_policies_export(
        &[PolicyRow {
            name: "default".to_owned(),
            spec: systemprompt::ai::GatewayPolicySpec::default(),
            enabled: true,
            priority: 0,
            updated_at: chrono::Utc::now(),
        }],
        chrono::Utc::now(),
    );
    assert!(!policies.contains("user_id"));

    for file in [RULES_FILE, "web/config/groups.yaml", POLICIES_FILE] {
        let text = read(file);
        for key in ["user:", "members:", "user_id:", "manual_roles"] {
            assert!(!text.contains(key), "{file} declares a person via `{key}`");
        }
    }
}
