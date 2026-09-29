//! Consent state and grants are isolated by principal and provider.

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;
use systemprompt_web_admin::repositories::users::connector_credentials::{
    self as repo, EncryptedGrant,
};

#[tokio::test]
async fn consent_is_single_use_and_bound_to_the_user_and_provider() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connector-alice");
    let bob = unique("connector-bob");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    let grant = EncryptedGrant {
        ciphertext: vec![7; 32],
        nonce: vec![1; 12],
    };
    repo::save_state(&db.pool, &user_id(&alice), "github", "test-consent", &grant)
        .await
        .unwrap();
    assert!(
        repo::consume_state(&db.pool, &user_id(&bob), "github", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "atlassian", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "github", "test-consent")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        repo::consume_state(&db.pool, &user_id(&alice), "github", "test-consent")
            .await
            .unwrap()
            .is_none()
    );
    let mut tx = db.pool.begin().await.unwrap();
    repo::store(&mut tx, &user_id(&alice), "github", &grant)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let mut tx = db.pool.begin().await.unwrap();
    assert!(
        repo::lock(&mut tx, &user_id(&bob), "github")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::lock(&mut tx, &user_id(&alice), "atlassian")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::lock(&mut tx, &user_id(&alice), "github")
            .await
            .unwrap()
            .is_some()
    );
    tx.commit().await.unwrap();
    db.cleanup().await;
}

#[tokio::test]
async fn account_revisions_survive_disconnect_and_are_isolated_per_user() {
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connection-state-alice");
    let bob = unique("connection-state-bob");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    let user = user_id(&alice);
    let mut tx = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut tx, &user, "github")
        .await
        .unwrap();
    let initial = row.revision;
    row.status = "connected".into();
    row.account_id = Some("verified-github-id".into());
    accounts::update_account(&mut tx, &user, &row)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let connected = accounts::list_accounts(&db.pool, &user).await.unwrap();
    assert!(connected[0].revision > initial);
    assert!(
        accounts::list_accounts(&db.pool, &user_id(&bob))
            .await
            .unwrap()
            .is_empty()
    );
    let mut tx = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut tx, &user, "github")
        .await
        .unwrap();
    row.status = "not_connected".into();
    row.generation += 1;
    accounts::update_account(&mut tx, &user, &row)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    let disconnected = accounts::list_accounts(&db.pool, &user).await.unwrap();
    assert_eq!(disconnected[0].generation, 1);
    assert!(disconnected[0].revision > connected[0].revision);
    db.cleanup().await;
}

#[tokio::test]
async fn refresh_and_disconnect_serialize_on_the_account_generation() {
    use std::time::Duration;
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connection-lock-alice");
    insert_user(&db.pool, &alice).await;
    let user = user_id(&alice);
    let mut first = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut first, &user, "atlassian")
        .await
        .unwrap();
    row.generation += 1;
    accounts::update_account(&mut first, &user, &row)
        .await
        .unwrap();
    let mut second = db.pool.begin().await.unwrap();
    assert!(
        tokio::time::timeout(
            Duration::from_millis(50),
            accounts::get_locked_account(&mut second, &user, "atlassian")
        )
        .await
        .is_err()
    );
    first.commit().await.unwrap();
    // Cancellation may abort the waiting PostgreSQL statement; start a fresh
    // transaction to prove the committed generation is what a callback sees.
    second.rollback().await.unwrap();
    let mut second = db.pool.begin().await.unwrap();
    let observed = accounts::get_locked_account(&mut second, &user, "atlassian")
        .await
        .unwrap();
    assert_eq!(observed.generation, 1);
    second.commit().await.unwrap();
    db.cleanup().await;
}

