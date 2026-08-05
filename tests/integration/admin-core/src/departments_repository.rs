//! `repositories::departments` — record lifecycle, the rename cascade, the
//! guarded delete, and the cross-user rollups the management views read.
//!
//! `departments.name` is unique and migration 009 seeds a `Default` row, so
//! every fixture name is minted through `unique` and no test asserts on the
//! table being empty.

use systemprompt_web_admin::repositories::departments;
use systemprompt_web_admin::types::DepartmentInput;

use crate::fixtures::{
    AclRuleSpec, insert_acl_rule, insert_department, insert_user, set_department, unclaimed_email,
    unique,
};
use crate::tempdb::TempDb;


fn input(name: &str) -> DepartmentInput {
    DepartmentInput {
        name: name.to_owned(),
        description: "under test".to_owned(),
    }
}

#[tokio::test]
async fn find_department_returns_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = departments::find_department(&db.pool, &unique("absent"))
        .await
        .expect("lookup succeeds");

    assert!(found.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_department_reads_back_what_was_inserted() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    let name = unique("Research");
    insert_department(&db.pool, &id, &name).await;

    let found = departments::find_department(&db.pool, &id)
        .await
        .expect("lookup succeeds")
        .expect("the department exists");

    assert_eq!(found.name, name);
    assert_eq!(found.description, "fixture department");
    db.cleanup().await;
}

#[tokio::test]
async fn find_department_by_name_returns_none_for_an_unknown_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = departments::find_department_by_name(&db.pool, &unique("Nowhere"))
        .await
        .expect("lookup succeeds");

    assert!(found.is_none());
    db.cleanup().await;
}

#[tokio::test]
async fn find_department_by_name_finds_the_seeded_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = departments::find_department_by_name(&db.pool, "Default")
        .await
        .expect("lookup succeeds")
        .expect("migration 009 seeds the Default department");

    assert_eq!(found.name, "Default");
    db.cleanup().await;
}

#[tokio::test]
async fn create_department_returns_the_row_it_inserted() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("New");

    let created = departments::create_department(&db.pool, &input(&name))
        .await
        .expect("create_department succeeds");

    assert_eq!(created.name, name);
    assert_eq!(created.description, "under test");
    let found = departments::find_department(&db.pool, &created.id)
        .await
        .expect("lookup succeeds")
        .expect("the new department is readable");
    assert_eq!(found.name, name);
    db.cleanup().await;
}

#[tokio::test]
async fn create_department_rejects_a_name_already_taken() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let name = unique("Dup");
    departments::create_department(&db.pool, &input(&name))
        .await
        .expect("the first insert succeeds");

    let err = departments::create_department(&db.pool, &input(&name))
        .await
        .expect_err("departments.name is unique");

    assert!(
        err.to_string().contains("duplicate key"),
        "unexpected failure mode: {err}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_department_rewrites_name_and_description() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    insert_department(&db.pool, &id, &unique("Before")).await;
    let after = unique("After");

    let updated = departments::update_department(&db.pool, &id, &input(&after))
        .await
        .expect("update succeeds");

    assert_eq!(updated.name, after);
    assert_eq!(updated.description, "under test");
    db.cleanup().await;
}

#[tokio::test]
async fn update_department_errors_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    departments::update_department(&db.pool, &unique("absent"), &input("Anything"))
        .await
        .expect_err("the row is selected FOR UPDATE first, so an absent id is an error");
    db.cleanup().await;
}

#[tokio::test]
async fn update_department_carries_members_across_a_rename() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    let before = unique("Sales");
    insert_department(&db.pool, &id, &before).await;
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    set_department(&db.pool, &user, &before).await;
    let after = unique("Revenue");

    departments::update_department(&db.pool, &id, &input(&after))
        .await
        .expect("update succeeds");

    let now: String =
        sqlx::query_scalar("SELECT department FROM user_profile_ext WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the profile row exists");
    assert_eq!(
        now, after,
        "a rename must not orphan the department's members"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_department_carries_access_rules_across_a_rename() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    let before = unique("Support");
    insert_department(&db.pool, &id, &before).await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &unique("skill"), "department", &before),
    )
    .await;
    let after = unique("Service");

    departments::update_department(&db.pool, &id, &input(&after))
        .await
        .expect("update succeeds");

    let moved: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_control_rules
         WHERE rule_type = 'department' AND rule_value = $1",
    )
    .bind(&after)
    .fetch_one(&*db.pool)
    .await
    .expect("count succeeds");
    assert_eq!(moved, 1, "a rename must not silently drop the grants");
    db.cleanup().await;
}

#[tokio::test]
async fn delete_department_reassigns_its_members_to_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    let name = unique("Doomed");
    insert_department(&db.pool, &id, &name).await;
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("orphan")).await;
    set_department(&db.pool, &user, &name).await;

    departments::delete_department(&db.pool, &id)
        .await
        .expect("delete succeeds");

    let now: String =
        sqlx::query_scalar("SELECT department FROM user_profile_ext WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the profile row exists");
    assert_eq!(now, "Default");
    assert!(
        departments::find_department(&db.pool, &id)
            .await
            .expect("lookup succeeds")
            .is_none()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn delete_department_removes_its_access_rules() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("dept");
    let name = unique("Doomed");
    insert_department(&db.pool, &id, &name).await;
    insert_acl_rule(
        &db.pool,
        &AclRuleSpec::allow("skill", &unique("skill"), "department", &name),
    )
    .await;

    departments::delete_department(&db.pool, &id)
        .await
        .expect("delete succeeds");

    let left: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM access_control_rules
         WHERE rule_type = 'department' AND rule_value = $1",
    )
    .bind(&name)
    .fetch_one(&*db.pool)
    .await
    .expect("count succeeds");
    assert_eq!(left, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn delete_department_refuses_to_remove_default() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let default = departments::find_department_by_name(&db.pool, "Default")
        .await
        .expect("lookup succeeds")
        .expect("Default is seeded");

    let err = departments::delete_department(&db.pool, &default.id)
        .await
        .expect_err("Default is where the delete path reassigns members");

    assert!(
        err.to_string().contains("cannot be deleted"),
        "unexpected: {err}"
    );
    assert!(
        departments::find_department_by_name(&db.pool, "Default")
            .await
            .expect("lookup succeeds")
            .is_some(),
        "the refusal must roll back, not half-delete"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn assign_user_to_department_is_an_upsert() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("assign")).await;

    departments::assign_user_to_department(&db.pool, &user, "Engineering")
        .await
        .expect("first assignment succeeds");
    departments::assign_user_to_department(&db.pool, &user, "Finance")
        .await
        .expect("reassignment succeeds");

    let now: String =
        sqlx::query_scalar("SELECT department FROM user_profile_ext WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the profile row exists");
    assert_eq!(now, "Finance");
    db.cleanup().await;
}
