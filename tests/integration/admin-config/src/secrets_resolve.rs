//! `repositories::secrets` — the audit trail, one-shot resolution tokens, and
//! the plugin-facing secret read.

use systemprompt_web_admin::repositories::secrets::secret_audit::{
    insert_audit_entry, list_audit_log,
};
use systemprompt_web_admin::repositories::secrets::secret_crypto::{encrypt, generate_nonce};
use systemprompt_web_admin::repositories::secrets::secret_keys::get_or_create_user_dek;
use systemprompt_web_admin::repositories::secrets::secret_resolve::{
    create_resolution_token, resolve_secrets_for_plugin, validate_and_consume_token,
};

use crate::fixtures::{insert_secret_env_var, insert_user, unique, user_id};
use crate::tempdb::TempDb;

const MASTER_KEY: [u8; 32] = [7u8; 32];

// Stores `value` as a sealed secret the way the handler path would.
async fn store_secret(pool: &sqlx::PgPool, user: &str, plugin: &str, name: &str, value: &str) {
    let dek = get_or_create_user_dek(pool, &user_id(user), &MASTER_KEY)
        .await
        .expect("issue dek");
    let nonce = generate_nonce();
    let sealed = encrypt(&dek, &nonce, value.as_bytes()).expect("seal value");
    let id = insert_secret_env_var(pool, user, plugin, name, "").await;
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
async fn insert_audit_entry_then_list_audit_log_round_trips() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    insert_audit_entry(&db.pool, &user_id(&user), "plug", "created")
        .await
        .expect("write audit entry");
    insert_audit_entry(&db.pool, &user_id(&user), "plug", "deleted")
        .await
        .expect("write second audit entry");

    let rows = list_audit_log(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list audit log");

    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|r| r.actor_id == user));
    assert!(rows.iter().all(|r| r.var_name == "*"));

    db.cleanup().await;
}

#[tokio::test]
async fn list_audit_log_is_scoped_to_one_plugin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_audit_entry(&db.pool, &user_id(&user), "plug-a", "created")
        .await
        .expect("write audit entry");

    let rows = list_audit_log(&db.pool, &user_id(&user), "plug-b")
        .await
        .expect("list audit log");

    assert!(rows.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn insert_audit_entry_rejects_an_unknown_action() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let result = insert_audit_entry(&db.pool, &user_id(&user), "plug", "exfiltrated").await;

    assert!(
        result.is_err(),
        "the audit vocabulary is a database constraint, not a convention"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn create_resolution_token_can_be_consumed_once() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let token = create_resolution_token(&db.pool, &user_id(&user), "plug")
        .await
        .expect("create token");
    let (resolved_user, resolved_plugin) = validate_and_consume_token(&db.pool, &token)
        .await
        .expect("consume token");
    let replay = validate_and_consume_token(&db.pool, &token).await;

    assert_eq!(resolved_user, user);
    assert_eq!(resolved_plugin, "plug");
    assert!(replay.is_err(), "a one-shot token must not be reusable");

    db.cleanup().await;
}

#[tokio::test]
async fn create_resolution_token_stores_only_the_hash() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let token = create_resolution_token(&db.pool, &user_id(&user), "plug")
        .await
        .expect("create token");

    let stored = sqlx::query_scalar::<_, String>(
        "SELECT token_hash FROM secret_resolution_tokens WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("read stored token");
    assert_ne!(stored, token, "a leaked row must not yield a usable token");

    db.cleanup().await;
}

#[tokio::test]
async fn validate_and_consume_token_rejects_an_expired_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let token = create_resolution_token(&db.pool, &user_id(&user), "plug")
        .await
        .expect("create token");
    sqlx::query("UPDATE secret_resolution_tokens SET expires_at = NOW() - INTERVAL '1 minute' WHERE user_id = $1")
        .bind(&user)
        .execute(&*db.pool)
        .await
        .expect("expire the token");

    let result = validate_and_consume_token(&db.pool, &token).await;

    assert!(result.is_err());

    db.cleanup().await;
}

#[tokio::test]
async fn validate_and_consume_token_rejects_an_unknown_token() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = validate_and_consume_token(&db.pool, "not-a-token").await;

    assert!(result.is_err());

    db.cleanup().await;
}

#[tokio::test]
async fn resolve_secrets_for_plugin_returns_only_that_plugins_secrets() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    store_secret(&db.pool, &user, "plug-a", "A_TOKEN", "alpha").await;
    store_secret(&db.pool, &user, "plug-b", "B_TOKEN", "beta").await;

    let resolved = resolve_secrets_for_plugin(&db.pool, &user_id(&user), "plug-a", &MASTER_KEY)
        .await
        .expect("resolve secrets");

    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved.get("A_TOKEN").map(String::as_str), Some("alpha"));

    db.cleanup().await;
}

#[tokio::test]
async fn resolve_secrets_for_plugin_audits_the_access() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    store_secret(&db.pool, &user, "plug", "A_TOKEN", "alpha").await;

    resolve_secrets_for_plugin(&db.pool, &user_id(&user), "plug", &MASTER_KEY)
        .await
        .expect("resolve secrets");

    let actions = sqlx::query_scalar::<_, String>(
        "SELECT action FROM secret_audit_log WHERE user_id = $1 AND plugin_id = 'plug'",
    )
    .bind(&user)
    .fetch_all(&*db.pool)
    .await
    .expect("read audit trail");
    assert_eq!(actions, vec!["accessed".to_owned()]);

    db.cleanup().await;
}

#[tokio::test]
async fn resolve_secrets_for_plugin_skips_unsealed_rows() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_secret_env_var(&db.pool, &user, "plug", "LEGACY", "plaintext").await;

    let resolved = resolve_secrets_for_plugin(&db.pool, &user_id(&user), "plug", &MASTER_KEY)
        .await
        .expect("resolve secrets");

    assert!(
        resolved.is_empty(),
        "a row still awaiting migration has no sealed value to return"
    );

    db.cleanup().await;
}
