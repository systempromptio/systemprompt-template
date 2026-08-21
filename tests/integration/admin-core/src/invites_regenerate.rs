//! `repositories::users::invites::insert_regenerated_invite` — the recovery
//! path for an invite link that was shown once and lost.
//!
//! The property under test is atomicity: after a regenerate there is exactly
//! one live invite for the email, the old token no longer resolves, and the
//! new one carries the original org, department, and roles.

use systemprompt_web_admin::repositories::organizations::budget_warnings as _;
use systemprompt_web_admin::repositories::users::invites;

use crate::fixtures::{OrgSpec, insert_org, insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

struct Fixture {
    org_id: String,
    email: String,
    token_hash: String,
    invite_id: String,
}

async fn mint(db: &TempDb) -> Fixture {
    let org_id = unique("org");
    insert_org(&db.pool, &OrgSpec::active(&org_id, &org_id)).await;
    let admin = insert_user(&db.pool, &unique("admin"), &unclaimed_email("inviter")).await;
    let email = unclaimed_email("regen");
    let token_hash = unique("hash");
    let invite_id = invites::insert_invite(
        &db.pool,
        &invites::NewInvite {
            email: &email,
            token_hash: &token_hash,
            org_id: &org_id,
            department: "Engineering",
            roles: &["user".to_owned()],
            invited_by: &admin,
            expires_at: chrono::Utc::now() + chrono::Duration::days(7),
        },
    )
    .await
    .expect("mint invite");
    Fixture {
        org_id,
        email,
        token_hash,
        invite_id,
    }
}

#[tokio::test]
async fn regenerating_retires_the_old_token_and_issues_a_live_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let f = mint(&db).await;
    let new_hash = unique("hash-new");

    let new_id = invites::insert_regenerated_invite(
        &db.pool,
        &f.invite_id,
        None,
        &new_hash,
        chrono::Utc::now() + chrono::Duration::days(7),
    )
    .await
    .expect("regenerate succeeds")
    .expect("a pending invite matched");
    assert_ne!(new_id, f.invite_id, "a fresh row, not an update in place");

    assert!(
        invites::find_valid_invite_by_hash(&db.pool, &f.token_hash)
            .await
            .expect("lookup")
            .is_none(),
        "the old token must stop resolving"
    );
    let fresh = invites::find_valid_invite_by_hash(&db.pool, &new_hash)
        .await
        .expect("lookup")
        .expect("the new token resolves");
    assert_eq!(fresh.email, f.email);
    assert_eq!(fresh.org_id, f.org_id);
    assert_eq!(fresh.department, "Engineering", "carries the original department");
    assert_eq!(fresh.roles, vec!["user".to_owned()], "carries the original roles");

    let pending = invites::list_pending_invites(&db.pool, Some(&f.org_id))
        .await
        .expect("list succeeds");
    assert_eq!(
        pending.iter().filter(|p| p.email == f.email).count(),
        1,
        "exactly one live invite per email survives"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn regenerating_another_organizations_invite_matches_nothing() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let f = mint(&db).await;
    let other_org = unique("org");
    insert_org(&db.pool, &OrgSpec::active(&other_org, &other_org)).await;

    let result = invites::insert_regenerated_invite(
        &db.pool,
        &f.invite_id,
        Some(&other_org),
        &unique("hash-x"),
        chrono::Utc::now() + chrono::Duration::days(7),
    )
    .await
    .expect("query succeeds");
    assert!(result.is_none(), "org scoping must not match across tenants");

    assert!(
        invites::find_valid_invite_by_hash(&db.pool, &f.token_hash)
            .await
            .expect("lookup")
            .is_some(),
        "a refused regenerate must leave the original live"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn an_accepted_invite_cannot_be_regenerated() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let f = mint(&db).await;
    let invite = invites::find_valid_invite_by_hash(&db.pool, &f.token_hash)
        .await
        .expect("lookup")
        .expect("invite is live");
    invites::accept_invite_and_provision(&db.pool, &invite)
        .await
        .expect("accept provisions");

    let result = invites::insert_regenerated_invite(
        &db.pool,
        &f.invite_id,
        None,
        &unique("hash-y"),
        chrono::Utc::now() + chrono::Duration::days(7),
    )
    .await
    .expect("query succeeds");
    assert!(result.is_none(), "a consumed invite is not a pending one");
    db.cleanup().await;
}
