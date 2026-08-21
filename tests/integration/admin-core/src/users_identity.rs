//! `repositories::users` — identity CRUD, role/department lookup, the index
//! listing, and the per-user side tables.

use systemprompt::identifiers::{Email, UserId};
use systemprompt_web_admin::repositories::{organizations, users};
use systemprompt_web_admin::types::{CreateUserRequest, UpdateUserRequest};

use crate::fixtures::{insert_user, insert_user_full, set_department, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn create_request(user_id: &str, email: &str) -> CreateUserRequest {
    CreateUserRequest {
        user_id: UserId::new(user_id.to_owned()),
        display_name: "Fixture User".to_owned(),
        email: Email::try_new(email.to_owned()).expect("fixture email is valid"),
        roles: vec!["user".to_owned()],
        status: None,
        department: None,
    }
}

fn empty_update() -> UpdateUserRequest {
    UpdateUserRequest {
        display_name: None,
        email: None,
        roles: None,
        is_active: None,
        department: None,
    }
}

#[tokio::test]
async fn create_user_returns_the_row_it_inserted() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("user");
    let email = unclaimed_email("created");

    let summary = users::create_user(&db.pool, &create_request(&id, &email))
        .await
        .expect("create_user succeeds for an unclaimed domain")
        .summary;

    assert_eq!(summary.user_id.as_str(), id);
    assert_eq!(summary.display_name.as_deref(), Some("Fixture User"));
    assert!(summary.is_active, "no status given defaults to active");
    assert_eq!(summary.roles, vec!["user".to_owned()]);
    db.cleanup().await;
}

#[tokio::test]
async fn create_user_with_unclaimed_domain_joins_no_organization() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("user");
    let email = unclaimed_email("unattached");

    let summary = users::create_user(&db.pool, &create_request(&id, &email))
        .await
        .expect("create_user succeeds")
        .summary;

    let org = organizations::crud::find_organization_for_user(&db.pool, &summary.user_id)
        .await
        .expect("membership lookup succeeds");
    assert_eq!(
        org, None,
        "a domain no organization claims must leave the user unattached"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn create_user_honours_an_explicit_inactive_status() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mut req = create_request(&unique("user"), &unclaimed_email("inactive"));
    req.status = Some("inactive".to_owned());

    let summary = users::create_user(&db.pool, &req)
        .await
        .expect("create_user succeeds")
        .summary;

    assert!(!summary.is_active);
    db.cleanup().await;
}

#[tokio::test]
async fn create_user_on_a_taken_email_updates_rather_than_duplicating() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("conflict");
    let first = unique("user");
    users::create_user(&db.pool, &create_request(&first, &email))
        .await
        .expect("first create succeeds");

    let mut second = create_request(&unique("user"), &email);
    second.roles = vec!["user".to_owned(), "admin".to_owned()];
    let summary = users::create_user(&db.pool, &second)
        .await
        .expect("ON CONFLICT (email) updates the existing row")
        .summary;

    assert_eq!(
        summary.user_id.as_str(),
        first,
        "the conflicting insert must keep the original row's id"
    );
    assert!(summary.roles.contains(&"admin".to_owned()));
    db.cleanup().await;
}

#[tokio::test]
async fn update_user_returns_none_for_an_absent_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let missing = UserId::new(unique("absent"));

    let updated = users::update_user(&db.pool, &missing, &empty_update())
        .await
        .expect("update of an absent user is not an error");

    assert!(
        updated.is_none(),
        "update_ of an absent row reports None, not an error"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn update_user_renames_the_display_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("rename")).await;
    let mut req = empty_update();
    req.display_name = Some("Renamed".to_owned());

    let updated = users::update_user(&db.pool, &user, &req)
        .await
        .expect("update succeeds")
        .expect("an existing user yields a row");

    assert_eq!(updated.display_name.as_deref(), Some("Renamed"));
    db.cleanup().await;
}

#[tokio::test]
async fn update_user_deactivating_flips_is_active() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("deactivate")).await;
    let mut req = empty_update();
    req.is_active = Some(false);

    let updated = users::update_user(&db.pool, &user, &req)
        .await
        .expect("update succeeds")
        .expect("an existing user yields a row");

    assert!(!updated.is_active);
    db.cleanup().await;
}

#[tokio::test]
async fn update_user_writes_the_department_side_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("dept")).await;
    let mut req = empty_update();
    req.department = Some("Engineering".to_owned());

    users::update_user(&db.pool, &user, &req)
        .await
        .expect("update succeeds")
        .expect("an existing user yields a row");

    let found = users::queries::find_user_roles_department(&db.pool, &user)
        .await
        .expect("lookup succeeds")
        .expect("the user exists");
    assert_eq!(found.1, "Engineering");
    db.cleanup().await;
}

#[tokio::test]
async fn update_user_leaves_fields_the_request_omits() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("partial");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    let mut req = empty_update();
    req.roles = Some(vec!["admin".to_owned()]);

    let updated = users::update_user(&db.pool, &user, &req)
        .await
        .expect("update succeeds")
        .expect("an existing user yields a row");

    assert_eq!(updated.roles, vec!["admin".to_owned()]);
    assert_eq!(
        updated.email.as_ref().map(Email::as_str),
        Some(email.as_str()),
        "a None email must not clear the stored address"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn delete_user_reports_whether_a_row_went() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("delete")).await;

    let removed = users::delete_user(&db.pool, &user)
        .await
        .expect("delete succeeds");
    let removed_again = users::delete_user(&db.pool, &user)
        .await
        .expect("a second delete is not an error");

    assert!(removed, "the first delete removes the row");
    assert!(!removed_again, "the second finds nothing to remove");
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_roles_department_returns_none_for_an_absent_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let missing = UserId::new(unique("absent"));

    let found = users::queries::find_user_roles_department(&db.pool, &missing)
        .await
        .expect("lookup succeeds");

    assert!(
        found.is_none(),
        "find_ reports absence as None, not an error"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_roles_department_defaults_to_default_without_a_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("noprofile"),
        Some("No Profile"),
        &["user".to_owned(), "auditor".to_owned()],
        "active",
    )
    .await;

    let (roles, department) = users::queries::find_user_roles_department(&db.pool, &user)
        .await
        .expect("lookup succeeds")
        .expect("the user exists");

    assert_eq!(roles, vec!["user".to_owned(), "auditor".to_owned()]);
    assert_eq!(department, "Default");
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_roles_department_reads_the_assigned_department() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("assigned")).await;
    set_department(&db.pool, &user, "Support").await;

    let (_, department) = users::queries::find_user_roles_department(&db.pool, &user)
        .await
        .expect("lookup succeeds")
        .expect("the user exists");

    assert_eq!(department, "Support");
    db.cleanup().await;
}
