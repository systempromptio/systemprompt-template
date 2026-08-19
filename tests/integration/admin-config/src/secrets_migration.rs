//! `repositories::secrets::secret_migration` — re-encryption of legacy
//! plaintext secrets, driven by the migration job.

use systemprompt_web_admin::repositories::secrets::secret_crypto::{encrypt, generate_nonce};
use systemprompt_web_admin::repositories::secrets::secret_keys::get_or_create_user_dek;
use systemprompt_web_admin::repositories::secrets::secret_migration::{
    get_key_version, insert_migration_audit, list_unencrypted_secrets, update_encrypted_value,
};

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
async fn get_key_version_defaults_to_one_for_an_unknown_user() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let version = get_key_version(&db.pool, &user_id(&unique("nobody"))).await;

    assert_eq!(version, 1, "a missing row reads as the first key version");

    db.cleanup().await;
}

#[tokio::test]
async fn list_unencrypted_secrets_finds_legacy_plaintext_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_env_var(&db.pool, &user, "plug", "LEGACY", "plaintext", true).await;

    let rows = list_unencrypted_secrets(&db.pool)
        .await
        .expect("list unencrypted");

    let row = rows
        .iter()
        .find(|r| r.user_id.as_str() == user)
        .expect("the legacy row is listed");
    assert_eq!(row.var_name, "LEGACY");
    assert_eq!(row.var_value, "plaintext");

    db.cleanup().await;
}

#[tokio::test]
async fn list_unencrypted_secrets_skips_non_secret_and_empty_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_env_var(&db.pool, &user, "plug", "PUBLIC", "visible", false).await;
    insert_env_var(&db.pool, &user, "plug", "BLANK", "", true).await;

    let rows = list_unencrypted_secrets(&db.pool)
        .await
        .expect("list unencrypted");

    assert!(!rows.iter().any(|r| r.user_id.as_str() == user));

    db.cleanup().await;
}

#[tokio::test]
async fn list_unencrypted_secrets_skips_rows_already_sealed() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    store_secret(&db.pool, &user, "plug", "SEALED", "value").await;

    let rows = list_unencrypted_secrets(&db.pool)
        .await
        .expect("list unencrypted");

    assert!(!rows.iter().any(|r| r.user_id.as_str() == user));

    db.cleanup().await;
}

#[tokio::test]
async fn update_encrypted_value_clears_the_plaintext_column() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let id = insert_env_var(&db.pool, &user, "plug", "LEGACY", "plaintext", true).await;
    let dek = get_or_create_user_dek(&db.pool, &user_id(&user), &MASTER_KEY)
        .await
        .expect("issue dek");
    let nonce = generate_nonce();
    let sealed = encrypt(&dek, &nonce, b"plaintext").expect("seal value");

    update_encrypted_value(&db.pool, &id, &sealed, &nonce[..], 1)
        .await
        .expect("store sealed value");

    let row = sqlx::query_as::<_, (String, i32)>(
        "SELECT var_value, key_version FROM plugin_env_vars WHERE id = $1",
    )
    .bind(&id)
    .fetch_one(&*db.pool)
    .await
    .expect("read row back");
    assert_eq!(row.0, "", "the plaintext must not survive the migration");
    assert_eq!(row.1, 1);

    db.cleanup().await;
}

#[tokio::test]
async fn insert_migration_audit_records_who_re_encrypted_what() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    let actor = unique("admin");
    insert_user(&db.pool, &user).await;

    insert_migration_audit(&db.pool, &user_id(&user), "LEGACY", &user_id(&actor))
        .await
        .expect("write audit row");

    let row = sqlx::query_as::<_, (String, String, String)>(
        "SELECT action, actor_id, var_name FROM secret_audit_log WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("read audit row");
    assert_eq!(row.0, "updated");
    assert_eq!(row.1, actor);
    assert_eq!(row.2, "LEGACY");

    db.cleanup().await;
}
