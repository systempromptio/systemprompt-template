//! `repositories::bridge` — device enrolment, the user lookup behind a bridge
//! credential, device certificates, and one-shot exchange codes.

use chrono::Utc;
use systemprompt_web_admin::repositories::bridge::{
    BridgeRepoError, EXCHANGE_CODE_TTL_SECONDS, EnrollDeviceParams, enroll_device,
    find_bridge_user, issue_exchange_code, list_api_keys_for_user, revoke_device_cert,
};

use crate::fixtures::{insert_user, insert_user_with_roles, unique, user_id};
use crate::tempdb::TempDb;

fn device_params<'a>(name: &'a str, platform: &'a str) -> EnrollDeviceParams<'a> {
    EnrollDeviceParams {
        name,
        platform,
        hostname: "  workstation  ",
        expires_at: None,
    }
}

async fn insert_device_cert(pool: &sqlx::PgPool, user: &str) -> String {
    let id = unique("cert");
    sqlx::query(
        "INSERT INTO user_device_certs (id, user_id, fingerprint, label)
         VALUES ($1, $2, $3, 'Laptop')",
    )
    .bind(&id)
    .bind(user)
    .bind(unique("fp"))
    .execute(pool)
    .await
    .expect("insert device cert");
    id
}

#[tokio::test]
async fn find_bridge_user_returns_none_for_an_unknown_id() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let found = find_bridge_user(&db.pool, &user_id(&unique("ghost")))
        .await
        .expect("look up user");

    assert!(found.is_none(), "find_ reports absence as None");

    db.cleanup().await;
}

#[tokio::test]
async fn find_bridge_user_carries_the_roles_the_bridge_scopes_on() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user_with_roles(&db.pool, &user, &["user".to_owned(), "admin".to_owned()]).await;

    let found = find_bridge_user(&db.pool, &user_id(&user))
        .await
        .expect("look up user")
        .expect("the user exists");

    assert_eq!(found.id, user);
    assert_eq!(found.email, format!("{user}@example.test"));
    assert_eq!(found.roles, vec!["user".to_owned(), "admin".to_owned()]);
    assert_eq!(found.display_name, Some(format!("User {user}")));

    db.cleanup().await;
}

#[tokio::test]
async fn revoke_device_cert_reports_whether_it_changed_anything() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;
    let cert = insert_device_cert(&db.pool, &user).await;

    let first = revoke_device_cert(&db.pool, &user_id(&user), &cert)
        .await
        .expect("revoke cert");
    let second = revoke_device_cert(&db.pool, &user_id(&user), &cert)
        .await
        .expect("revoke again");

    assert!(first);
    assert!(!second);

    db.cleanup().await;
}

#[tokio::test]
async fn revoke_device_cert_will_not_revoke_someone_elses_cert() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let owner = unique("u");
    let stranger = unique("u");
    insert_user(&db.pool, &owner).await;
    insert_user(&db.pool, &stranger).await;
    let cert = insert_device_cert(&db.pool, &owner).await;

    let revoked = revoke_device_cert(&db.pool, &user_id(&stranger), &cert)
        .await
        .expect("attempt revoke");

    assert!(!revoked);

    db.cleanup().await;
}

#[tokio::test]
async fn issue_exchange_code_stores_only_the_hash_and_a_short_expiry() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let issued = issue_exchange_code(&db.pool, &user_id(&user))
        .await
        .expect("issue exchange code");

    let stored = sqlx::query_scalar::<_, String>(
        "SELECT code_hash FROM bridge_exchange_codes WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("read stored code");
    assert_ne!(stored, issued.code, "the code travels, the hash stays");
    assert_eq!(issued.code.len(), 64, "32 bytes, hex encoded");
    let ttl = (issued.expires_at - Utc::now()).num_seconds();
    assert!(
        ttl > EXCHANGE_CODE_TTL_SECONDS - 60 && ttl <= EXCHANGE_CODE_TTL_SECONDS,
        "expiry sits within the declared ten-minute window, got {ttl}s"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn issue_exchange_code_mints_a_fresh_code_each_time() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let first = issue_exchange_code(&db.pool, &user_id(&user))
        .await
        .expect("first code");
    let second = issue_exchange_code(&db.pool, &user_id(&user))
        .await
        .expect("second code");

    assert_ne!(first.code, second.code);
    let count = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM bridge_exchange_codes WHERE user_id = $1",
    )
    .bind(&user)
    .fetch_one(&*db.pool)
    .await
    .expect("count codes");
    assert_eq!(count, 2);

    db.cleanup().await;
}

#[tokio::test]
async fn issue_exchange_code_fails_for_a_user_who_does_not_exist() {
    let Some(db) = TempDb::create().await else {
        return;
    };

    let result = issue_exchange_code(&db.pool, &user_id(&unique("ghost"))).await;

    assert!(matches!(result, Err(BridgeRepoError::Database(_))));

    db.cleanup().await;
}

#[tokio::test]
async fn enroll_device_issues_a_key_and_links_the_installation() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let device = enroll_device(
        &db.pool,
        &user_id(&user),
        device_params("  Ed's laptop  ", "  MacOS  "),
    )
    .await
    .expect("enroll device");

    assert_eq!(device.name, "Ed's laptop");
    assert_eq!(device.platform, "macos", "the platform is normalised");
    assert_eq!(device.hostname, "workstation");
    assert!(device.secret.starts_with(&device.key_prefix));
    let linked = sqlx::query_scalar::<_, String>(
        "SELECT app_platform FROM device_app_links WHERE device_id = $1",
    )
    .bind(&device.id)
    .fetch_one(&*db.pool)
    .await
    .expect("read the link row");
    assert_eq!(linked, "macos");

    db.cleanup().await;
}

#[tokio::test]
async fn enroll_device_rejects_an_unsupported_platform() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let result = enroll_device(
        &db.pool,
        &user_id(&user),
        device_params("laptop", "solaris"),
    )
    .await;

    assert!(matches!(result, Err(BridgeRepoError::Validation(_))));
    let keys = list_api_keys_for_user(&db.pool, &user_id(&user))
        .await
        .expect("list keys");
    assert!(keys.is_empty(), "a rejected enrolment must issue no key");

    db.cleanup().await;
}

#[tokio::test]
async fn enroll_device_rejects_a_blank_name() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let result = enroll_device(&db.pool, &user_id(&user), device_params("  ", "linux")).await;

    assert!(matches!(result, Err(BridgeRepoError::Validation(_))));

    db.cleanup().await;
}

#[tokio::test]
async fn enroll_device_gives_each_installation_its_own_credential() {
    let Some(db) = TempDb::create().await else {
        return;
    };
    let user = unique("u");
    insert_user(&db.pool, &user).await;

    let laptop = enroll_device(&db.pool, &user_id(&user), device_params("laptop", "linux"))
        .await
        .expect("first enrolment");
    let desktop = enroll_device(
        &db.pool,
        &user_id(&user),
        device_params("desktop", "windows"),
    )
    .await
    .expect("second enrolment");

    assert_ne!(laptop.id, desktop.id);
    assert_ne!(laptop.secret, desktop.secret);
    assert_eq!(desktop.platform, "windows");
    let keys = list_api_keys_for_user(&db.pool, &user_id(&user))
        .await
        .expect("list keys");
    assert_eq!(
        keys.len(),
        2,
        "revoking one machine must not disconnect the other"
    );

    db.cleanup().await;
}
