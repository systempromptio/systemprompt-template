//! `repositories::bridge::api_keys` — personal access token issue, listing,
//! and revocation.

use chrono::{Duration, Utc};
use systemprompt_web_admin::repositories::bridge::api_keys::API_KEY_PREFIX;
use systemprompt_web_admin::repositories::bridge::{
    BridgeRepoError, issue_api_key, list_api_keys_for_user, revoke_api_key,
};

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;

#[tokio::test]
async fn issue_api_key_returns_a_secret_that_extends_its_prefix() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let issued = issue_api_key(&db.pool, &user_id(&user), "laptop", None)
        .await
        .expect("issue key");

    assert!(issued.key_prefix.starts_with(API_KEY_PREFIX));
    assert!(issued.secret.starts_with(&issued.key_prefix));
    assert!(issued.created_at.is_some());
    assert!(issued.expires_at.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn issue_api_key_stores_only_a_hash_of_the_secret() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let issued = issue_api_key(&db.pool, &user_id(&user), "laptop", None)
        .await
        .expect("issue key");

    let hash = sqlx::query_scalar::<_, String>("SELECT key_hash FROM user_api_keys WHERE id = $1")
        .bind(&issued.id)
        .fetch_one(&*db.pool)
        .await
        .expect("read stored hash");
    assert_ne!(hash, issued.secret);
    assert!(!hash.contains(&issued.secret));

    db.cleanup().await;
}

#[tokio::test]
async fn issue_api_key_trims_the_name_and_rejects_a_blank_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let issued = issue_api_key(&db.pool, &user_id(&user), "  laptop  ", None)
        .await
        .expect("issue key");
    let blank = issue_api_key(&db.pool, &user_id(&user), "   ", None).await;

    assert_eq!(issued.name, "laptop");
    assert!(matches!(blank, Err(BridgeRepoError::Validation(_))));

    db.cleanup().await;
}

#[tokio::test]
async fn issue_api_key_records_an_expiry_when_one_is_given() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let expires = Utc::now() + Duration::days(30);

    let issued = issue_api_key(&db.pool, &user_id(&user), "laptop", Some(expires))
        .await
        .expect("issue key");

    assert!(issued.expires_at.is_some());

    db.cleanup().await;
}

#[tokio::test]
async fn issue_api_key_fails_for_a_user_who_does_not_exist() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = issue_api_key(&db.pool, &user_id(&unique("ghost")), "laptop", None).await;

    assert!(matches!(result, Err(BridgeRepoError::Database(_))));

    db.cleanup().await;
}

#[tokio::test]
async fn issued_keys_are_distinct_across_calls() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let first = issue_api_key(&db.pool, &user_id(&user), "one", None)
        .await
        .expect("issue first key");
    let second = issue_api_key(&db.pool, &user_id(&user), "two", None)
        .await
        .expect("issue second key");

    assert_ne!(first.id, second.id);
    assert_ne!(first.key_prefix, second.key_prefix);
    assert_ne!(first.secret, second.secret);

    db.cleanup().await;
}

#[tokio::test]
async fn list_api_keys_for_user_is_empty_before_any_are_issued() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let rows = list_api_keys_for_user(&db.pool, &user_id(&user))
        .await
        .expect("list keys");

    assert!(rows.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn list_api_keys_for_user_is_scoped_to_that_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let mine = unique("u");
    let theirs = unique("u");
    insert_user(&db.pool, &mine).await;
    insert_user(&db.pool, &theirs).await;
    issue_api_key(&db.pool, &user_id(&theirs), "theirs", None)
        .await
        .expect("issue their key");
    let own = issue_api_key(&db.pool, &user_id(&mine), "mine", None)
        .await
        .expect("issue my key");

    let rows = list_api_keys_for_user(&db.pool, &user_id(&mine))
        .await
        .expect("list keys");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].id, own.id);
    assert!(rows[0].revoked_at.is_none());
    assert!(rows[0].last_used_at.is_none());

    db.cleanup().await;
}

#[tokio::test]
async fn revoke_api_key_reports_whether_it_changed_anything() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let issued = issue_api_key(&db.pool, &user_id(&user), "laptop", None)
        .await
        .expect("issue key");

    let first = revoke_api_key(&db.pool, &user_id(&user), &issued.id)
        .await
        .expect("revoke");
    let second = revoke_api_key(&db.pool, &user_id(&user), &issued.id)
        .await
        .expect("revoke again");

    assert!(first);
    assert!(!second, "an already-revoked key is not revoked twice");
    let rows = list_api_keys_for_user(&db.pool, &user_id(&user))
        .await
        .expect("list keys");
    assert!(rows[0].revoked_at.is_some());

    db.cleanup().await;
}

#[tokio::test]
async fn revoke_api_key_will_not_revoke_someone_elses_key() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = unique("u");
    let stranger = unique("u");
    insert_user(&db.pool, &owner).await;
    insert_user(&db.pool, &stranger).await;
    let issued = issue_api_key(&db.pool, &user_id(&owner), "laptop", None)
        .await
        .expect("issue key");

    let revoked = revoke_api_key(&db.pool, &user_id(&stranger), &issued.id)
        .await
        .expect("attempt revoke");

    assert!(!revoked, "the key id alone must not authorise a revoke");

    db.cleanup().await;
}
