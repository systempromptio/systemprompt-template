//! `repositories::marketplace::plugin_env` — per-plugin environment records.

use systemprompt_web_admin::repositories::marketplace::plugin_env::list_plugin_env_vars;

use crate::fixtures::{insert_env_var, insert_secret_env_var, insert_user, unique, user_id};
use crate::tempdb::TempDb;

const MASK: &str = "••••••••";

#[tokio::test]
async fn list_plugin_env_vars_is_empty_when_nothing_is_configured() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list env vars");

    assert!(rows.is_empty());

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_masks_secret_values() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_secret_env_var(&db.pool, &user, "plug", "API_TOKEN", "s3cret").await;
    insert_env_var(&db.pool, &user, "plug", "BASE_URL", "https://x.test").await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list env vars");

    assert_eq!(rows.len(), 2, "ordered by name");
    assert_eq!(rows[0].var_name, "API_TOKEN");
    assert_eq!(rows[0].var_value, MASK, "a secret never leaves the process");
    assert_eq!(rows[1].var_value, "https://x.test");

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_leaves_an_empty_secret_unmasked() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_secret_env_var(&db.pool, &user, "plug", "API_TOKEN", "").await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list env vars");

    assert_eq!(
        rows[0].var_value, "",
        "masking an unset secret would read as configured"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_falls_back_to_the_admin_defaults() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_env_var(
        &db.pool,
        "admin",
        "plug",
        "BASE_URL",
        "https://default.test",
    )
    .await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list env vars");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].var_value, "https://default.test");

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_prefers_the_users_own_values() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_env_var(
        &db.pool,
        "admin",
        "plug",
        "BASE_URL",
        "https://default.test",
    )
    .await;
    insert_env_var(&db.pool, &user, "plug", "BASE_URL", "https://mine.test").await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug")
        .await
        .expect("list env vars");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].var_value, "https://mine.test");

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_is_scoped_to_one_plugin() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    insert_env_var(&db.pool, &user, "plug-a", "A", "alpha").await;
    insert_env_var(&db.pool, &user, "plug-b", "B", "beta").await;

    let rows = list_plugin_env_vars(&db.pool, &user_id(&user), "plug-a")
        .await
        .expect("list env vars");

    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].var_name, "A");
    assert_eq!(rows[0].plugin_id.as_str(), "plug-a");

    db.cleanup().await;
}

#[tokio::test]
async fn list_plugin_env_vars_for_admin_does_not_recurse_into_the_fallback() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let rows = list_plugin_env_vars(&db.pool, &user_id("admin"), "plug")
        .await
        .expect("list env vars");

    assert!(rows.is_empty(), "admin is the fallback, so it has none");

    db.cleanup().await;
}
