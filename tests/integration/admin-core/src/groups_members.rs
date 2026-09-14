//! `repositories::groups::members` — the two membership sources, the derived
//! `unassigned` group, and what the directory's replace-on-sign-in is allowed
//! to touch.
//!
//! The invariant under test is that a dashboard grant and a directory grant
//! are separate rows. A sign-in rewrites only the directory half; anything
//! else would make an operator's deliberate act evaporate the next time
//! someone logged in.

use systemprompt_web_admin::repositories::groups::members::{
    delete_group_member, insert_group_member, list_group_ids_for_user, list_group_members,
    list_source_ad_groups, list_unassigned_users, replace_directory_group_memberships,
};

use crate::fixtures::{insert_group, insert_user, unclaimed_email, unique, unique_group};
use crate::tempdb::TempDb;
use systemprompt_web_shared::GroupId;

async fn map_ad_group(pool: &sqlx::PgPool, ad_group: &str, group_id: &str) {
    sqlx::query(
        "INSERT INTO group_ad_mappings (ad_group, group_id, source) VALUES ($1, $2, 'yaml')
         ON CONFLICT DO NOTHING",
    )
    .bind(ad_group)
    .bind(group_id)
    .execute(pool)
    .await
    .expect("map ad group");
}

#[tokio::test]
async fn a_user_with_no_membership_is_derived_into_unassigned() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("nogroup")).await;

    let groups = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("list groups");

    assert_eq!(
        groups,
        vec![GroupId::new("unassigned")],
        "membership is computed, never written"
    );
    let unassigned = list_unassigned_users(&db.pool)
        .await
        .expect("list unassigned");
    assert!(unassigned.contains(&user));
    db.cleanup().await;
}

#[tokio::test]
async fn one_real_membership_removes_the_derived_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("placed")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Placed").await;
    insert_group_member(&db.pool, &group, &user, &user)
        .await
        .expect("manual grant");

    let groups = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("list groups");

    assert_eq!(groups, vec![group]);
    let unassigned = list_unassigned_users(&db.pool)
        .await
        .expect("list unassigned");
    assert!(!unassigned.contains(&user));
    db.cleanup().await;
}

#[tokio::test]
async fn the_directory_replaces_only_its_own_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("both")).await;
    let by_hand = unique_group("grp");
    let from_ad = unique_group("grp");
    insert_group(&db.pool, &by_hand, "By hand").await;
    insert_group(&db.pool, &from_ad, "From AD").await;
    map_ad_group(&db.pool, "Systemprompt-Commerce", from_ad.as_str()).await;
    insert_group_member(&db.pool, &by_hand, &user, &user)
        .await
        .expect("manual grant");

    replace_directory_group_memberships(&db.pool, &user, &["Systemprompt-Commerce".to_owned()])
        .await
        .expect("first sign-in");
    let after_first = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("list groups");

    replace_directory_group_memberships(&db.pool, &user, &[])
        .await
        .expect("second sign-in, no group claim");
    let after_second = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("list groups");

    let mut expected = vec![by_hand.clone(), from_ad];
    expected.sort();
    assert_eq!(after_first, expected);
    assert_eq!(
        after_second,
        vec![by_hand],
        "the AD row went; the dashboard grant stayed"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_unmapped_ad_group_places_nobody_and_that_is_not_an_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("unmapped")).await;

    replace_directory_group_memberships(&db.pool, &user, &["Systemprompt-Unknown".to_owned()])
        .await
        .expect("a group the DB does not map is not a failure");

    assert_eq!(
        list_group_ids_for_user(&db.pool, &user)
            .await
            .expect("list groups"),
        vec![GroupId::new("unassigned")]
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_ad_group_that_produced_a_membership_is_recorded_on_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("source")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    map_ad_group(&db.pool, "Systemprompt-Commerce", group.as_str()).await;

    replace_directory_group_memberships(&db.pool, &user, &["Systemprompt-Commerce".to_owned()])
        .await
        .expect("sign-in");

    assert_eq!(
        list_source_ad_groups(&db.pool, &user)
            .await
            .expect("list source ad groups"),
        vec!["Systemprompt-Commerce".to_owned()],
        "the bridge shows this on the account card"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn a_directory_membership_cannot_be_removed_by_hand() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("refuse")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    map_ad_group(&db.pool, "Systemprompt-Commerce", group.as_str()).await;
    replace_directory_group_memberships(&db.pool, &user, &["Systemprompt-Commerce".to_owned()])
        .await
        .expect("sign-in");

    let refusal = delete_group_member(&db.pool, &group, &user)
        .await
        .expect_err("a directory row is not the dashboard's to remove");

    assert!(
        refusal.to_string().contains("directory"),
        "the refusal must say where the change belongs: {refusal}"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn removing_a_manual_grant_leaves_the_directory_row_in_place() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("mixed")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    map_ad_group(&db.pool, "Systemprompt-Commerce", group.as_str()).await;
    replace_directory_group_memberships(&db.pool, &user, &["Systemprompt-Commerce".to_owned()])
        .await
        .expect("sign-in");
    insert_group_member(&db.pool, &group, &user, &user)
        .await
        .expect("manual grant on top");

    delete_group_member(&db.pool, &group, &user)
        .await
        .expect("the manual half is removable");

    assert_eq!(
        list_group_ids_for_user(&db.pool, &user)
            .await
            .expect("list groups"),
        vec![group],
        "the directory still places them"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn granting_the_same_membership_twice_is_a_conflict() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("twice")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, &user)
        .await
        .expect("first grant");

    let refusal = insert_group_member(&db.pool, &group, &user, &user)
        .await
        .expect_err("already a member");

    assert!(refusal.to_string().contains("already"), "{refusal}");
    db.cleanup().await;
}

#[tokio::test]
async fn removing_someone_who_is_not_a_member_is_a_not_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("absent")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;

    let refusal = delete_group_member(&db.pool, &group, &user)
        .await
        .expect_err("not a member");

    assert!(refusal.to_string().contains("not a member"), "{refusal}");
    db.cleanup().await;
}

#[tokio::test]
async fn the_member_list_carries_the_identity_a_dashboard_shows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("listed");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, &user)
        .await
        .expect("grant");

    let members = list_group_members(&db.pool, &group)
        .await
        .expect("list members");

    assert_eq!(members.len(), 1);
    assert_eq!(members[0].user_id, user.as_str());
    assert_eq!(members[0].email.as_deref(), Some(email.as_str()));
    db.cleanup().await;
}
