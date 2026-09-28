//! `rules.yaml` parsing and its projection into database rows.
//!
//! The projection is the contract everything else stands on: the drift engine
//! compares against it, the sync writes it, the export inverts it. Each test
//! pins one rule of that projection with no database in the way.

use systemprompt_security::authz::{Access, EntityKind, RegisteredEntities};
use systemprompt_web_admin::repositories::access_control::declared::{
    DeclaredInputs, DeclaredKey, build_declared_set,
};
use systemprompt_web_admin::repositories::config::rules_yaml_loader::parse_rules_doc;

fn key(entity_type: &str, entity_id: &str, rule_type: &str, rule_value: &str) -> DeclaredKey {
    DeclaredKey {
        entity_type: entity_type.to_owned(),
        entity_id: entity_id.to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: rule_value.to_owned(),
    }
}

fn project(
    yaml: &str,
    routes: &[&str],
    marketplaces: &[&str],
) -> Result<systemprompt_web_admin::repositories::access_control::declared::DeclaredSet, String> {
    let doc = parse_rules_doc(yaml).map_err(|e| e.to_string())?;
    let routes: Vec<String> = routes.iter().map(|r| (*r).to_owned()).collect();
    let marketplaces: Vec<String> = marketplaces.iter().map(|m| (*m).to_owned()).collect();
    build_declared_set(
        &doc,
        &DeclaredInputs {
            gateway_routes: &routes,
            marketplace_ids: &marketplaces,
            registered: &RegisteredEntities::default(),
        },
    )
    .map_err(|e| e.to_string())
}

#[test]
fn every_band_value_becomes_one_row_carrying_the_entity_why() {
    let set = project(
        "entities:\n  - entity: mcp_server/atlassian\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n      group: [india-devs, uk]\n    deny:\n      project: [storefront]\n",
        &[],
        &[],
    )
    .expect("projects");
    assert_eq!(set.rules.len(), 4);
    let uk = &set.rules[&key("mcp_server", "atlassian", "group", "uk")];
    assert_eq!(uk.access, Access::Allow);
    assert_eq!(uk.justification, "pilot");
    let denied = &set.rules[&key("mcp_server", "atlassian", "project", "storefront")];
    assert_eq!(denied.access, Access::Deny);
    let entity = &set.entities[&("mcp_server".to_owned(), "atlassian".to_owned())];
    assert!(!entity.default_included);
    assert!(set.governs("mcp_server", "atlassian"));
    assert!(!set.governs("mcp_server", "github"));
}

#[test]
fn a_band_may_override_the_why_and_the_default_may_be_open() {
    let set = project(
        "entities:\n  - entity: marketplace/commons\n    default: open\n    why: baseline\n    allow:\n      role:\n        values: [user]\n        why: everyone gets the commons\n",
        &[],
        &["commons"],
    )
    .expect("projects");
    let rule = &set.rules[&key("marketplace", "commons", "role", "user")];
    assert_eq!(rule.justification, "everyone gets the commons");
    assert!(set.entities[&("marketplace".to_owned(), "commons".to_owned())].default_included);
}

#[test]
fn the_gateway_glob_expands_over_the_catalog_only() {
    let set = project(
        "entities:\n  - entity: gateway_route/*\n    default: open\n    why: routes\n    allow:\n      role: [user]\n",
        &["claude-star-4203d1", "gemini-star-5db635"],
        &[],
    )
    .expect("projects");
    assert_eq!(set.rules.len(), 2);
    assert!(set.governs("gateway_route", "claude-star-4203d1"));
    assert!(set.governs("gateway_route", "gemini-star-5db635"));
    assert!(!set.governs("gateway_route", "openai-o4-mini"));
}

#[test]
fn a_glob_over_an_empty_catalog_is_an_error_not_an_empty_declaration() {
    let err = project(
        "entities:\n  - entity: gateway_route/*\n    default: open\n    why: routes\n    allow:\n      role: [user]\n",
        &[],
        &[],
    )
    .expect_err("an empty expansion declares nothing and must not read as clean");
    assert!(err.contains("gateway_route/*"), "{err}");
    assert!(err.contains("no entity of that kind"), "{err}");
}

#[test]
fn a_literal_gateway_route_id_is_rejected() {
    let err = parse_rules_doc(
        "entities:\n  - entity: gateway_route/claude-star-4203d1\n    why: x\n    allow:\n      role: [user]\n",
    )
    .expect_err("rejected")
    .to_string();
    assert!(err.contains("generated"), "{err}");
}

#[test]
fn why_is_required_and_globs_are_for_generated_kinds_only() {
    let missing = parse_rules_doc(
        "entities:\n  - entity: mcp_server/x\n    why: ' '\n    allow:\n      role: [user]\n",
    )
    .expect_err("rejected")
    .to_string();
    assert!(missing.contains("`why` is required"), "{missing}");
    let glob = parse_rules_doc(
        "entities:\n  - entity: mcp_server/*\n    why: x\n    allow:\n      role: [user]\n",
    )
    .expect_err("rejected")
    .to_string();
    assert!(glob.contains("only gateway_route and hook"), "{glob}");
}

#[test]
fn a_subject_cannot_be_both_allowed_and_denied_on_one_band() {
    let err = parse_rules_doc(
        "entities:\n  - entity: mcp_server/x\n    why: x\n    allow:\n      group: [uk]\n    deny:\n      group: [uk]\n",
    )
    .expect_err("rejected")
    .to_string();
    assert!(err.contains("both allowed and denied"), "{err}");
}

