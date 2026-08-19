//! "Sign in with Salesforce" — the branches reachable without a live org.
//!
//! SSO reports failure by *redirecting*, not by returning an error status: a
//! `500` on the callback would strand the browser on a dead end instead of
//! returning it to a usable login page. Every case here therefore asserts on
//! the `Location` header — `?sso=<reason>` for a login, `?sf=<outcome>` for the
//! profile link flow — because the status is `303` on success and failure
//! alike and proves nothing on its own.
//!
//! The suite runs the router twice. Once with SSO disabled, which is how
//! [`crate::app::App::new`] builds it and which pins the "unavailable"
//! response every route owes when no `salesforce.yaml` is present. Once with a
//! configuration pointed at `127.0.0.1:9` — the discard port, which refuses
//! connections immediately — so the PKCE redirect, the state cookie, and the
//! callback's whole validation ladder run for real while the network call
//! fails fast and locally.

use axum::http::StatusCode;
use systemprompt_web_admin::SalesforceConfig;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const START: &str = "/admin/auth/salesforce/start";
const CALLBACK: &str = "/admin/auth/salesforce/callback";

// A configuration that is structurally complete but points at a port nothing
// listens on, so `is_usable()` passes and the token exchange fails on connect.
fn unreachable_org() -> SalesforceConfig {
    SalesforceConfig {
        enabled: true,
        my_domain: "http://127.0.0.1:9".to_owned(),
        consumer_key: "contract-consumer-key".to_owned(),
        redirect_uri: "http://localhost:8099/admin/auth/salesforce/callback".to_owned(),
        scopes: "openid email profile api".to_owned(),
        allowed_email_domains: vec!["contract.test".to_owned()],
        auto_provision: true,
    }
}

