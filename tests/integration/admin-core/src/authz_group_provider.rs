//! The `group` subject dimension: the provider that reads a user's group
//! memberships and the band it occupies.
//!
//! `group` is not a core concept. It reaches the resolver only because this
//! extension registers a `SubjectAttributeProvider` for it, so a value read
//! back here proves the whole registration path rather than one query.

use std::sync::Arc;

use systemprompt_security::authz::SubjectAttributeProvider;
use systemprompt_web_admin::authz::group::{GroupAttributeProvider, group_rule_type, invalidate};

use crate::fixtures::{
    insert_group, insert_group_member, insert_user, unclaimed_email, unique, unique_group,
};
use crate::tempdb::TempDb;

#[tokio::test]
async fn the_provider_resolves_a_users_groups() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("gdim")).await;
    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, "manual").await;
    let provider = GroupAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;

    let values = provider
        .values_for(&user)
        .await
        .expect("group membership lookup");

    assert_eq!(values, vec![group.as_str().to_owned()]);
    db.cleanup().await;
}

#[tokio::test]
async fn a_user_in_no_group_resolves_to_unassigned_here_too() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("gunassigned")).await;
    let provider = GroupAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;

    let values = provider
        .values_for(&user)
        .await
        .expect("group membership lookup");

    assert_eq!(
        values,
        vec!["unassigned".to_owned()],
        "a rule written against unassigned must actually bind, or the group is decorative"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn the_group_band_sits_between_user_and_role() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let provider = GroupAttributeProvider::new(Arc::new((*db.pool).clone()));

    let dimension = provider.dimension();

    assert_eq!(dimension.rule_type, group_rule_type());
    assert_eq!(dimension.rule_type.as_str(), "group");
    assert_eq!(dimension.precedence, 150);
    db.cleanup().await;
}

#[tokio::test]
async fn invalidating_makes_a_membership_change_bind_on_the_next_read() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("ginval")).await;
    let provider = GroupAttributeProvider::new(Arc::new((*db.pool).clone()));
    invalidate(&user).await;
    assert_eq!(
        provider
            .values_for(&user)
            .await
            .expect("group membership lookup"),
        vec!["unassigned".to_owned()]
    );

    let group = unique_group("grp");
    insert_group(&db.pool, &group, "Commerce").await;
    insert_group_member(&db.pool, &group, &user, "adfs").await;
    invalidate(&user).await;

    assert_eq!(
        provider
            .values_for(&user)
            .await
            .expect("group membership lookup"),
        vec![group.as_str().to_owned()],
        "sign-in invalidates rather than waiting out the TTL"
    );
    db.cleanup().await;
}
