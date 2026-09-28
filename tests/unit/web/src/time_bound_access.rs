//! Time-bound access, the pure halves: the `valid_until` that rides through
//! `rules.yaml` (projection, expiry, drift, export), and the manage-role test
//! the expiry sweep revokes on.

use chrono::{DateTime, Utc};
use systemprompt_security::authz::{Access, RegisteredEntities};
use systemprompt_web_admin::repositories::access_control::declared::{
    DeclaredInputs, DeclaredKey, DeclaredSet, build_declared_set,
};
use systemprompt_web_admin::repositories::access_control::drift::{
    BandRuleRow, EntityDefaultRow, compute_drift,
};
use systemprompt_web_admin::repositories::access_control::export::{Owners, render_export};
use systemprompt_web_admin::repositories::config::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_extension::jobs::lost_manage_role;

fn at(rfc3339: &str) -> DateTime<Utc> {
    DateTime::parse_from_rfc3339(rfc3339)
        .expect("rfc3339")
        .with_timezone(&Utc)
}

fn project(yaml: &str) -> DeclaredSet {
    let doc = parse_rules_doc(yaml).expect("parses");
    build_declared_set(
        &doc,
        &DeclaredInputs {
            gateway_routes: &[],
            marketplace_ids: &[],
            registered: &RegisteredEntities::default(),
        },
    )
    .expect("projects")
}

fn key(rule_type: &str, value: &str) -> DeclaredKey {
    DeclaredKey {
        entity_type: "mcp_server".to_owned(),
        entity_id: "atlassian".to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: value.to_owned(),
    }
}

const WINDOWED: &str = "entities:\n  - entity: mcp_server/atlassian\n    why: pilot\n    valid_until: 2026-12-31T00:00:00Z\n    allow:\n      group: [uk]\n";

#[test]
fn valid_until_lands_on_every_rule_of_the_entity_and_changes_the_hash() {
    let windowed = project(WINDOWED);
    let rule = &windowed.rules[&key("group", "uk")];
    assert_eq!(rule.valid_until, Some(at("2026-12-31T00:00:00Z")));

    let open = project(
        "entities:\n  - entity: mcp_server/atlassian\n    why: pilot\n    allow:\n      group: [uk]\n",
    );
    assert_eq!(open.rules[&key("group", "uk")].valid_until, None);
    assert_ne!(open.declared_hash(), windowed.declared_hash());
}

#[test]
fn a_declaration_past_its_window_is_dropped_but_the_entity_stays_governed() {
    let mut set = project(WINDOWED);
    set.drop_expired(at("2026-12-30T00:00:00Z"));
    assert_eq!(set.rule_count(), 1, "still inside the window");
    set.drop_expired(at("2027-01-01T00:00:00Z"));
    assert_eq!(set.rule_count(), 0);
    assert!(
        set.governs("mcp_server", "atlassian"),
        "the sweep's deletion of the expired row must not read as dashboard drift"
    );
}

fn db_row(valid_until: Option<DateTime<Utc>>) -> BandRuleRow {
    BandRuleRow {
        id: "r1".to_owned(),
        entity_type: "mcp_server".to_owned(),
        entity_id: "atlassian".to_owned(),
        rule_type: "group".to_owned(),
        rule_value: "uk".to_owned(),
        access: Access::Allow,
        justification: Some("pilot".to_owned()),
        source: "yaml".to_owned(),
        valid_until,
    }
}

#[test]
fn a_window_that_differs_between_code_and_database_is_drift() {
    let declared = project(WINDOWED);
    let entities = [EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: "atlassian".to_owned(),
        default_included: false,
        source: "yaml".to_owned(),
    }];
    let same = compute_drift(
        &declared,
        &[db_row(Some(at("2026-12-31T00:00:00Z")))],
        &entities,
    );
    assert!(same.is_clean());

    let differs = compute_drift(&declared, &[db_row(None)], &entities);
    assert_eq!(differs.changed.len(), 1);
    assert_eq!(
        differs.changed[0].declared_valid_until,
        Some(at("2026-12-31T00:00:00Z"))
    );
    assert_eq!(differs.changed[0].db_valid_until, None);
}

#[test]
fn the_export_writes_the_window_back_and_the_document_round_trips() {
    let rows = [db_row(Some(at("2026-12-31T00:00:00Z")))];
    let entities = [EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: "atlassian".to_owned(),
        default_included: false,
        source: "yaml".to_owned(),
    }];
    let exported = render_export(&rows, &entities, &Owners::default());
    assert!(exported.contains("valid_until:"), "{exported}");
    let reparsed = project(&exported);
    assert_eq!(
        reparsed.rules[&key("group", "uk")].valid_until,
        Some(at("2026-12-31T00:00:00Z"))
    );
}

#[test]
fn rows_that_disagree_on_the_window_export_open_ended_with_a_note() {
    let mut second = db_row(None);
    second.id = "r2".to_owned();
    second.rule_value = "india-devs".to_owned();
    let rows = [db_row(Some(at("2026-12-31T00:00:00Z"))), second];
    let exported = render_export(&rows, &[], &Owners::default());
    assert!(!exported.contains("valid_until:"), "{exported}");
    assert!(
        exported.contains("different valid_until windows"),
        "{exported}"
    );
}

#[test]
fn losing_a_manage_role_is_what_triggers_credential_revocation() {
    let admin = ["admin".to_owned(), "user".to_owned()];
    let user = ["user".to_owned()];
    assert!(lost_manage_role(&admin, &user));
    assert!(!lost_manage_role(&user, &user), "nothing to lose");
    assert!(!lost_manage_role(&admin, &admin), "still held");
    let platform = ["platform_admin".to_owned()];
    assert!(
        !lost_manage_role(&admin, &platform),
        "another manage role remains"
    );
}
