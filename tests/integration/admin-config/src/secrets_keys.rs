//! `repositories::secrets::secret_keys` — per-user data encryption key issue
//! and rotation.
//!
//! The master key is supplied directly rather than read from the environment,
//! so these tests exercise the storage paths without mutating process state.

use systemprompt_web_admin::repositories::secrets::secret_crypto::{encrypt, generate_nonce};
use systemprompt_web_admin::repositories::secrets::secret_keys::{
    get_or_create_user_dek, rotate_user_dek,
};
use systemprompt_web_admin::repositories::secrets::secret_migration::get_key_version;
use systemprompt_web_admin::repositories::secrets::secret_resolve::resolve_secrets_for_plugin;

use crate::fixtures::{insert_env_var, insert_user, unique, user_id};
use crate::tempdb::TempDb;

const MASTER_KEY: [u8; 32] = [7u8; 32];

// Stores `value` as a sealed secret the way the handler path would.
async fn store_secret(pool: &sqlx::PgPool, user: &str, plugin: &str, name: &str, value: &str) {
    let dek = get_or_create_user_dek(pool, &user_id(user), &MASTER_KEY)
        .await
        .expect("issue dek");
    let nonce = generate_nonce();
    let sealed = encrypt(&dek, &nonce, value.as_bytes()).expect("seal value");
    let id = insert_env_var(pool, user, plugin, name, "", true).await;
    sqlx::query(
        "UPDATE plugin_env_vars SET encrypted_value = $1, value_nonce = $2, key_version = 1
         WHERE id = $3",
    )
    .bind(sealed)
    .bind(nonce.to_vec())
    .bind(&id)
    .execute(pool)
    .await
    .expect("store sealed value");
}

#[tokio::test]
async fn get_or_create_user_dek_issues_one_key_and_reuses_it() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let first = get_or_create_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("issue dek");
    let second = get_or_create_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("read dek back");

    assert_eq!(first, second, "the stored key must round-trip");
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_encryption_keys WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("count keys");
    assert_eq!(count, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn get_or_create_user_dek_issues_distinct_keys_per_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let one = unique("u");
    let two = unique("u");
    insert_user(&db.pool, &one).await;
    insert_user(&db.pool, &two).await;

    let key_one = get_or_create_user_dek(&db.pool, &user_id(&one), &MASTER_KEY)
        .await
        .expect("issue first dek");
    let key_two = get_or_create_user_dek(&db.pool, &user_id(&two), &MASTER_KEY)
        .await
        .expect("issue second dek");

    assert_ne!(key_one, key_two);

    db.cleanup().await;
}

#[tokio::test]
async fn get_or_create_user_dek_fails_under_the_wrong_master_key() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    get_or_create_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("issue dek");

    let result = get_or_create_user_dek(&db.pool, &user_id(&user), &[9u8; 32]).await;

    assert!(
        result.is_err(),
        "a key sealed under one master key must not open under another"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn rotate_user_dek_re_seals_every_secret_under_the_new_key() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    store_secret(&db.pool, &user, "plug", "API_TOKEN", "s3cret").await;

    rotate_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("rotate dek");

    let resolved = resolve_secrets_for_plugin(&db.pool, &user_id(&user), "plug", &MASTER_KEY)
        .await
        .expect("resolve after rotation");
    assert_eq!(
        resolved.get("API_TOKEN").map(String::as_str),
        Some("s3cret")
    );

    db.cleanup().await;
}

#[tokio::test]
async fn rotate_user_dek_bumps_the_key_version() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    store_secret(&db.pool, &user, "plug", "API_TOKEN", "s3cret").await;

    rotate_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("rotate dek");

    assert_eq!(get_key_version(&db.pool, &user_id(&user)).await, 2);
    let secret_version = sqlx::query_scalar::<_, i32>(
        "SELECT key_version FROM plugin_env_vars WHERE user_id = $1 AND var_name = 'API_TOKEN'",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("read secret version");
    assert_eq!(secret_version, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn rotate_user_dek_issues_a_key_for_a_user_who_never_had_one() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    rotate_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("rotate without prior key");

    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM user_encryption_keys WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("count keys");
    assert_eq!(count, 1);

    db.cleanup().await;
}
