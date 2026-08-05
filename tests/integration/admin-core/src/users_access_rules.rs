//! `repositories::users::access_control::rules` — CRUD over
//! `access_control_rules`.
//!
//! `set_entity_rules` and `bulk_set_rules` are replace-in-place: they create
//! the catalog row the foreign key needs, delete every existing grant on the
//! entity, then insert the supplied set, all in one transaction. The tests
//! scope themselves to entity ids they minted, because a deployment may carry
//! grants written by the governance bootstrap.

use systemprompt_web_admin::repositories::users::access_control::{
    bulk_set_rules, list_all_rules, list_rules_for_entity, set_entity_rules,
};
use systemprompt_web_admin::types::access_control::{
    AccessControlRuleInput, AccessDecision, RuleType,
};

use crate::fixtures::{AclRuleSpec, insert_acl_rule, unique};
use crate::tempdb::TempDb;

fn allow_user(user_id: &str) -> AccessControlRuleInput {
    AccessControlRuleInput {
        rule_type: RuleType::USER,
        rule_value: user_id.to_owned(),
        access: AccessDecision::Allow,
    }
}

fn deny_role(role: &str) -> AccessControlRuleInput {
    AccessControlRuleInput {
        rule_type: RuleType::ROLE,
        rule_value: role.to_owned(),
        access: AccessDecision::Deny,
    }
}

#[tokio::test]
async fn list_rules_for_entity_is_empty_for_an_entity_with_no_grants() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let rules = list_rules_for_entity(&db.pool, "skill", &unique("skill"))
        .await
        .expect("list rules");

    assert!(rules.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_rules_for_entity_returns_only_that_entitys_grants() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let wanted = unique("skill");
    let other = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &wanted, "user", "alice"),
    )
    .await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &other, "user", "bob"),
    )
    .await;

    let rules = list_rules_for_entity(&db.pool, "skill", &wanted)
        .await
        .expect("list rules");

    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_value, "alice");
    assert_eq!(rules[0].access, AccessDecision::Allow);
    db.cleanup().await;
}

#[tokio::test]
async fn list_rules_for_entity_orders_by_rule_type_then_value() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &entity, "user", "zoe"),
    )
    .await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &entity, "role", "admin"),
    )
    .await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &entity, "user", "adam"),
    )
    .await;

    let rules = list_rules_for_entity(&db.pool, "skill", &entity)
        .await
        .expect("list rules");

    let values: Vec<&str> = rules.iter().map(|r| r.rule_value.as_str()).collect();
    assert_eq!(values, ["admin", "adam", "zoe"]);
    db.cleanup().await;
}

#[tokio::test]
async fn list_all_rules_includes_every_entity_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let skill = unique("skill");
    let server = unique("server");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &skill, "user", "alice"),
    )
    .await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::deny("mcp_server", &server, "role", "admin"),
    )
    .await;

    let rules = list_all_rules(&db.pool).await.expect("list all rules");

    assert!(rules.iter().any(|r| r.entity_id == skill));
    assert!(
        rules
            .iter()
            .any(|r| r.entity_id == server && r.access == AccessDecision::Deny)
    );
    db.cleanup().await;
}

#[tokio::test]
async fn set_entity_rules_creates_the_catalog_row_the_foreign_key_needs() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");

    let written = set_entity_rules(&db.pool, "skill", &entity, &[allow_user("alice")])
        .await
        .expect("set rules");

    assert_eq!(written.len(), 1);
    assert_eq!(written[0].rule_value, "alice");
    let catalog: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_control_entities WHERE entity_type = 'skill' AND entity_id = $1",
    )
    .bind(&entity)
    .fetch_one(&*db.pool)
    .await
    .expect("count catalog rows");
    assert_eq!(catalog, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn set_entity_rules_replaces_the_previous_grants_rather_than_adding_to_them() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");
    set_entity_rules(
        &db.pool,
        "skill",
        &entity,
        &[allow_user("alice"), allow_user("bob")],
    )
    .await
    .expect("set initial rules");

    set_entity_rules(&db.pool, "skill", &entity, &[deny_role("contractor")])
        .await
        .expect("replace rules");

    let rules = list_rules_for_entity(&db.pool, "skill", &entity)
        .await
        .expect("list rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_type, RuleType::ROLE);
    assert_eq!(rules[0].rule_value, "contractor");
    assert_eq!(rules[0].access, AccessDecision::Deny);
    db.cleanup().await;
}

#[tokio::test]
async fn set_entity_rules_with_an_empty_set_clears_every_grant() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");
    set_entity_rules(&db.pool, "skill", &entity, &[allow_user("alice")])
        .await
        .expect("set initial rules");

    let written = set_entity_rules(&db.pool, "skill", &entity, &[])
        .await
        .expect("clear rules");

    assert!(written.is_empty());
    assert!(
        list_rules_for_entity(&db.pool, "skill", &entity)
            .await
            .expect("list rules")
            .is_empty()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn set_entity_rules_accepts_an_extension_rule_type() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");
    let input = AccessControlRuleInput {
        rule_type: RuleType::from("department"),
        rule_value: "Platform".to_owned(),
        access: AccessDecision::Allow,
    };

    set_entity_rules(&db.pool, "skill", &entity, &[input])
        .await
        .expect("set department rule");

    let rules = list_rules_for_entity(&db.pool, "skill", &entity)
        .await
        .expect("list rules");
    assert_eq!(rules[0].rule_type.as_str(), "department");
    db.cleanup().await;
}

#[tokio::test]
async fn bulk_set_rules_applies_the_same_grants_to_every_entity() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let first = unique("skill");
    let second = unique("skill");
    let entities = vec![
        ("skill".to_owned(), first.clone()),
        ("skill".to_owned(), second.clone()),
    ];

    let count = bulk_set_rules(
        &db.pool,
        &entities,
        &[allow_user("alice"), deny_role("temp")],
    )
    .await
    .expect("bulk set");

    assert_eq!(count, 2, "the return value counts entities, not rules");
    for entity in [&first, &second] {
        let rules = list_rules_for_entity(&db.pool, "skill", entity)
            .await
            .expect("list rules");
        assert_eq!(rules.len(), 2);
    }
    db.cleanup().await;
}

#[tokio::test]
async fn bulk_set_rules_replaces_grants_that_were_already_there() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let entity = unique("skill");
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &entity, "user", "stale"),
    )
    .await;
    let entities = vec![("skill".to_owned(), entity.clone())];

    bulk_set_rules(&db.pool, &entities, &[allow_user("fresh")])
        .await
        .expect("bulk set");

    let rules = list_rules_for_entity(&db.pool, "skill", &entity)
        .await
        .expect("list rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].rule_value, "fresh");
    db.cleanup().await;
}

#[tokio::test]
async fn bulk_set_rules_over_no_entities_writes_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let count = bulk_set_rules(&db.pool, &[], &[allow_user("alice")])
        .await
        .expect("bulk set");

    assert_eq!(count, 0);
    db.cleanup().await;
}
