//! `repositories::groups::crud` — the group rows, and the one row that cannot
//! be deleted.

use systemprompt_web_admin::repositories::groups::crud::{
    delete_group, find_group, insert_group, list_group_summaries, list_groups, update_group,
};
use systemprompt_web_admin::types::groups::{CreateGroupRequest, UpdateGroupRequest};
use systemprompt_web_shared::GroupId;

use crate::fixtures::{insert_user, unclaimed_email, unique, unique_group};
use crate::tempdb::TempDb;

fn create(id: &GroupId) -> CreateGroupRequest {
    CreateGroupRequest {
        id: id.clone(),
        name: "Test group".to_owned(),
        description: Some("seeded by the group CRUD suite".to_owned()),
    }
}

#[tokio::test]
async fn a_created_group_is_readable_and_renamable() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique_group("grp");

    insert_group(&db.pool, &create(&id), "dashboard")
        .await
        .expect("insert");
    update_group(
        &db.pool,
        &id,
        &UpdateGroupRequest {
            name: Some("Renamed".to_owned()),
            description: None,
        },
    )
    .await
    .expect("rename");

    let found = find_group(&db.pool, &id).await.expect("read").expect("row");
    assert_eq!(found.name, "Renamed");
    assert_eq!(
        found.description.as_deref(),
        Some("seeded by the group CRUD suite"),
        "a rename that names no description leaves the old one standing"
    );
    assert!(!found.is_system);
    db.cleanup().await;
}

#[tokio::test]
async fn a_duplicate_id_is_a_conflict_not_a_database_error() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique_group("grp");
    insert_group(&db.pool, &create(&id), "dashboard")
        .await
        .expect("insert");

    let err = insert_group(&db.pool, &create(&id), "dashboard")
        .await
        .expect_err("second insert refused");

    assert_eq!(
        err.status().as_u16(),
        409,
        "a taken id is the caller's problem, not a 500"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_unassigned_group_cannot_be_deleted() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let err = delete_group(&db.pool, &GroupId::new("unassigned"))
        .await
        .expect_err("system group refused");

    assert_eq!(err.status().as_u16(), 409);
    assert!(
        find_group(&db.pool, &GroupId::new("unassigned"))
            .await
            .expect("read")
            .is_some(),
        "the row survives the refusal"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn deleting_a_group_removes_it_from_the_listing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique_group("grp");
    insert_group(&db.pool, &create(&id), "dashboard")
        .await
        .expect("insert");

    delete_group(&db.pool, &id).await.expect("delete");

    let ids: Vec<GroupId> = list_groups(&db.pool)
        .await
        .expect("list")
        .into_iter()
        .map(|g| g.id)
        .collect();
    assert!(!ids.contains(&id));
    db.cleanup().await;
}

// Why: a user with no membership row is in `unassigned` by derivation, so the
// summary counts have to come from the view or the system group reads empty
// on an instance where everyone is unplaced.
#[tokio::test]
async fn the_unassigned_summary_counts_derived_members() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    insert_user(&db.pool, &unique("user"), &unclaimed_email("derived")).await;

    let summaries = list_group_summaries(&db.pool).await.expect("summaries");
    let unassigned = summaries
        .iter()
        .find(|g| g.id == "unassigned")
        .expect("the system row is listed");

    assert!(unassigned.is_system);
    assert!(
        unassigned.member_count >= 1,
        "the user with no group row counts as a member"
    );
    db.cleanup().await;
}
