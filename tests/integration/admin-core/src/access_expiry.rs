//! A membership past its `valid_until` is not a membership.
//!
//! The predicate lives in the `user_groups` view and the member queries, so
//! one expired row must vanish from the resolver's `group` dimension, from
//! the group's member list, and from the person's group ids — while the row
//! itself stays until the sweep stamps it revoked.

use std::sync::Arc;

use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin::authz::group::GroupAttributeProvider;
use systemprompt_web_admin::repositories::groups::members::{
    list_group_ids_for_user, list_group_members,
};
use systemprompt_web_admin::repositories::scope::expiry::revoke_expired_memberships;
use systemprompt_web_shared::GroupId;

use crate::fixtures::{insert_group, insert_group_member, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

async fn expire(pool: &sqlx::PgPool, group: &str, user: &str) {
    sqlx::query(
        "UPDATE group_members SET valid_until = NOW() - INTERVAL '1 hour'
         WHERE group_id = $1 AND user_id = $2",
    )
    .bind(group)
    .bind(user)
    .execute(pool)
    .await
    .expect("expire membership");
}

#[tokio::test]
async fn an_expired_membership_is_invisible_to_the_resolver() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("expiry")).await;
    let group = unique("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, "manual").await;
    let provider = GroupAttributeProvider::new(Arc::new((*db.pool).clone()));
    assert_eq!(
        provider.values_for(&user).await.expect("resolves"),
        vec![group.clone()]
    );

    expire(&db.pool, &group, user.as_str()).await;

    assert_eq!(
        provider.values_for(&user).await.expect("resolves"),
        vec!["unassigned".to_owned()],
        "the window closed, so the person falls back to unassigned at once"
    );
    assert!(
        list_group_ids_for_user(&db.pool, &user)
            .await
            .expect("group ids")
            .iter()
            .all(|g| g.as_str() != group)
    );
    assert!(
        list_group_members(&db.pool, &GroupId::new(group.clone()))
            .await
            .expect("members")
            .is_empty(),
        "the member list reads the same predicate"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_sweep_stamps_the_expired_row_revoked_and_names_the_person() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("sweep")).await;
    let group = unique("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, "manual").await;
    expire(&db.pool, &group, user.as_str()).await;

    let swept = revoke_expired_memberships(&db.pool).await.expect("sweep");

    assert!(swept.users.contains(&user));
    let revoked: bool = sqlx::query_scalar(
        "SELECT revoked_at IS NOT NULL FROM group_members WHERE group_id = $1 AND user_id = $2",
    )
    .bind(&group)
    .bind(user.as_str())
    .fetch_one(&*db.pool)
    .await
    .expect("row survives as the audit trail");
    assert!(revoked);

    let again = revoke_expired_memberships(&db.pool).await.expect("sweep");
    assert!(
        !again.users.contains(&user),
        "a revoked row is not revoked twice"
    );
    db.cleanup().await;
}
