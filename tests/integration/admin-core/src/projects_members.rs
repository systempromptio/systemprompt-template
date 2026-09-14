//! `repositories::projects::members` — attribution membership, and the same
//! two-writer rule groups carry.

use systemprompt_web_admin::repositories::projects::members::{
    delete_project_member, insert_project_member, list_project_ids_for_user, list_project_members,
    replace_directory_project_memberships,
};

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;
use systemprompt_web_shared::ProjectId;

#[tokio::test]
async fn a_manual_member_is_listed_with_its_source() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("member")).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("admin")).await;

    insert_project_member(&db.pool, &ProjectId::new("core"), &user, &admin)
        .await
        .expect("add");

    let members = list_project_members(&db.pool, &ProjectId::new("core"))
        .await
        .expect("list");
    let row = members
        .iter()
        .find(|m| m.user_id == user.as_str())
        .expect("the member is listed");
    assert_eq!(row.sources, vec!["manual".to_owned()]);
    assert_eq!(
        list_project_ids_for_user(&db.pool, &user)
            .await
            .expect("read"),
        vec![ProjectId::new("core")]
    );
    db.cleanup().await;
}

// Why: removing it here would be undone at the member's next sign-in, so the
// refusal says what is actually true — the change belongs in AD.
#[tokio::test]
async fn a_directory_sourced_member_cannot_be_removed_by_hand() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sso")).await;
    sqlx::query(
        "INSERT INTO project_ad_mappings (ad_group, project_id, source) \
         VALUES ('AD-Core', 'core', 'yaml') ON CONFLICT DO NOTHING",
    )
    .execute(&*db.pool)
    .await
    .expect("map");
    replace_directory_project_memberships(&db.pool, &user, &["AD-Core".to_owned()])
        .await
        .expect("sign-in");

    let err = delete_project_member(&db.pool, &ProjectId::new("core"), &user)
        .await
        .expect_err("refused");

    assert_eq!(err.status().as_u16(), 409);
    assert_eq!(
        list_project_ids_for_user(&db.pool, &user)
            .await
            .expect("read"),
        vec![ProjectId::new("core")],
        "the membership survives the refusal"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn removing_a_member_who_is_not_one_is_a_not_found() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("stranger")).await;

    let err = delete_project_member(&db.pool, &ProjectId::new("core"), &user)
        .await
        .expect_err("refused");

    assert_eq!(err.status().as_u16(), 404);
    db.cleanup().await;
}
