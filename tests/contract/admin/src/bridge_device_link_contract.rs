//! The bridge device-link flow, in full.
//!
//! This is the one place in the admin plane where a *browser* hands a
//! credential to a *local process*, and the only thing standing between the
//! exchange code and a third party is `validate_loopback_redirect`. The
//! exhaustive contract table cannot reach any of it: `ssr_bridge.rs` is not
//! one of the two route modules [`crate::route_source`] reads, and the branch
//! that matters is selected by a query parameter and a form field rather than
//! by the path.
//!
//! The redirect validator is therefore driven as a table of rejections, each
//! one a different way of not being loopback, and the approve/deny handlers
//! are driven both ways — with a callback, where the outcome is a `Location`,
//! and without one, where the outcome is a rendered page carrying the code for
//! the user to paste back into a terminal.

use axum::http::StatusCode;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const PAGE: &str = "/bridge-auth/device-link";
const APPROVE: &str = "/bridge-auth/device-link/approve";
const DENY: &str = "/bridge-auth/device-link/deny";

const FORM: &str = "application/x-www-form-urlencoded";

fn form<'a>(path: &'a str, body: &'a str) -> Call<'a> {
    Call {
        method: "post",
        path,
        principal: Principal::Admin,
        content_type: Some(FORM),
        body: Some(body),
    }
}

// Every shape of redirect the validator must refuse, and why it is not a
// loopback callback. A miss on any of these hands the exchange code to
// whoever owns the host.
const NON_LOOPBACK: [(&str, &str); 6] = [
    ("not a url at all", "just-a-string"),
    ("no port", "http://127.0.0.1/callback"),
    ("https scheme", "https://127.0.0.1:9999/callback"),
    ("a public host", "http://example.com:9999/callback"),
    (
        "loopback in the path, not the host",
        "http://example.com:9999/127.0.0.1",
    ),
    (
        "loopback as a userinfo prefix",
        "http://127.0.0.1@example.com:9999/callback",
    ),
];

#[tokio::test]
async fn the_device_link_page_requires_a_signed_in_user() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, _) = app.call(Call::get(PAGE, Principal::Anonymous)).await;
    assert!(
        !status.is_success(),
        "an anonymous browser cannot reach the approval page, got {status}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn the_device_link_page_renders_with_and_without_a_callback() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // No callback: the CLI-without-a-browser case. The page must still render,
    // because the code will be shown rather than posted back.
    let (status, body) = app.call(Call::get(PAGE, Principal::Admin)).await;
    assert_eq!(status, StatusCode::OK, "no redirect: {body}");
    assert!(!body.is_empty(), "the page renders");

    // Both accepted loopback spellings, each with a port.
    for redirect in [
        "http://127.0.0.1:9876/callback",
        "http://localhost:31337/cb",
    ] {
        let path = format!("{PAGE}?redirect={}", urlencode(redirect));
        let (status, body) = app.call(Call::get(&path, Principal::Admin)).await;
        assert_eq!(status, StatusCode::OK, "{redirect}: {body}");
        let host = redirect
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or_default();
        assert!(
            body.contains(host),
            "the page names the callback host {host} it is about to trust"
        );
    }

    db.cleanup().await;
}

#[tokio::test]
async fn a_non_loopback_callback_is_refused_on_every_endpoint() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    for (label, redirect) in NON_LOOPBACK {
        let encoded = urlencode(redirect);

        let (status, body) = app
            .call(Call::get(
                &format!("{PAGE}?redirect={encoded}"),
                Principal::Admin,
            ))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "page, {label}: {body}");
        assert!(
            body.contains("Invalid redirect"),
            "page, {label}: the refusal names itself"
        );

        // Approve is the dangerous one: a redirect that slipped past here
        // would be handed a live exchange code.
        let (status, body) = app
            .call(form(APPROVE, &format!("redirect={encoded}")))
            .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "approve, {label}: {body}");
        assert!(
            !body.contains("code="),
            "approve, {label}: no code is issued for a rejected callback"
        );

        let (status, body) = app.call(form(DENY, &format!("redirect={encoded}"))).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "deny, {label}: {body}");
    }

    db.cleanup().await;
}

#[tokio::test]
async fn approving_without_a_callback_displays_the_code_and_the_login_command() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let (status, body) = app.call(form(APPROVE, "")).await;
    assert_eq!(status, StatusCode::OK, "approve without redirect: {body}");
    assert!(
        body.contains("astound-bridge login --code"),
        "the page prints the command the user is meant to paste, got: {body}"
    );

    db.cleanup().await;
}

#[tokio::test]
async fn approving_with_a_callback_redirects_carrying_the_code() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // A callback with no query string of its own: the code is appended with
    // `?`.
    let (status, location) = app
        .redirect_of(form(
            APPROVE,
            &format!("redirect={}", urlencode("http://127.0.0.1:9876/callback")),
        ))
        .await;
    assert!(status.is_redirection(), "approve redirects, got {status}");
    assert!(
        location.starts_with("http://127.0.0.1:9876/callback?code="),
        "the code is appended with `?`, got {location}"
    );

    // A callback that already carries a query string: appending another `?`
    // would corrupt it, so the separator must be `&`.
    let (_, location) = app
        .redirect_of(form(
            APPROVE,
            &format!(
                "redirect={}",
                urlencode("http://localhost:9876/callback?state=abc")
            ),
        ))
        .await;
    assert!(
        location.starts_with("http://localhost:9876/callback?state=abc&code="),
        "an existing query string is preserved, got {location}"
    );

    // Two approvals must not hand out the same code.
    let first = location
        .split("code=")
        .nth(1)
        .unwrap_or_default()
        .to_owned();
    let (_, location) = app
        .redirect_of(form(
            APPROVE,
            &format!("redirect={}", urlencode("http://127.0.0.1:9876/cb")),
        ))
        .await;
    let second = location.split("code=").nth(1).unwrap_or_default();
    assert_ne!(first, second, "each approval issues a fresh exchange code");
    assert!(!first.is_empty(), "the code is not empty");

    db.cleanup().await;
}

#[tokio::test]
async fn denying_reports_the_refusal_both_ways() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };
    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    // With a callback the bridge is told, out of band, that the user said no.
    let (status, location) = app
        .redirect_of(form(
            DENY,
            &format!("redirect={}", urlencode("http://127.0.0.1:9876/callback")),
        ))
        .await;
    assert!(status.is_redirection(), "deny redirects, got {status}");
    assert_eq!(location, "http://127.0.0.1:9876/callback?error=denied");
    assert!(
        !location.contains("code="),
        "a denial never carries an exchange code"
    );

    // Without one, the page itself has to say so.
    let (status, body) = app.call(form(DENY, "")).await;
    assert_eq!(status, StatusCode::OK, "deny without redirect: {body}");
    assert!(
        !body.contains("astound-bridge login --code"),
        "the denial page offers no login command"
    );

    db.cleanup().await;
}

// Percent-encode the characters that would otherwise be read as query or form
// structure. Enough for the URLs these cases use.
fn urlencode(value: &str) -> String {
    value
        .chars()
        .map(|c| match c {
            ':' => "%3A".to_owned(),
            '/' => "%2F".to_owned(),
            '?' => "%3F".to_owned(),
            '&' => "%26".to_owned(),
            '=' => "%3D".to_owned(),
            '@' => "%40".to_owned(),
            other => other.to_string(),
        })
        .collect()
}
