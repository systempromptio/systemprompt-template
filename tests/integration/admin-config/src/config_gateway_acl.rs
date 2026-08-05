//! `repositories::config::gateway_acl` — the route-scoped wrappers over core's
//! access-control repository.

use systemprompt_security::authz::{Access, EntityKind, RuleType};
use systemprompt_web_admin::repositories::config::gateway_acl;

use crate::fixtures::{count_rows, insert_acl_entity, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_rules_for_route_returns_nothing_for_an_unknown_route() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let rules = gateway_acl::list_rules_for_route(&db.pool, &unique("route"))
        .await
        .expect("list rules");

    assert!(rules.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_rule_then_list_rules_for_route_round_trips() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let route = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &route, false).await;

    let created =
        gateway_acl::upsert_rule(&db.pool, &route, RuleType::ROLE, "admin", Access::Allow)
            .await
            .expect("upsert rule");
    assert_eq!(created.rule_value, "admin");
    assert_eq!(created.access, Access::Allow);

    let rules = gateway_acl::list_rules_for_route(&db.pool, &route)
        .await
        .expect("list rules");
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].id, created.id);

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_rule_updates_access_in_place_rather_than_duplicating() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let route = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &route, false).await;

    let first = gateway_acl::upsert_rule(&db.pool, &route, RuleType::ROLE, "admin", Access::Allow)
        .await
        .expect("first upsert");
    let second = gateway_acl::upsert_rule(&db.pool, &route, RuleType::ROLE, "admin", Access::Deny)
        .await
        .expect("second upsert");

    assert_eq!(first.id, second.id, "the unique key is entity + subject");
    assert_eq!(second.access, Access::Deny);
    let rules = gateway_acl::list_rules_for_route(&db.pool, &route)
        .await
        .expect("list rules");
    assert_eq!(rules.len(), 1);

    db.cleanup().await;
}

#[tokio::test]
async fn upsert_rule_fails_without_a_catalog_entity() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = gateway_acl::upsert_rule(
        &db.pool,
        &unique("route"),
        RuleType::ROLE,
        "admin",
        Access::Allow,
    )
    .await;

    assert!(
        result.is_err(),
        "a grant with no catalog row would be invisible to the resolver"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_rules_bulk_groups_rules_by_route() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let route_a = unique("route-a");
    let route_b = unique("route-b");
    let route_absent = unique("route-c");
    for route in [&route_a, &route_b] {
        insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), route, false).await;
    }
    gateway_acl::upsert_rule(&db.pool, &route_a, RuleType::ROLE, "admin", Access::Allow)
        .await
        .expect("rule on a");
    gateway_acl::upsert_rule(&db.pool, &route_a, RuleType::ROLE, "user", Access::Deny)
        .await
        .expect("second rule on a");
    gateway_acl::upsert_rule(&db.pool, &route_b, RuleType::ROLE, "admin", Access::Allow)
        .await
        .expect("rule on b");

    let ids = vec![route_a.clone(), route_b.clone(), route_absent.clone()];
    let map = gateway_acl::list_rules_bulk(&db.pool, &ids)
        .await
        .expect("bulk list");

    assert_eq!(map.get(&route_a).map(Vec::len), Some(2));
    assert_eq!(map.get(&route_b).map(Vec::len), Some(1));
    assert_eq!(
        map.get(&route_absent).map(Vec::len),
        Some(0),
        "every requested route is present, so the caller can index without a fallback"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn delete_rule_reports_whether_a_row_was_removed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let route = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &route, false).await;
    let rule = gateway_acl::upsert_rule(&db.pool, &route, RuleType::ROLE, "admin", Access::Allow)
        .await
        .expect("upsert rule");

    let removed = gateway_acl::delete_rule(&db.pool, rule.id.as_str())
        .await
        .expect("delete rule");
    let again = gateway_acl::delete_rule(&db.pool, rule.id.as_str())
        .await
        .expect("delete again");

    assert!(removed);
    assert!(!again, "a second delete removed nothing");
    assert_eq!(
        count_rows(
            &db.pool,
            "SELECT COUNT(*) FROM access_control_rules WHERE entity_id = $1",
            &route,
        )
        .await,
        0
    );

    db.cleanup().await;
}

#[tokio::test]
async fn find_entity_returns_none_for_an_unregistered_route() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let entity = gateway_acl::find_entity(&db.pool, &unique("route"))
        .await
        .expect("find entity");

    assert!(entity.is_none(), "find_ reports absence as None");

    db.cleanup().await;
}

#[tokio::test]
async fn find_entity_reads_back_default_included() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let route = unique("route");
    insert_acl_entity(&db.pool, EntityKind::GatewayRoute.as_str(), &route, true).await;

    let entity = gateway_acl::find_entity(&db.pool, &route)
        .await
        .expect("find entity")
        .expect("registered entity");

    assert_eq!(entity.id, route);
    assert_eq!(entity.kind, EntityKind::GatewayRoute);
    assert!(entity.default_included);

    db.cleanup().await;
}
