//! REQ-044 "User Awareness" — the negative access-control matrix the
//! requirements register asks for.
//!
//! Two principals whose session contexts differ (role, department,
//! organization, Salesforce-linked state) must resolve *different* authorized
//! sets over the entity classes the requirement names — MCP servers, models
//! (`gateway_route`), and knowledge sources (skills/plugins) — and a denied
//! entity must resolve deny through the same `resolve_user_matrix` path the
//! enforcement webhook uses, with the deciding band reported.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::users::access_control::{
    MatrixRow, SectionInput, resolve_user_matrix,
};

use crate::fixtures::{
    OrgSpec, insert_acl_rule, insert_member, insert_org, insert_user, insert_user_full,
    set_department, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

fn one_entity(entity_type: &str, entity_id: &str) -> Vec<SectionInput> {
    vec![(
        entity_type.to_owned(),
        entity_type.to_owned(),
        vec![(entity_id.to_owned(), entity_id.to_owned(), None)],
    )]
}

async fn grade_entity(
    pool: &PgPool,
    user: &UserId,
    entity_type: &str,
    entity_id: &str,
) -> MatrixRow {
    let matrix = resolve_user_matrix(pool, user, one_entity(entity_type, entity_id))
        .await
        .expect("resolve matrix")
        .expect("user found");
    let mut sections = matrix.sections;
    let section = sections.pop().expect("one section");
    section.rows.into_iter().next().expect("one row")
}

async fn link_salesforce(pool: &PgPool, user: &UserId) {
    sqlx::query(
        "INSERT INTO salesforce_user_identities (user_id, sf_username) VALUES ($1, $2)",
    )
    .bind(user.as_str())
    .bind(format!("{}@example.com.sandbox", user.as_str()))
    .execute(pool)
    .await
    .expect("link salesforce identity");
}

#[tokio::test]
async fn role_context_yields_different_mcp_server_sets() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let admin = insert_user_full(
        &db.pool,
        &unique("admin"),
        &unclaimed_email("req044a"),
        None,
        &["admin".to_owned()],
        "active",
    )
    .await;
    let plain = insert_user(&db.pool, &unique("user"), &unclaimed_email("req044b")).await;
    let server = unique("mcp");
    insert_acl_rule(&db.pool, "mcp_server", &server, "role", "admin", "allow").await;

    let admin_row = grade_entity(&db.pool, &admin, "mcp_server", &server).await;
    let plain_row = grade_entity(&db.pool, &plain, "mcp_server", &server).await;

    assert_eq!(admin_row.effective, "allow");
    assert_eq!(admin_row.source.layer, "role");
    assert_eq!(plain_row.effective, "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn gateway_route_is_denied_outside_the_granted_organization() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let insider = insert_user(&db.pool, &unique("in"), &unclaimed_email("req044c")).await;
    let outsider = insert_user(&db.pool, &unique("out"), &unclaimed_email("req044d")).await;
    let org_id = unique("org");
    let slug = unique("slug");
    insert_org(&db.pool, &OrgSpec::active(&org_id, &slug)).await;
    insert_member(&db.pool, &insider, &org_id, "member").await;
    let route = unique("model");
    insert_acl_rule(&db.pool, "gateway_route", &route, "organization", &slug, "allow").await;

    let insider_row = grade_entity(&db.pool, &insider, "gateway_route", &route).await;
    let outsider_row = grade_entity(&db.pool, &outsider, "gateway_route", &route).await;

    assert_eq!(insider_row.effective, "allow");
    assert_eq!(insider_row.source.layer, "organization");
    assert_eq!(outsider_row.effective, "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn narrower_band_deny_overrides_a_broader_allow() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("req044e")).await;
    let org_id = unique("org");
    let slug = unique("slug");
    insert_org(&db.pool, &OrgSpec::active(&org_id, &slug)).await;
    insert_member(&db.pool, &user, &org_id, "member").await;
    set_department(&db.pool, &user, "Restricted").await;
    let skill = unique("skill");
    insert_acl_rule(&db.pool, "skill", &skill, "organization", &slug, "allow").await;
    insert_acl_rule(&db.pool, "skill", &skill, "department", "Restricted", "deny").await;

    let row = grade_entity(&db.pool, &user, "skill", &skill).await;

    assert_eq!(row.effective, "deny");
    assert_eq!(row.source.layer, "department");
    db.cleanup().await;
}

#[tokio::test]
async fn salesforce_linked_state_gates_the_entity() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let linked = insert_user(&db.pool, &unique("linked"), &unclaimed_email("req044f")).await;
    let unlinked = insert_user(&db.pool, &unique("unlinked"), &unclaimed_email("req044g")).await;
    link_salesforce(&db.pool, &linked).await;
    let server = unique("sfmcp");
    insert_acl_rule(&db.pool, "mcp_server", &server, "salesforce", "linked", "allow").await;

    let linked_row = grade_entity(&db.pool, &linked, "mcp_server", &server).await;
    let unlinked_row = grade_entity(&db.pool, &unlinked, "mcp_server", &server).await;

    assert_eq!(linked_row.effective, "allow");
    assert_eq!(linked_row.source.layer, "salesforce");
    assert_eq!(unlinked_row.effective, "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn an_entity_with_rules_defaults_to_deny_for_unmatched_users() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("req044h")).await;
    let plugin = unique("plugin");
    insert_acl_rule(&db.pool, "plugin", &plugin, "role", "admin", "allow").await;

    let row = grade_entity(&db.pool, &user, "plugin", &plugin).await;

    assert_eq!(row.effective, "deny");
    db.cleanup().await;
}
