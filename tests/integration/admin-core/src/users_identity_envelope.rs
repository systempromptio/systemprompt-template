//! `repositories::users::queries::find_identity_envelope` — the single read
//! behind `GET /api/public/bridge/whoami`.
//!
//! The endpoint's whole reason to exist is that a caller's identity is spread
//! across several tables, so the assertions here are about the joins: that an
//! account with no SSO mapping and no group or project membership still
//! resolves, that the federated half and the memberships attach when they
//! exist, and that a user holding two IdP mappings reports the one they last
//! signed in with.

use systemprompt_web_admin::repositories::users::queries::find_identity_envelope;

use crate::fixtures::{
    insert_federated_identity, insert_user, insert_user_full, set_project, unclaimed_email, unique,
};
use crate::tempdb::TempDb;

const ISSUER: &str = "https://login.adfs.test/adfs";

#[tokio::test]
async fn a_local_account_resolves_with_no_federated_half_or_memberships() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("local")).await;

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(envelope.user_id, user);
    assert!(envelope.idp_issuer.is_none(), "never signed in via an IdP");
    // Why: an account in no group is in the derived `unassigned` group by
    // definition; projects have no such bucket.
    assert_eq!(
        envelope.group_ids,
        vec!["unassigned".to_owned()],
        "an account that never came through AD FS lands in unassigned"
    );
    assert!(
        envelope.project_ids.is_empty(),
        "an account that never came through AD FS is in no project"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_sso_account_carries_its_issuer_and_memberships() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("sso");
    let user = insert_user_full(
        &db.pool,
        &unique("user"),
        &email,
        Some("Federated Person"),
        &["admin".to_owned(), "user".to_owned()],
        "active",
    )
    .await;
    let sub = unique("sub");
    insert_federated_identity(&db.pool, ISSUER, &sub, &user).await;
    set_project(&db.pool, &user, Some("commerce")).await;

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(envelope.idp_issuer.as_deref(), Some(ISSUER));
    assert_eq!(envelope.external_sub.as_deref(), Some(sub.as_str()));
    assert_eq!(envelope.group_ids, vec!["commerce".to_owned()]);
    assert_eq!(envelope.project_ids, vec!["commerce".to_owned()]);
    assert_eq!(envelope.display_name.as_deref(), Some("Federated Person"));
    assert!(envelope.roles.contains(&"admin".to_owned()));
    assert!(
        envelope.linked_at.is_some() && envelope.last_seen_at.is_some(),
        "both federated timestamps are NOT NULL columns"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn two_mappings_report_the_one_most_recently_signed_in_with() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("two")).await;
    let old_sub = unique("sub-old");
    let new_sub = unique("sub-new");
    insert_federated_identity(&db.pool, "https://legacy.idp.test", &old_sub, &user).await;
    insert_federated_identity(&db.pool, ISSUER, &new_sub, &user).await;
    // Why: both rows default `last_seen_at` to now within the same statement
    // batch, so the ordering under test has to be made unambiguous rather than
    // raced against clock resolution.
    sqlx::query(
        "UPDATE federated_identities SET last_seen_at = NOW() - INTERVAL '1 day'
         WHERE issuer = $1 AND external_sub = $2",
    )
    .bind("https://legacy.idp.test")
    .bind(&old_sub)
    .execute(db.pool.as_ref())
    .await
    .expect("age the legacy mapping");

    let envelope = find_identity_envelope(&db.pool, &user)
        .await
        .expect("query succeeds")
        .expect("the user exists");

    assert_eq!(
        envelope.idp_issuer.as_deref(),
        Some(ISSUER),
        "the LATERAL join takes the newest mapping, and takes exactly one"
    );
    db.cleanup().await;
}
