//! `repositories::users::registration` — the self-registration setup tokens
//! that link a passkey to an existing account, and the rate-limit counter
//! guarding the door.
//!
//! `count_recent_setup_tokens` swallows its errors and reports `0`, so the
//! tests below pin that an unknown subject is indistinguishable from one with
//! no tokens: the rate limiter must never fail open by erroring.

use systemprompt::identifiers::UserId;
use systemprompt_web_admin::repositories::users::registration;

use crate::fixtures::{insert_user, unclaimed_email, unique};
use crate::tempdb::TempDb;

#[tokio::test]
async fn count_recent_setup_tokens_starts_at_zero() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("fresh");
    insert_user(&db.pool, &unique("user"), &email).await;

    let count = registration::count_recent_setup_tokens(&db.pool, &email).await;

    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn count_recent_setup_tokens_is_zero_for_an_unknown_email() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let count =
        registration::count_recent_setup_tokens(&db.pool, &unclaimed_email("stranger")).await;

    assert_eq!(count, 0);
    db.cleanup().await;
}

#[tokio::test]
async fn insert_setup_token_stores_a_credential_link_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("token");
    let user = insert_user(&db.pool, &unique("user"), &email).await;

    registration::insert_setup_token(&db.pool, &unique("tok"), &user, "hash-of-token")
        .await
        .expect("insert succeeds");

    let purpose: String =
        sqlx::query_scalar("SELECT purpose FROM webauthn_setup_tokens WHERE user_id = $1")
            .bind(user.as_str())
            .fetch_one(&*db.pool)
            .await
            .expect("the token row exists");
    assert_eq!(purpose, "credential_link");
    db.cleanup().await;
}

#[tokio::test]
async fn insert_setup_token_is_counted_by_the_rate_limiter() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let email = unclaimed_email("ratelimit");
    let user = insert_user(&db.pool, &unique("user"), &email).await;

    // `webauthn_setup_tokens.token_hash` is unique, so each mint needs its own.
    for _ in 0..3 {
        registration::insert_setup_token(&db.pool, &unique("tok"), &user, &unique("hash"))
            .await
            .expect("insert succeeds");
    }

    let count = registration::count_recent_setup_tokens(&db.pool, &email).await;

    assert_eq!(
        count, 3,
        "the counter is scoped by email, so every token this account minted counts"
    );
    db.cleanup().await;
}

#[tokio::test]
async fn insert_setup_token_requires_a_real_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let absent = UserId::new(unique("absent"));

    let err = registration::insert_setup_token(&db.pool, &unique("tok"), &absent, &unique("hash"))
        .await
        .expect_err("the user_id foreign key rejects an orphan token");

    assert!(
        err.to_string().contains("foreign key"),
        "unexpected failure mode: {err}"
    );
    db.cleanup().await;
}
