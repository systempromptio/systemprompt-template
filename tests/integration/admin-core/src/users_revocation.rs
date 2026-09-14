//! `repositories::users::revocation` — the teardown that runs when Active
//! Directory stops vouching for someone.
//!
//! The point of these tests is that a refused sign-in reaches *backwards*: a
//! cookie session and a bridge PAT both outlive the assertion that created
//! them, so a group removal that only blocks the next login has revoked
//! nothing.

use systemprompt_web_admin::repositories::bridge::{issue_api_key, issue_exchange_code};
use systemprompt_web_admin::repositories::users::revocation::{
    revoke_access_by_email, revoke_user_access,
};

use crate::fixtures::{insert_session, insert_user, insert_user_full, unclaimed_email, unique};
use crate::tempdb::TempDb;

async fn live_api_keys(pool: &sqlx::PgPool, user: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_api_keys WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .expect("count live keys")
}

async fn live_sessions(pool: &sqlx::PgPool, user: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT COUNT(*) FROM user_sessions WHERE user_id = $1 AND revoked_at IS NULL",
    )
    .bind(user)
    .fetch_one(pool)
    .await
    .expect("count live sessions")
}

#[tokio::test]
async fn revoke_user_access_kills_the_session_and_the_bridge_pat() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("revoked")).await;
    insert_session(&db.pool, &unique("sess"), &user).await;
    issue_api_key(&db.pool, &user, "bridge device-link", None)
        .await
        .expect("issue pat");
    issue_exchange_code(&db.pool, &user)
        .await
        .expect("issue exchange code");

    let counts = revoke_user_access(&db.pool, &user)
        .await
        .expect("revocation succeeds");

    assert_eq!(counts.sessions, 1, "the cookie session is revoked");
    assert_eq!(counts.api_keys, 1, "the bridge PAT is revoked");
    assert_eq!(
        counts.exchange_codes, 1,
        "an unredeemed device-link code is burned, not left to mint a fresh PAT"
    );
    assert_eq!(live_sessions(&db.pool, user.as_str()).await, 0);
    assert_eq!(live_api_keys(&db.pool, user.as_str()).await, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn revoke_user_access_is_idempotent() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = insert_user(&db.pool, &unique("user"), &unclaimed_email("twice")).await;
    insert_session(&db.pool, &unique("sess"), &user).await;

    revoke_user_access(&db.pool, &user).await.expect("first");
    let second = revoke_user_access(&db.pool, &user).await.expect("second");

    assert!(
        second.is_empty(),
        "a second pass moves nothing — revoked_at is set once and kept as the audit trail"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn revoke_user_access_leaves_other_users_alone() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let denied = insert_user(&db.pool, &unique("denied"), &unclaimed_email("denied")).await;
    let bystander = insert_user(&db.pool, &unique("other"), &unclaimed_email("other")).await;
    insert_session(&db.pool, &unique("sess"), &denied).await;
    insert_session(&db.pool, &unique("sess"), &bystander).await;

    revoke_user_access(&db.pool, &denied)
        .await
        .expect("revocation succeeds");

    assert_eq!(live_sessions(&db.pool, bystander.as_str()).await, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn revoke_access_by_email_finds_the_account_the_gate_refused() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("nogroup");
    let user = insert_user(&db.pool, &unique("user"), &email).await;
    insert_session(&db.pool, &unique("sess"), &user).await;

    let (found, counts) = revoke_access_by_email(&db.pool, &email.to_uppercase())
        .await
        .expect("lookup succeeds")
        .expect("the address resolves to an active account");

    assert_eq!(found, user, "matched case-insensitively, as AD asserts it");
    assert_eq!(counts.sessions, 1);
    db.cleanup().await;
}

#[tokio::test]
async fn revoke_access_by_email_ignores_an_unknown_or_closed_account() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let closed = unclaimed_email("closed");
    insert_user_full(
        &db.pool,
        &unique("user"),
        &closed,
        None,
        &["user".to_owned()],
        "suspended",
    )
    .await;

    assert!(
        revoke_access_by_email(&db.pool, &unclaimed_email("ghost"))
            .await
            .expect("lookup succeeds")
            .is_none(),
        "an address with no account is not an error"
    );
    assert!(
        revoke_access_by_email(&db.pool, &closed)
            .await
            .expect("lookup succeeds")
            .is_none(),
        "a closed account's credentials were taken when it closed"
    );
    db.cleanup().await;
}
