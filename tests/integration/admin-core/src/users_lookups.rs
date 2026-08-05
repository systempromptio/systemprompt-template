//! `repositories::users` read models — the user index, its filter options, and
//! the per-user side tables the detail page renders.

use systemprompt_web_admin::repositories::users;

use crate::fixtures::{insert_user, insert_user_full, set_department, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn list_users_includes_a_freshly_created_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("listed")).await;

    let listed = users::queries::list_users(&db.pool)
        .await
        .expect("listing succeeds");

    let row = listed
        .iter()
        .find(|u| u.user_id == user)
        .expect("the new user appears in the index");
    assert!(row.is_active);
    assert_eq!(row.total_events, 0, "a user with no traffic has no events");
    db.cleanup().await;
}

#[tokio::test]
async fn list_users_excludes_anonymous_identities() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let anon = insert_user_full(
        &db.pool,
        &unique("anon"),
        &unclaimed_email("anon"),
        None,
        &["anonymous".to_owned()],
    )
    .await;

    let listed = users::queries::list_users(&db.pool)
        .await
        .expect("listing succeeds");

    assert!(
        !listed.iter().any(|u| u.user_id == anon),
        "the anonymous role is filtered out of the user index"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn list_distinct_roles_surfaces_a_new_role_and_drops_machine_roles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let role = unique("auditor");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("roled"),
        None,
        std::slice::from_ref(&role),
    )
    .await;
    insert_user_full(
        &db.pool,
        &unique("svc"),
        &unclaimed_email("svc"),
        None,
        &["service".to_owned()],
    )
    .await;

    let roles = users::queries::list_distinct_roles(&db.pool)
        .await
        .expect("listing succeeds");

    assert!(roles.contains(&role));
    assert!(
        !roles.contains(&"service".to_owned()),
        "machine roles are filtered out of the operator-facing list"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_returns_none_without_a_profile_row() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share")).await;

    let version = users::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(
        version, None,
        "absence of a profile row is not an error for find_"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn find_share_token_version_starts_at_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("share2")).await;
    set_department(&db.pool, &user, "Default").await;

    let version = users::find_share_token_version(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert_eq!(version, Some(1));
    db.cleanup().await;
}

#[tokio::test]
async fn find_user_settings_returns_none_when_the_user_has_none() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("settings")).await;

    let settings = users::user_settings::find_user_settings(&db.pool, &user)
        .await
        .expect("lookup succeeds");

    assert!(settings.is_none());
    db.cleanup().await;
}
