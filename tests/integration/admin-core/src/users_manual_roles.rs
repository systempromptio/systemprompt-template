//! `repositories::users::roles` — the two halves of a role set and the
//! recomputation that joins them.
//!
//! `users.roles` is what every authorisation check reads, and it has two
//! writers. Only the manual half is stored, so the directory half is whatever
//! the effective set holds beyond it. These tests pin that derivation, because
//! getting it wrong either loses an admin's deliberate grant at the next
//! sign-in or makes a directory revocation un-revocable.

use systemprompt_web_admin::repositories::users::roles::{
    count_platform_admins, list_directory_roles, list_manual_roles, recompute_roles,
    set_manual_roles,
};

use crate::fixtures::{insert_user, insert_user_full, unclaimed_email, unique};
use crate::tempdb::TempDb;

fn roles(list: &[&str]) -> Vec<String> {
    list.iter().map(|r| (*r).to_owned()).collect()
}

#[tokio::test]
async fn a_fresh_user_holds_only_directory_roles() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("fresh")).await;

    assert!(
        list_manual_roles(&db.pool, &user)
            .await
            .expect("list manual")
            .is_empty()
    );
    assert_eq!(
        list_directory_roles(&db.pool, &user)
            .await
            .expect("list directory"),
        roles(&["user"]),
        "everything the row holds and nothing granted by hand is, by definition, the directory's"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_manual_grant_is_added_to_the_effective_set() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("manual")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter")).await;

    set_manual_roles(&db.pool, &user, &roles(&["developer"]), &admin)
        .await
        .expect("grant developer");
    let effective = recompute_roles(&db.pool, &user, None)
        .await
        .expect("recompute");

    assert_eq!(effective, roles(&["developer", "user"]));
    assert_eq!(
        list_manual_roles(&db.pool, &user)
            .await
            .expect("list manual"),
        roles(&["developer"])
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_sign_in_rewrites_the_directory_half_and_keeps_the_manual_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("signin"),
        None,
        &roles(&["user", "admin"]),
        "active",
    )
    .await;
    let granter = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter2")).await;
    set_manual_roles(&db.pool, &user, &roles(&["developer"]), &granter)
        .await
        .expect("grant developer by hand");

    let effective = recompute_roles(&db.pool, &user, Some(&roles(&["user"])))
        .await
        .expect("sign-in with admin taken away");

    assert_eq!(
        effective,
        roles(&["developer", "user"]),
        "the directory dropped admin; the dashboard grant is untouched"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn setting_manual_roles_replaces_rather_than_accumulates() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("replace")).await;
    let granter = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter3")).await;

    set_manual_roles(&db.pool, &user, &roles(&["developer", "admin"]), &granter)
        .await
        .expect("first edit");
    set_manual_roles(&db.pool, &user, &roles(&["developer"]), &granter)
        .await
        .expect("second edit drops admin");

    assert_eq!(
        list_manual_roles(&db.pool, &user)
            .await
            .expect("list manual"),
        roles(&["developer"])
    );
    db.cleanup().await;
}

#[tokio::test]
async fn clearing_every_manual_role_leaves_the_directory_set_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("clear")).await;
    let granter = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter4")).await;
    set_manual_roles(&db.pool, &user, &roles(&["developer"]), &granter)
        .await
        .expect("grant");

    set_manual_roles(&db.pool, &user, &[], &granter)
        .await
        .expect("revoke everything manual");
    let effective = recompute_roles(&db.pool, &user, None)
        .await
        .expect("recompute");

    assert_eq!(effective, roles(&["user"]));
    db.cleanup().await;
}

#[tokio::test]
async fn a_role_held_both_ways_counts_as_manual() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("both")).await;
    let granter = insert_user(&db.pool, &unique("admin"), &unclaimed_email("granter5")).await;
    set_manual_roles(&db.pool, &user, &roles(&["user"]), &granter)
        .await
        .expect("grant the role the directory also holds");

    assert!(
        list_directory_roles(&db.pool, &user)
            .await
            .expect("list directory")
            .is_empty(),
        "the safe direction: revoking it is refused by no rule, and the next sign-in restores it"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn platform_admins_are_counted_across_every_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let before = count_platform_admins(&db.pool).await.expect("count");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &unclaimed_email("platform"),
        None,
        &roles(&["platform_admin", "user"]),
        "active",
    )
    .await;

    let after = count_platform_admins(&db.pool).await.expect("count");

    assert_eq!(after, before + 1, "the last-platform-admin rule reads this");
    db.cleanup().await;
}
