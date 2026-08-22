//! `repositories::departments` read models — the department dashboard rollups
//! and the cross-user aggregates the management views read.

use systemprompt_web_admin::repositories::departments;
use systemprompt_web_admin::util::org_scope::OrgScope;

use crate::fixtures::{
    insert_acl_rule, insert_department, insert_user, insert_user_full, set_department,
    unclaimed_email, unique,
};
use crate::tempdb::TempDb;

const HOUSE: &str = "house";

#[tokio::test]
async fn list_department_names_includes_a_new_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Legal");
    insert_department(&db.pool, &unique("dept"), &name, HOUSE).await;

    let names = departments::list_department_names(&db.pool, &OrgScope::AllOrganizations)
        .await
        .expect("listing succeeds");

    assert!(names.contains(&name));
    assert!(names.contains(&"Default".to_owned()));
    db.cleanup().await;
}

#[tokio::test]
async fn list_departments_counts_members_through_the_users_table() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Ops");
    insert_department(&db.pool, &unique("dept"), &name, HOUSE).await;
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ops")).await;
    set_department(&db.pool, &user, &name).await;

    let listed = departments::list_departments(&db.pool)
        .await
        .expect("listing succeeds");

    let row = listed
        .iter()
        .find(|d| d.name == name)
        .expect("the new department is listed");
    assert_eq!(row.member_count, 1);
    assert_eq!(row.requests, 0);
    assert_eq!(row.cost_microdollars, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_department_members_excludes_anonymous_identities() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Mixed");
    insert_department(&db.pool, &unique("dept"), &name, HOUSE).await;
    let person = insert_user(&db.pool, &unique("user"), &unclaimed_email("person")).await;
    let robot = insert_user_full(
        &db.pool,
        &unique("anon"),
        &unclaimed_email("robot"),
        None,
        &["anonymous".to_owned()],
        "active",
    )
    .await;
    set_department(&db.pool, &person, &name).await;
    set_department(&db.pool, &robot, &name).await;

    let members = departments::list_department_members(&db.pool, &name)
        .await
        .expect("listing succeeds");

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].id, person.as_str());
    db.cleanup().await;
}

#[tokio::test]
async fn list_department_members_is_empty_for_an_unknown_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let members = departments::list_department_members(&db.pool, &unique("Nowhere"))
        .await
        .expect("listing succeeds");

    assert!(members.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_department_top_tools_is_empty_without_daily_rollups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Quiet");
    insert_department(&db.pool, &unique("dept"), &name, HOUSE).await;

    let tools = departments::list_department_top_tools(&db.pool, &name, 10)
        .await
        .expect("listing succeeds");

    assert!(tools.is_empty());
    db.cleanup().await;
}

#[tokio::test]
async fn list_user_management_aggregates_counts_the_grants_a_user_receives() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Grants");
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("granted")).await;
    set_department(&db.pool, &user, &name).await;
    insert_acl_rule(
        &db.pool,
        "skill",
        &unique("skill"),
        "department",
        &name,
        "allow",
    )
    .await;
    insert_acl_rule(
        &db.pool,
        "skill",
        &unique("skill"),
        "user",
        user.as_str(),
        "allow",
    )
    .await;
    insert_acl_rule(
        &db.pool,
        "skill",
        &unique("skill"),
        "user",
        user.as_str(),
        "deny",
    )
    .await;

    let rows = departments::list_user_management_aggregates(&db.pool)
        .await
        .expect("listing succeeds");

    let row = rows
        .iter()
        .find(|r| r.user_id == user)
        .expect("the user is aggregated");
    assert_eq!(row.department, name);
    assert_eq!(
        row.assigned_skills_count, 2,
        "only 'allow' grants count, from either the user or their department"
    );
    assert_eq!(row.devices_count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn list_user_marketplace_overrides_reports_both_scopes() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Overrides");
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("override")).await;
    set_department(&db.pool, &user, &name).await;
    let by_department = unique("market");
    let by_user = unique("market");
    insert_acl_rule(
        &db.pool,
        "marketplace",
        &by_department,
        "department",
        &name,
        "allow",
    )
    .await;
    insert_acl_rule(
        &db.pool,
        "marketplace",
        &by_user,
        "user",
        user.as_str(),
        "deny",
    )
    .await;

    let rows = departments::list_user_marketplace_overrides(&db.pool)
        .await
        .expect("listing succeeds");

    let mine: Vec<_> = rows.iter().filter(|r| r.user_id == user).collect();
    assert_eq!(mine.len(), 2, "the same user picks up both rule scopes");
    assert!(
        mine.iter()
            .any(|r| r.entity_id == by_department && r.access == "allow")
    );
    assert!(
        mine.iter()
            .any(|r| r.entity_id == by_user && r.access == "deny")
    );
    db.cleanup().await;
}