// With no `salesforce.yaml`, every SSO route reports unavailable rather than
// pretending the flow can start.
#[tokio::test(flavor = "multi_thread")]
async fn salesforce_routes_report_unavailable_when_sso_is_not_configured() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping Salesforce SSO suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    for path in [
        START,
        "/admin/auth/salesforce/start?redirect=/admin/profile",
        "/admin/auth/salesforce/start?mode=link",
        CALLBACK,
        "/admin/auth/salesforce/callback?code=abc&state=xyz",
    ] {
        let (status, location_header) =
            app.redirect_of(Call::get(path, Principal::Anonymous)).await;
        if !status.is_redirection() {
            failures.push(format!(
                "  {path} -> {} (expected a redirect)",
                status.as_u16()
            ));
            continue;
        }
        if !location_header.contains("sso=unavailable") {
            failures.push(format!(
                "  {path} -> redirected to {location_header:?}, expected ?sso=unavailable"
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} disabled-SSO case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// With SSO configured, `start` builds the authorize redirect and the state
// cookie that the callback later validates against.
#[tokio::test(flavor = "multi_thread")]
async fn salesforce_start_issues_a_pkce_challenge_and_a_state_cookie() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::with_salesforce(&db.pool, credentials, unreachable_org());

    let mut failures = Vec::new();
    let expectations: [(&str, &[&str]); 3] = [
        (
            START,
            &[
                "127.0.0.1:9/services/oauth2/authorize",
                "response_type=code",
                "client_id=contract-consumer-key",
                "code_challenge_method=S256",
            ],
        ),
        // The post-login target is carried in the cookie, not the OAuth state,
        // so it cannot be tampered with in transit.
        (
            "/admin/auth/salesforce/start?redirect=/admin/profile",
            &["code_challenge="],
        ),
        // An off-site redirect target is replaced by the default rather than
        // honoured — this is the open-redirect gate.
        (
            "/admin/auth/salesforce/start?redirect=https://evil.example/steal",
            &["code_challenge="],
        ),
    ];

    for (path, markers) in expectations {
        let (status, headers) = app
            .response_headers(Call::get(path, Principal::Anonymous))
            .await;
        if !status.is_redirection() {
            failures.push(format!(
                "  {path} -> {} (expected a redirect)",
                status.as_u16()
            ));
            continue;
        }
        let target = headers.location.clone().unwrap_or_default();
        for marker in markers {
            if !target.contains(marker) {
                failures.push(format!(
                    "  {path} -> authorize URL {target:?} lacks {marker:?}"
                ));
            }
        }
        let cookie = headers.set_cookie.join(" ");
        if !cookie.contains("sf_oauth_state=") {
            failures.push(format!(
                "  {path} -> set no sf_oauth_state cookie: {cookie:?}"
            ));
        }
        if !cookie.contains("HttpOnly") || !cookie.contains("SameSite=Lax") {
            failures.push(format!(
                "  {path} -> state cookie is not HttpOnly+Lax: {cookie:?}"
            ));
        }
    }

    // The off-site target must not survive into the cookie.
    let (_, headers) = app
        .response_headers(Call::get(
            "/admin/auth/salesforce/start?redirect=https://evil.example/steal",
            Principal::Anonymous,
        ))
        .await;
    let cookie = headers.set_cookie.join(" ");
    if cookie.contains("evil.example") {
        failures.push(format!(
            "  an off-site redirect target survived into the state cookie: {cookie:?}"
        ));
    }

    // `mode=link` marks the flow as "attach to the signed-in user"; the marker
    // lives in the cookie so the callback can tell the two flows apart.
    let (_, headers) = app
        .response_headers(Call::get(
            "/admin/auth/salesforce/start?mode=link",
            Principal::Anonymous,
        ))
        .await;
    if !headers.set_cookie.join(" ").contains("|link") {
        failures.push("  mode=link did not mark the state cookie as a link flow".to_owned());
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} Salesforce start case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The callback's validation ladder, each rung reached by a request that is
// wrong in exactly one way.
#[tokio::test(flavor = "multi_thread")]
async fn salesforce_callback_rejects_every_malformed_return() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::with_salesforce(&db.pool, credentials, unreachable_org());

    // A cookie matching the state the case sends, so the ladder can be entered
    // above the state check.
    let good_cookie = "sf_oauth_state=state-1|verifier-1|/admin";
    let link_cookie = "sf_oauth_state=state-1|verifier-1|/admin|link";

    let cases: [(&str, &str, Option<&str>, &str); 6] = [
        (
            "Salesforce reported an OAuth error",
            "/admin/auth/salesforce/callback?error=access_denied&error_description=user+said+no",
            None,
            "sso=denied",
        ),
        (
            "no code and no state",
            CALLBACK,
            Some(good_cookie),
            "sso=error",
        ),
        (
            "a code with no state",
            "/admin/auth/salesforce/callback?code=abc",
            Some(good_cookie),
            "sso=error",
        ),
        (
            "a state with no cookie to compare it against",
            "/admin/auth/salesforce/callback?code=abc&state=state-1",
            None,
            "sso=error",
        ),
        (
            "a state that does not match the cookie",
            "/admin/auth/salesforce/callback?code=abc&state=someone-elses-state",
            Some(good_cookie),
            "sso=error",
        ),
        // The link flow belongs to a signed-in user; without a session cookie
        // there is nobody to attach the identity to.
        (
            "a link-mode return with no signed-in user",
            "/admin/auth/salesforce/callback?code=abc&state=state-1",
            Some(link_cookie),
            "sso=error",
        ),
    ];

    let mut failures = Vec::new();
    for (label, path, cookie, marker) in cases {
        let call = Call::get(path, Principal::Anonymous);
        let headers: Vec<(&str, &str)> = cookie.map(|c| vec![("cookie", c)]).unwrap_or_default();
        let (status, location_header) = app.redirect_with_headers(call, &headers).await;
        if !status.is_redirection() {
            failures.push(format!(
                "  {label} -> {} (expected a redirect)",
                status.as_u16()
            ));
            continue;
        }
        if !location_header.contains(marker) {
            failures.push(format!(
                "  {label} -> redirected to {location_header:?}, expected {marker:?}"
            ));
        }
    }

    // A well-formed return whose code cannot be exchanged — no client secret is
    // configured in the fixture, so the exchange is refused before any network
    // call — still lands the browser back on the login page rather than an
    // error document.
    let (status, location_header) = app
        .redirect_with_headers(
            Call::get(
                "/admin/auth/salesforce/callback?code=abc&state=state-1",
                Principal::Anonymous,
            ),
            &[("cookie", good_cookie)],
        )
        .await;
    if !status.is_redirection() || !location_header.contains("/admin/login?sso=") {
        failures.push(format!(
            "  an unexchangeable code -> {} {location_header:?}, expected a login redirect",
            status.as_u16()
        ));
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} Salesforce callback case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The unlink endpoint refuses a user whose only remaining credential would be
// the Salesforce identity they are detaching.
#[tokio::test(flavor = "multi_thread")]
async fn salesforce_unlink_requires_another_credential() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::with_salesforce(&db.pool, credentials, unreachable_org());

    let (status, body) = app
        .call(Call::json(
            "post",
            "/admin/api/profile/salesforce/unlink",
            Principal::Admin,
            "{}",
        ))
        .await;
    assert!(
        !status.is_server_error(),
        "unlink faulted: {} {}",
        status.as_u16(),
        body.chars().take(200).collect::<String>()
    );
    assert_ne!(
        status,
        StatusCode::OK,
        "a user with no passkey must not be able to unlink their only credential"
    );

    let (status, _) = app
        .call(Call::json(
            "post",
            "/admin/api/profile/salesforce/unlink",
            Principal::Anonymous,
            "{}",
        ))
        .await;
    assert!(
        status == StatusCode::UNAUTHORIZED || status.is_redirection(),
        "an anonymous unlink must be refused, got {}",
        status.as_u16()
    );

    db.cleanup().await;
}