#[test]
fn an_undefined_marketplace_is_rejected_at_projection() {
    let err = project(
        "entities:\n  - entity: marketplace/ghost\n    why: x\n    allow:\n      role: [user]\n",
        &[],
        &["commons"],
    )
    .expect_err("rejected");
    assert!(err.contains("marketplace/ghost"), "{err}");
}

#[test]
fn unknown_keys_and_bands_are_rejected() {
    assert!(parse_rules_doc("entities:\n  - entity: mcp_server/x\n    why: x\n    reason: y\n    allow:\n      role: [user]\n").is_err());
    assert!(parse_rules_doc("entities:\n  - entity: mcp_server/x\n    why: x\n    allow:\n      department: [sales]\n").is_err());
    assert!(matches!(
        "skill/x".parse::<systemprompt_web_admin::repositories::config::rules_yaml_types::EntityRef>(),
        Ok(r) if r.kind == EntityKind::Skill && r.id == "x"
    ));
}

#[test]
fn an_owned_marketplace_waits_for_its_bundle_instead_of_failing() {
    let set = project(
        "entities:\n  - entity: marketplace/europe\n    owner: bundle:sfnext\n    why: kit\n    allow:\n      group: [europe-devs]\n",
        &[],
        &["commons"],
    )
    .expect("an absent id with an owner is not an error");
    assert_eq!(
        set.rule_count(),
        0,
        "nothing projected until the bundle is active"
    );
    assert_eq!(set.awaiting.len(), 1);
    assert_eq!(set.awaiting[0].entity_id, "europe");
    assert_eq!(set.awaiting[0].owner, "bundle:sfnext");
    assert_eq!(
        set.owners.get("marketplace/europe").map(String::as_str),
        Some("bundle:sfnext")
    );

    let active = project(
        "entities:\n  - entity: marketplace/europe\n    owner: bundle:sfnext\n    why: kit\n    allow:\n      group: [europe-devs]\n",
        &[],
        &["commons", "europe"],
    )
    .expect("projects once the id exists");
    assert_eq!(
        active.rule_count(),
        1,
        "governed like any other once active"
    );
    assert!(active.awaiting.is_empty());
}

#[test]
fn a_malformed_owner_is_rejected_at_parse() {
    let err = parse_rules_doc(
        "entities:\n  - entity: marketplace/europe\n    owner: sfnext\n    why: kit\n    allow:\n      group: [europe-devs]\n",
    )
    .expect_err("rejected")
    .to_string();
    assert!(err.contains("bundle:<name>"), "{err}");
}

fn rejection(yaml: &str) -> String {
    parse_rules_doc(yaml).expect_err("rejected").to_string()
}

#[test]
fn validate_with_literal_generated_id_returns_the_glob_hint() {
    let err = rejection(
        "entities:\n  - entity: gateway_route/abc\n    why: x\n    allow:\n      role: [user]\n",
    );
    assert!(
        err.contains("gateway_route/abc: gateway_route ids are generated, never written — use gateway_route/*"),
        "{err}"
    );
}

#[test]
fn validate_with_neither_allow_nor_deny_returns_declares_nothing() {
    let err = rejection("entities:\n  - entity: mcp_server/x\n    why: x\n");
    assert!(
        err.contains("mcp_server/x: declares no allow and no deny"),
        "{err}"
    );
}

#[test]
fn validate_with_owner_on_a_glob_returns_glob_owner_error() {
    let err = rejection(
        "entities:\n  - entity: gateway_route/*\n    owner: bundle:kit\n    why: x\n    allow:\n      role: [user]\n",
    );
    assert!(
        err.contains("gateway_route/*: a glob cannot name an owner"),
        "{err}"
    );
}

#[test]
fn validate_with_conflict_returns_band_and_subject() {
    let err = rejection(
        "entities:\n  - entity: mcp_server/x\n    why: x\n    allow:\n      role: [admin, user]\n    deny:\n      role: [user]\n",
    );
    assert!(
        err.contains("mcp_server/x: role 'user' is both allowed and denied"),
        "{err}"
    );
}

#[test]
fn validate_with_empty_band_returns_names_no_subjects() {
    let err = rejection(
        "entities:\n  - entity: mcp_server/x\n    why: x\n    allow:\n      role: [admin]\n    deny:\n      group: []\n",
    );
    assert!(
        err.contains("mcp_server/x: group names no subjects"),
        "{err}"
    );
}

#[test]
fn validate_with_blank_subject_returns_blank_subject_error() {
    let err = rejection(
        "entities:\n  - entity: mcp_server/x\n    why: x\n    allow:\n      project: [' ']\n",
    );
    assert!(
        err.contains("mcp_server/x: project contains a blank subject"),
        "{err}"
    );
}

#[test]
fn the_declared_hash_follows_meaning_not_formatting() {
    let a = project(
        "entities:\n  - entity: mcp_server/x\n    why: pilot\n    allow:\n      role: [admin]\n      group: [uk]\n",
        &[],
        &[],
    )
    .expect("a");
    let b = project(
        "entities:\n\n  - entity: mcp_server/x\n    why: \"pilot\"\n    allow:\n      group: [uk]\n      role: [admin]\n",
        &[],
        &[],
    )
    .expect("b");
    let c = project(
        "entities:\n  - entity: mcp_server/x\n    why: pilot for now\n    allow:\n      role: [admin]\n      group: [uk]\n",
        &[],
        &[],
    )
    .expect("c");
    assert_eq!(
        a.declared_hash(),
        b.declared_hash(),
        "whitespace and band order do not count"
    );
    assert_ne!(
        a.declared_hash(),
        c.declared_hash(),
        "a changed reason does"
    );
    assert_eq!(a.declared_hash().len(), 64);
}
