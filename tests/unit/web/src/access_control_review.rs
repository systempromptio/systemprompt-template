//! The access review: drift regrouped by entity, one kind per entity, and a
//! "keep the database" decision that holds only for the diff it was taken on.

use chrono::Utc;
use systemprompt_security::authz::{Access, DASHBOARD_SOURCE, RegisteredEntities};
use systemprompt_web_admin::repositories::access_control::declared::{
    DeclaredInputs, DeclaredSet, build_declared_set,
};
use systemprompt_web_admin::repositories::access_control::drift::{
    BandRuleRow, EntityDefaultRow, compute_drift,
};
use systemprompt_web_admin::repositories::access_control::review::{
    EntityReview, ReviewKind, review_entities,
};
use systemprompt_web_admin::repositories::config::rules_yaml_loader::parse_rules_doc;
use systemprompt_web_admin::repositories::sync::attention::split_reviews;
use systemprompt_web_admin::repositories::sync::history::KeptReview;

fn declared() -> DeclaredSet {
    let doc = parse_rules_doc(
        "entities:\n  - entity: mcp_server/atlassian\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n      group: [india-devs]\n  - entity: mcp_server/jira\n    default: closed\n    why: pilot\n    allow:\n      role: [admin]\n",
    )
    .expect("parses");
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

fn row(entity_id: &str, rule_type: &str, value: &str, access: Access, source: &str) -> BandRuleRow {
    BandRuleRow {
        id: format!("{entity_id}-{rule_type}-{value}"),
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        rule_type: rule_type.to_owned(),
        rule_value: value.to_owned(),
        access,
        justification: Some("pilot".to_owned()),
        source: source.to_owned(),
        valid_until: None,
    }
}

fn entity(entity_id: &str) -> EntityDefaultRow {
    EntityDefaultRow {
        entity_type: "mcp_server".to_owned(),
        entity_id: entity_id.to_owned(),
        default_included: false,
        source: "yaml".to_owned(),
    }
}

fn find<'a>(reviews: &'a [EntityReview], id: &str) -> &'a EntityReview {
    reviews
        .iter()
        .find(|r| r.entity_id == id)
        .unwrap_or_else(|| panic!("no review for {id}: {reviews:?}"))
}

#[test]
fn an_entity_never_applied_is_new_in_code_and_not_live() {
    let reviews = review_entities(&compute_drift(&declared(), &[], &[]));
    let jira = find(&reviews, "jira");
    assert_eq!(jira.kind, ReviewKind::NewInCode);
    assert!(jira.not_live);
    assert!(jira.applies);
    assert_eq!(jira.key, "mcp_server/jira");
}

#[test]
fn a_live_entity_missing_a_declared_rule_reads_as_removed_in_console() {
    let rules = [row("atlassian", "role", "admin", Access::Allow, "yaml")];
    let entities = [entity("atlassian"), entity("jira")];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    let atlassian = find(&reviews, "atlassian");
    assert_eq!(atlassian.kind, ReviewKind::RemovedInConsole);
    assert!(!atlassian.not_live);
    assert_eq!(atlassian.lines.len(), 1, "{atlassian:?}");
    assert_eq!(atlassian.lines[0].subject, "india-devs");
}

#[test]
fn a_disagreeing_row_is_changed_in_code_and_a_console_extra_is_only_in_console() {
    let rules = [
        row("atlassian", "role", "admin", Access::Deny, "yaml"),
        row("atlassian", "group", "india-devs", Access::Allow, "yaml"),
        row("jira", "role", "admin", Access::Allow, "yaml"),
        row("github", "group", "uk", Access::Allow, DASHBOARD_SOURCE),
    ];
    let entities = [entity("atlassian"), entity("jira"), entity("github")];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    assert_eq!(find(&reviews, "atlassian").kind, ReviewKind::ChangedInCode);
    let github = find(&reviews, "github");
    assert_eq!(github.kind, ReviewKind::OnlyInConsole);
    assert!(
        !github.applies,
        "code does not govern github, so applying it moves nothing"
    );
    assert!(reviews.iter().all(|r| r.entity_id != "jira"), "jira agrees");
}

#[test]
fn a_bundle_row_on_an_ungoverned_entity_is_never_reviewed() {
    let rules = [
        row("atlassian", "role", "admin", Access::Allow, "yaml"),
        row("atlassian", "group", "india-devs", Access::Allow, "yaml"),
        row("jira", "role", "admin", Access::Allow, "yaml"),
        row("kitsrv", "group", "uk", Access::Allow, "bundle:sfnext"),
    ];
    let entities = [entity("atlassian"), entity("jira")];
    let reviews = review_entities(&compute_drift(&declared(), &rules, &entities));
    assert!(reviews.is_empty(), "{reviews:?}");
}

#[test]
fn a_kept_decision_holds_only_for_the_diff_it_was_taken_on() {
    let reviews = review_entities(&compute_drift(&declared(), &[], &[]));
    let jira = find(&reviews, "jira").clone();
    let kept = |fingerprint: &str| KeptReview {
        key: jira.key.clone(),
        fingerprint: fingerprint.to_owned(),
        reason: "retiring it".to_owned(),
        display_name: "Ed".to_owned(),
        created_at: Utc::now(),
    };

    let held = split_reviews(reviews.clone(), &[kept(&jira.fingerprint)]);
    assert!(held.pending.iter().all(|r| r.review.key != jira.key));
    assert_eq!(held.kept.len(), 1);

    let moved = split_reviews(reviews, &[kept("an-older-diff")]);
    assert!(moved.pending.iter().any(|r| r.review.key == jira.key));
    assert!(moved.kept.is_empty());
}

#[test]
fn the_fingerprint_is_stable_and_moves_with_the_diff() {
    let a = review_entities(&compute_drift(&declared(), &[], &[]));
    let b = review_entities(&compute_drift(&declared(), &[], &[]));
    assert_eq!(find(&a, "jira").fingerprint, find(&b, "jira").fingerprint);
    let rules = [row("jira", "group", "uk", Access::Allow, DASHBOARD_SOURCE)];
    let c = review_entities(&compute_drift(&declared(), &rules, &[]));
    assert_ne!(find(&a, "jira").fingerprint, find(&c, "jira").fingerprint);
}
