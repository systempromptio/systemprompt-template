//! `repositories::groups::mappings` and
//! `members::replace_directory_group_memberships` — what an AD group claim
//! resolves to, and what a sign-in does to it.

use systemprompt_web_admin::repositories::groups::crud::insert_group;
use systemprompt_web_admin::repositories::groups::mappings::{
    delete_group_ad_mapping, insert_group_ad_mapping, list_group_ad_mappings,
};
use systemprompt_web_admin::repositories::groups::members::{
    insert_group_member, list_group_ids_for_user, list_source_ad_groups,
    replace_directory_group_memberships,
};
use systemprompt_web_admin::types::groups::CreateGroupRequest;
use systemprompt_web_shared::GroupId;

use crate::fixtures::{insert_user, unclaimed_email, unique, unique_group};
use crate::tempdb::TempDb;

async fn seed_group(pool: &sqlx::PgPool, id: &GroupId) {
    insert_group(
        pool,
        &CreateGroupRequest {
            id: id.clone(),
            name: id.as_str().to_owned(),
            description: None,
        },
        "dashboard",
    )
    .await
    .expect("insert group");
}

#[tokio::test]
async fn a_mapping_is_listed_once_and_deleted_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let group = unique_group("grp");
    seed_group(&db.pool, &group).await;

    insert_group_ad_mapping(&db.pool, &group, "AD-Engineering", "dashboard")
        .await
        .expect("insert");
    let duplicate = insert_group_ad_mapping(&db.pool, &group, "AD-Engineering", "dashboard").await;
    assert!(duplicate.is_err(), "the same pair cannot be mapped twice");

    let mappings = list_group_ad_mappings(&db.pool, &group)
        .await
        .expect("list");
    assert_eq!(mappings.len(), 1);
    assert_eq!(mappings[0].ad_group, "AD-Engineering");

    delete_group_ad_mapping(&db.pool, &group, "AD-Engineering")
        .await
        .expect("delete");
    assert!(
        list_group_ad_mappings(&db.pool, &group)
            .await
            .expect("list")
            .is_empty()
    );
    db.cleanup().await;
}

// Why: the whole point of `source` in the primary key. A sign-in has to be
// able to withdraw what AD no longer says without touching what an admin
// added by hand.
#[tokio::test]
async fn a_sign_in_replaces_directory_rows_and_leaves_manual_ones() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let directory_group = unique_group("dir");
    let manual_group = unique_group("man");
    seed_group(&db.pool, &directory_group).await;
    seed_group(&db.pool, &manual_group).await;
    insert_group_ad_mapping(&db.pool, &directory_group, "AD-Commerce", "yaml")
        .await
        .expect("map");

    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sso")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;
    insert_group_member(&db.pool, &manual_group, &user, &admin)
        .await
        .expect("manual grant");

    replace_directory_group_memberships(&db.pool, &user, &["AD-Commerce".to_owned()])
        .await
        .expect("first sign-in");
    let mut groups = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("read");
    groups.sort();
    assert!(groups.contains(&directory_group));
    assert!(groups.contains(&manual_group));
    assert_eq!(
        list_source_ad_groups(&db.pool, &user).await.expect("read"),
        vec!["AD-Commerce".to_owned()],
        "the AD group that produced the row is recorded on it"
    );

    replace_directory_group_memberships(&db.pool, &user, &[])
        .await
        .expect("second sign-in, no groups");
    let groups = list_group_ids_for_user(&db.pool, &user)
        .await
        .expect("read");
    assert!(
        !groups.contains(&directory_group),
        "leaving the AD group withdraws the membership"
    );
    assert!(
        groups.contains(&manual_group),
        "the manual grant survives the directory's replace"
    );
    db.cleanup().await;
}

// Why: an AD group nothing maps to must not invent a membership. The user
// falls through to the derived `unassigned` row instead.
#[tokio::test]
async fn an_unmapped_ad_group_leaves_the_user_unassigned() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("unmapped")).await;

    replace_directory_group_memberships(&db.pool, &user, &["AD-Nothing-Maps-Here".to_owned()])
        .await
        .expect("sign-in");

    assert_eq!(
        list_group_ids_for_user(&db.pool, &user)
            .await
            .expect("read"),
        vec![GroupId::new("unassigned")]
    );
    db.cleanup().await;
}
