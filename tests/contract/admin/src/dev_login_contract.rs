//! The developer login link: `GET /admin/auth/dev/login?code=…`.
//!
//! Like SSO, the route answers with a redirect in every case, so each case
//! asserts on `Location` and `Set-Cookie`: a live code lands on `/admin`
//! carrying an `access_token`, and does so again inside the short grace that
//! lets the person's browser follow a prefetcher; a spent, unknown, missing or
//! expired code lands on `/admin/login?dev=invalid` carrying nothing, and the
//! failures are indistinguishable so the route never says whether an account
//! exists.
//!
//! The route is mounted because the fixture profile is development/local.
//! Its absence on production cannot be shown here — the profile is a
//! per-process `OnceLock` — and is pinned by the gate's unit test instead.

use systemprompt_web_admin::repositories::dev_login::{hash_dev_login_code, insert_dev_login_code};

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const REDEEM: &str = "/admin/auth/dev/login";
const INVALID: &str = "/admin/login?dev=invalid";

fn has_session_cookie(cookies: &[String]) -> bool {
    cookies
        .iter()
        .any(|c| c.starts_with("access_token=") && c.contains("HttpOnly"))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_live_code_signs_in_and_survives_a_prefetch_race() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping dev login suite");
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let issued = insert_dev_login_code(&db.pool, &credentials.non_admin_user_id)
        .await
        .expect("issue a code");
    let expected_user_id = credentials.non_admin_user_id.to_string();
    let app = App::new(&db.pool, credentials);
    let path = format!("{REDEEM}?code={}", issued.code);

    let (status, headers) = app
        .response_headers(Call::get(&path, Principal::Anonymous))
        .await;
    assert_eq!(status, 303, "success is a redirect");
    assert_eq!(headers.location.as_deref(), Some("/admin"));
    assert!(
        has_session_cookie(&headers.set_cookie),
        "the redeem must set the HttpOnly access_token cookie"
    );

    let cookie = headers
        .set_cookie
        .iter()
        .find(|cookie| cookie.starts_with("access_token="))
        .expect("session cookie");
    let token = cookie
        .trim_start_matches("access_token=")
        .split(';')
        .next()
        .expect("cookie value");
    let (identity_status, identity_body) = app
        .call_with_bearer(Call::get("/admin/auth/me", Principal::Anonymous), token)
        .await;
    assert_eq!(
        identity_status, 200,
        "the minted cookie is a usable browser session"
    );
    let identity: serde_json::Value =
        serde_json::from_str(&identity_body).expect("identity response");
    assert_eq!(
        identity["is_admin"], false,
        "dev login preserves the user's role"
    );
    assert_eq!(
        identity["user_id"], expected_user_id,
        "string user ids survive session minting"
    );

    let (status, headers) = app
        .response_headers(Call::get(&path, Principal::Anonymous))
        .await;
    assert_eq!(status, 303);
    assert_eq!(
        headers.location.as_deref(),
        Some("/admin"),
        "a second redeem inside the grace window is the real browser arriving after a prefetch"
    );
    assert!(has_session_cookie(&headers.set_cookie));

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn a_code_spent_longer_ago_than_the_grace_is_dead() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping dev login suite");
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let spent = "d".repeat(64);
    sqlx::query(
        "INSERT INTO dev_login_codes (code_hash, user_id, expires_at, consumed_at) \
         VALUES ($1, $2, NOW() + INTERVAL '5 minutes', NOW() - INTERVAL '2 minutes')",
    )
    .bind(hash_dev_login_code(&spent))
    .bind(credentials.non_admin_user_id.as_str())
    .execute(&*db.pool)
    .await
    .expect("seed a spent code");
    let app = App::new(&db.pool, credentials);

    let path = format!("{REDEEM}?code={spent}");
    let (status, headers) = app
        .response_headers(Call::get(&path, Principal::Anonymous))
        .await;
    assert_eq!(status, 303);
    assert_eq!(headers.location.as_deref(), Some(INVALID));
    assert!(!has_session_cookie(&headers.set_cookie));

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn missing_unknown_and_expired_codes_all_fail_the_same_way() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping dev login suite");
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let expired = "e".repeat(64);
    sqlx::query(
        "INSERT INTO dev_login_codes (code_hash, user_id, expires_at) \
         VALUES ($1, $2, NOW() - INTERVAL '1 minute')",
    )
    .bind(hash_dev_login_code(&expired))
    .bind(credentials.non_admin_user_id.as_str())
    .execute(&*db.pool)
    .await
    .expect("seed an expired code");
    let app = App::new(&db.pool, credentials);

    let unknown = format!("{REDEEM}?code={}", "f".repeat(64));
    let expired_path = format!("{REDEEM}?code={expired}");
    for path in [REDEEM, unknown.as_str(), expired_path.as_str()] {
        let (status, headers) = app
            .response_headers(Call::get(path, Principal::Anonymous))
            .await;
        assert_eq!(status, 303, "{path}");
        assert_eq!(headers.location.as_deref(), Some(INVALID), "{path}");
        assert!(!has_session_cookie(&headers.set_cookie), "{path}");
    }

    db.cleanup().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn redeem_checks_current_status_and_roles() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let user_id = credentials.non_admin_user_id.clone();
    let issued = insert_dev_login_code(&db.pool, &user_id)
        .await
        .expect("issue code");
    sqlx::query("UPDATE users SET status = 'suspended' WHERE id = $1")
        .bind(user_id.as_str())
        .execute(&*db.pool)
        .await
        .expect("suspend user");
    let app = App::new(&db.pool, credentials);
    let path = format!("{REDEEM}?code={}", issued.code);
    let (_, rejected) = app
        .response_headers(Call::get(&path, Principal::Anonymous))
        .await;
    assert_eq!(rejected.location.as_deref(), Some(INVALID));
    assert!(!has_session_cookie(&rejected.set_cookie));

    sqlx::query(
        "UPDATE users SET status = 'active', roles = ARRAY['platform_admin', 'user'] WHERE id = $1",
    )
    .bind(user_id.as_str())
    .execute(&*db.pool)
    .await
    .expect("reactivate with new role");
    let (_, accepted) = app
        .response_headers(Call::get(&path, Principal::Anonymous))
        .await;
    assert_eq!(accepted.location.as_deref(), Some("/admin"));
    let cookie = accepted
        .set_cookie
        .iter()
        .find(|cookie| cookie.starts_with("access_token="))
        .expect("session cookie");
    let token = cookie
        .trim_start_matches("access_token=")
        .split(';')
        .next()
        .expect("cookie value");
    let claims = systemprompt::oauth::validate_jwt_token(
        token,
        &globals::jwt_issuer(),
        &[systemprompt::models::auth::JwtAudience::Api],
    )
    .expect("validate the issued session token");
    assert_eq!(claims.sub, user_id.as_str());
    assert!(claims.roles.iter().any(|role| role == "platform_admin"));
    assert!(
        claims
            .scope
            .contains(&systemprompt::models::auth::Permission::Admin)
    );
    db.cleanup().await;
}
