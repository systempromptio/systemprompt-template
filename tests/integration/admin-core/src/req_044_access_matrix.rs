//! REQ-044 "User Awareness" — the negative access-control matrix the
//! requirements register asks for.
//!
//! Two principals whose session contexts differ (role, group membership)
//! must resolve *different* authorized
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
    insert_acl_rule, insert_group, insert_group_member, insert_user, insert_user_full,
    unclaimed_email, unique,
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

// Why: a directory-sourced membership, which is what a sign-in writes. The
// gate must not care which of the two sources placed the member.
async fn join_group(pool: &PgPool, user: &UserId, group: &str) {
    insert_group(pool, group, "Fixture group").await;
    insert_group_member(pool, group, user, "adfs").await;
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
async fn gateway_route_is_denied_outside_the_granted_group() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let insider = insert_user(&db.pool, &unique("in"), &unclaimed_email("req044c")).await;
    let outsider = insert_user(&db.pool, &unique("out"), &unclaimed_email("req044d")).await;
    let commerce = unique("grp");
    let core = unique("grp");
    join_group(&db.pool, &insider, &commerce).await;
    join_group(&db.pool, &outsider, &core).await;
    let route = unique("model");
    insert_acl_rule(
        &db.pool,
        "gateway_route",
        &route,
        "group",
        &commerce,
        "allow",
    )
    .await;

    let insider_row = grade_entity(&db.pool, &insider, "gateway_route", &route).await;
    let outsider_row = grade_entity(&db.pool, &outsider, "gateway_route", &route).await;

    assert_eq!(insider_row.effective, "allow");
    assert_eq!(insider_row.source.layer, "group");
    assert_eq!(outsider_row.effective, "deny");
    db.cleanup().await;
}

#[tokio::test]
async fn narrower_band_deny_overrides_a_broader_allow() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("req044e")).await;
    let core = unique("grp");
    join_group(&db.pool, &user, &core).await;
    let skill = unique("skill");
    insert_acl_rule(&db.pool, "skill", &skill, "role", "user", "allow").await;
    insert_acl_rule(&db.pool, "skill", &skill, "group", &core, "deny").await;

    let row = grade_entity(&db.pool, &user, "skill", &skill).await;

    assert_eq!(row.effective, "deny");
    assert_eq!(row.source.layer, "group");
    db.cleanup().await;
}

#[tokio::test]
async fn group_membership_gates_the_entity() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let member = insert_user(&db.pool, &unique("member"), &unclaimed_email("req044f")).await;
    let outsider = insert_user(&db.pool, &unique("outsider"), &unclaimed_email("req044g")).await;
    let engineering = unique("grp");
    join_group(&db.pool, &member, &engineering).await;
    let server = unique("groupmcp");
    insert_acl_rule(
        &db.pool,
        "mcp_server",
        &server,
        "group",
        &engineering,
        "allow",
    )
    .await;

    let member_row = grade_entity(&db.pool, &member, "mcp_server", &server).await;
    let outsider_row = grade_entity(&db.pool, &outsider, "mcp_server", &server).await;

    assert_eq!(member_row.effective, "allow");
    assert_eq!(member_row.source.layer, "group");
    assert_eq!(outsider_row.effective, "deny");
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
