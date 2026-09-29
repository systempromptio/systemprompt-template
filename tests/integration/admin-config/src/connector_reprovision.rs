//! Reprovisioning a connector resets every account on that provider and no
//! other.

use crate::fixtures::{insert_user, unique, user_id};
use crate::tempdb::TempDb;
use systemprompt_web_admin::repositories::users::connector_accounts as accounts;
use systemprompt_web_admin::repositories::users::connector_credentials::{
    self as credentials, EncryptedGrant,
};

async fn connect(db: &TempDb, user: &str, provider: &str) -> i64 {
    let user = user_id(user);
    let mut tx = db.pool.begin().await.unwrap();
    let mut row = accounts::get_locked_account(&mut tx, &user, provider)
        .await
        .unwrap();
    row.status = "connected".into();
    row.auth_method = Some("oauth".into());
    row.verified_at = Some(chrono::Utc::now());
    accounts::update_account(&mut tx, &user, &row)
        .await
        .unwrap();
    credentials::store(
        &mut tx,
        &user,
        provider,
        &EncryptedGrant {
            ciphertext: vec![9; 16],
            nonce: vec![2; 12],
        },
    )
    .await
    .unwrap();
    tx.commit().await.unwrap();
    row.generation
}

async fn account(db: &TempDb, user: &str, provider: &str) -> accounts::ProviderConnection {
    let user = user_id(user);
    let mut tx = db.pool.begin().await.unwrap();
    let row = accounts::get_locked_account(&mut tx, &user, provider)
        .await
        .unwrap();
    tx.commit().await.unwrap();
    row
}

#[tokio::test]
async fn reprovision_resets_only_the_named_provider() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let alice = unique("reprov-alice");
    let bob = unique("reprov-bob");
    insert_user(&db.pool, &alice).await;
    insert_user(&db.pool, &bob).await;
    let before = connect(&db, &alice, "acme-tools").await;
    connect(&db, &bob, "acme-tools").await;
    connect(&db, &alice, "github").await;

    let mut tx = db.pool.begin().await.unwrap();
    let reset = accounts::reprovision_provider(&mut tx, "acme-tools", "provider_reprovisioned")
        .await
        .unwrap();
    let deleted = credentials::delete_all_for_provider(&mut tx, "acme-tools")
        .await
        .unwrap();
    tx.commit().await.unwrap();
    assert_eq!(reset, 2);
    assert_eq!(deleted, 2);

    for user in [&alice, &bob] {
        let row = account(&db, user, "acme-tools").await;
        assert_eq!(row.status, "reconnect_required");
        assert_eq!(row.error_code.as_deref(), Some("provider_reprovisioned"));
        assert!(row.verified_at.is_none());
        assert_eq!(row.generation, before + 1);
    }
    let untouched = account(&db, &alice, "github").await;
    assert_eq!(untouched.status, "connected");
    assert!(untouched.verified_at.is_some());
    let mut tx = db.pool.begin().await.unwrap();
    assert!(
        credentials::lock(&mut tx, &user_id(&alice), "github")
            .await
            .unwrap()
            .is_some()
    );
    assert!(
        credentials::lock(&mut tx, &user_id(&alice), "acme-tools")
            .await
            .unwrap()
            .is_none()
    );
}