#[tokio::test]
async fn connector_login_ignores_analytics_end_but_rejects_revocation_and_expiry() {
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("connector-session");
    let bob = unique("connector-other");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    sqlx::query("INSERT INTO user_sessions (session_id, user_id, ended_at, expires_at) VALUES ($1,$2,NOW(),NOW()+INTERVAL '1 hour')")
        .bind(&alice).bind(&alice).execute(&*db.pool).await.unwrap();
    assert!(
        accounts::is_live_session(&db.pool, &user_id(&alice), &alice)
            .await
            .unwrap()
    );
    assert!(
        !accounts::is_live_session(&db.pool, &user_id(&bob), &alice)
            .await
            .unwrap()
    );
    sqlx::query("UPDATE user_sessions SET revoked_at=NOW() WHERE session_id=$1")
        .bind(&alice)
        .execute(&*db.pool)
        .await
        .unwrap();
    assert!(
        !accounts::is_live_session(&db.pool, &user_id(&alice), &alice)
            .await
            .unwrap()
    );
    sqlx::query("UPDATE user_sessions SET revoked_at=NULL, expires_at=NOW()-INTERVAL '1 second' WHERE session_id=$1")
        .bind(&alice).execute(&*db.pool).await.unwrap();
    assert!(
        !accounts::is_live_session(&db.pool, &user_id(&alice), &alice)
            .await
            .unwrap()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn configured_provider_ids_preserve_accounts_and_single_use_consent() {
    use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
    let Some(db) = TempDb::create().await else {
        return;
    };
    let id = unique("generic-connector");
    insert_user(&db.pool, &id).await;
    let user = user_id(&id);
    let grant = EncryptedGrant {
        ciphertext: vec![7; 32],
        nonce: vec![1; 12],
    };
    let mut tx = db.pool.begin().await.unwrap();
    for provider in ["atlassian", "fourth-mcp"] {
        repo::store(&mut tx, &user, provider, &grant).await.unwrap();
        let mut account = accounts::get_locked_account(&mut tx, &user, provider)
            .await
            .unwrap();
        account.status = "connected".into();
        account.auth_method = Some("oauth".into());
        accounts::update_account(&mut tx, &user, &account)
            .await
            .unwrap();
    }
    tx.commit().await.unwrap();
    assert_eq!(
        accounts::list_accounts(&db.pool, &user)
            .await
            .unwrap()
            .len(),
        2
    );
    repo::save_state(&db.pool, &user, "fourth-mcp", "generic-state", &grant)
        .await
        .unwrap();
    assert!(
        repo::consume_state(&db.pool, &user, "github", "generic-state")
            .await
            .unwrap()
            .is_none()
    );
    assert!(
        repo::consume_state(&db.pool, &user, "fourth-mcp", "generic-state")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        repo::consume_state(&db.pool, &user, "fourth-mcp", "generic-state")
            .await
            .unwrap()
            .is_none()
    );
    db.cleanup().await;
}

#[tokio::test]
async fn active_account_scope_uses_current_roles_and_rejects_inactive_accounts() {
    use systemprompt_security::policy::types::AccessScope;
    use systemprompt_web_admin::authz::account_scope;
    let Some(db) = TempDb::create().await else {
        return;
    };
    for role in [
        "admin",
        "platform_admin",
        "developer",
        "project_manager",
        "knowledge_worker",
        "custom-role",
    ] {
        let id = unique("account-scope");
        crate::fixtures::insert_user_with_roles(&db.pool, &id, &[role.into()]).await;
        let expected = if ["admin", "platform_admin"].contains(&role) {
            AccessScope::Admin
        } else {
            AccessScope::User
        };
        assert_eq!(
            account_scope(&db.pool, &user_id(&id)).await.unwrap(),
            expected
        );
        sqlx::query("UPDATE users SET roles = ARRAY[]::TEXT[] WHERE id = $1")
            .bind(&id)
            .execute(&*db.pool)
            .await
            .unwrap();
        assert_eq!(
            account_scope(&db.pool, &user_id(&id)).await.unwrap(),
            AccessScope::User
        );
        sqlx::query("UPDATE users SET status = 'inactive' WHERE id = $1")
            .bind(&id)
            .execute(&*db.pool)
            .await
            .unwrap();
        assert_eq!(
            account_scope(&db.pool, &user_id(&id)).await.unwrap(),
            AccessScope::Unknown
        );
    }
    assert!(
        account_scope(&db.pool, &user_id(&unique("missing")))
            .await
            .is_err()
    );
    db.cleanup().await;
}
