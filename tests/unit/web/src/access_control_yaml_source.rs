//! The access-control page's rule-source column.
//!
//! The column claims a rule is "declared in YAML", so the thing worth pinning
//! is the reconstruction: a glob in roles.yaml has to cover the entity ids it
//! expands to, a marketplace's own `access` block counts as declared, and a
//! rule naming a subject no file mentions must come back as an edit made in
//! the instance's database.

use std::fs;

use systemprompt_web_admin::repositories::access_control::yaml_declared::load_declared_rules;
use tempfile::TempDir;

fn services_tree() -> TempDir {
    let dir = TempDir::new().expect("tempdir");
    let root = dir.path();
    fs::create_dir_all(root.join("access-control")).expect("access-control dir");
    fs::create_dir_all(root.join("marketplaces/astound-europe-dev")).expect("marketplace dir");
    fs::write(
        root.join("access-control/roles.yaml"),
        "rules:\n  - entity_type: mcp_server\n    entity_id: systemprompt\n    roles: [admin, \
         platform_admin]\n  - entity_type: gateway_route\n    entity_match: \"claude-*\"\n    \
         roles: [developer]\n",
    )
    .expect("roles.yaml");
    fs::write(
        root.join("access-control/groups.yaml"),
        "grants:\n  - entity_type: skill\n    entity_id: dev_plan\n    members: [europe-devs]\n",
    )
    .expect("groups.yaml");
    fs::write(
        root.join("marketplaces/astound-europe-dev/config.yaml"),
        "marketplace:\n  id: astound-europe-dev\n  access:\n    roles: []\n    rules:\n      - \
         rule_type: group\n        values: [europe-devs]\n",
    )
    .expect("marketplace config");
    dir
}

#[test]
fn an_exact_role_rule_is_declared() {
    let dir = services_tree();
    let declared = load_declared_rules(dir.path());
    assert!(declared.declares("mcp_server", "systemprompt", "role", "admin"));
    assert!(declared.declares("mcp_server", "systemprompt", "role", "platform_admin"));
}

#[test]
fn a_glob_covers_the_ids_it_expands_to() {
    let dir = services_tree();
    let declared = load_declared_rules(dir.path());
    assert!(declared.declares("gateway_route", "claude-sonnet-4", "role", "developer"));
    assert!(!declared.declares("gateway_route", "gpt-4o", "role", "developer"));
}

#[test]
fn a_marketplace_declares_its_own_access_block() {
    let dir = services_tree();
    let declared = load_declared_rules(dir.path());
    assert!(declared.declares("marketplace", "astound-europe-dev", "group", "europe-devs"));
}

#[test]
fn a_member_grant_file_declares_its_band() {
    let dir = services_tree();
    let declared = load_declared_rules(dir.path());
    assert!(declared.declares("skill", "dev_plan", "group", "europe-devs"));
}

#[test]
fn a_subject_no_file_mentions_is_not_declared() {
    let dir = services_tree();
    let declared = load_declared_rules(dir.path());
    assert!(!declared.declares("mcp_server", "systemprompt", "role", "user"));
    assert!(!declared.declares("mcp_server", "salesforce", "role", "admin"));
}

#[test]
fn a_missing_services_tree_declares_nothing_rather_than_panicking() {
    let declared = load_declared_rules(std::path::Path::new("/nonexistent-services-tree"));
    assert!(!declared.declares("mcp_server", "systemprompt", "role", "admin"));
}
