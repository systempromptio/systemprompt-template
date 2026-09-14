//! "Sign in with Astound SSO" — the branches reachable without a live farm.
//!
//! SSO reports failure by *redirecting*, not by returning an error status: a
//! `500` on the callback would strand the browser on a dead end instead of
//! returning it to a usable login page. Every case here therefore asserts on
//! the `Location` header — `?sso=<reason>` — because the status is `303` on
//! success and failure alike and proves nothing on its own.
//!
//! The suite runs the router twice. Once with SSO disabled, which is how
//! [`crate::app::App::new`] builds it and which pins the "unavailable"
//! response every route owes when no `adfs.yaml` is present. Once with the
//! real, committed AD FS federation metadata, so the AuthnRequest, the state
//! cookie, and the callback's whole validation ladder — including a genuine
//! XML-DSig check against the pinned certificate — run for real, offline.

use std::collections::BTreeMap;

use systemprompt_web_admin::AdfsConfig;

use crate::app::{App, Call};
use crate::principal::Principal;
use crate::tempdb::TempDb;
use crate::{globals, principal};

const START: &str = "/admin/auth/adfs/start";
const ACS_PATH: &str = "/admin/auth/adfs/acs";
const ACS: &str = "https://sp-dev.astound.digital/admin/auth/adfs/acs";

fn configured() -> AdfsConfig {
    let metadata = globals::repo_root().join("services/web/config/adfs-federation-metadata.xml");
    AdfsConfig {
        enabled: true,
        entity_id: "https://sp-dev.astound.digital/saml/metadata".to_owned(),
        acs_url: ACS.to_owned(),
        idp_metadata_path: "adfs-federation-metadata.xml".to_owned(),
        idp_metadata_xml: std::fs::read_to_string(metadata).expect("committed IdP metadata"),
        allowed_email_domains: vec!["contract.test".to_owned()],
        auto_provision: true,
        email_attribute: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/emailaddress"
            .to_owned(),
        name_attribute: "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/name".to_owned(),
        groups_attribute: "http://temp/groups".to_owned(),
        deny_without_group: true,
        allow_idp_initiated: true,
        clock_skew_seconds: 120,
        group_roles: BTreeMap::from([("Systemprompt-Users".to_owned(), vec!["user".to_owned()])]),
        group_role_patterns: BTreeMap::new(),
    }
}

// With no `adfs.yaml`, every SSO route reports unavailable rather than
// pretending the flow can start.
#[tokio::test(flavor = "multi_thread")]
async fn adfs_routes_report_unavailable_when_sso_is_not_configured() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        eprintln!("no DATABASE_URL — skipping ADFS SSO suite");
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::new(&db.pool, credentials);

    let mut failures = Vec::new();
    for call in [
        Call::get(START, Principal::Anonymous),
        Call::get(
            "/admin/auth/adfs/start?redirect=/admin/profile",
            Principal::Anonymous,
        ),
        Call {
            method: "post",
            path: ACS_PATH,
            principal: Principal::Anonymous,
            content_type: Some("application/x-www-form-urlencoded"),
            body: Some("SAMLResponse=abc"),
        },
    ] {
        let path = call.path;
        let (status, location_header) = app.redirect_of(call).await;
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

// With SSO configured, `start` builds the AuthnRequest redirect to the farm
// and the state cookie the callback later validates against.
#[tokio::test(flavor = "multi_thread")]
async fn adfs_start_redirects_to_the_farm_with_a_request_and_a_state_cookie() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::with_adfs(&db.pool, credentials, configured());

    let mut failures = Vec::new();
    for path in [
        START,
        "/admin/auth/adfs/start?redirect=/admin/profile",
        // An off-site redirect target is replaced by the default rather than
        // honoured — this is the open-redirect gate.
        "/admin/auth/adfs/start?redirect=https://evil.example/steal",
    ] {
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
        for marker in [
            "https://login.astoundcommerce.com/adfs/ls/",
            "SAMLRequest=",
            "RelayState=",
        ] {
            if !target.contains(marker) {
                failures.push(format!("  {path} -> SSO URL {target:?} lacks {marker:?}"));
            }
        }
        let cookie = headers.set_cookie.join(" ");
        if !cookie.contains("adfs_saml_state=") {
            failures.push(format!(
                "  {path} -> set no adfs_saml_state cookie: {cookie:?}"
            ));
        }
        if !cookie.contains("HttpOnly") || !cookie.contains("SameSite=Lax") {
            failures.push(format!(
                "  {path} -> state cookie is not HttpOnly+Lax: {cookie:?}"
            ));
        }
        if cookie.contains("evil.example") {
            failures.push(format!(
                "  an off-site redirect target survived into the state cookie: {cookie:?}"
            ));
        }
    }

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} ADFS start case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

// The callback's validation ladder, each rung reached by a request that is
// wrong in exactly one way.
#[tokio::test(flavor = "multi_thread")]
async fn adfs_callback_rejects_every_malformed_return() {
    if !globals::init() {
        return;
    }
    let Some(db) = TempDb::create().await else {
        return;
    };

    let credentials = principal::provision(&db.pool).await;
    let app = App::with_adfs(&db.pool, credentials, configured());

    // A cookie matching the RelayState the case sends, so the ladder can be
    // entered above the correlation check.
    let good_cookie = "adfs_saml_state=state-1|_req-1|1700000000|/admin";
    // A well-formed but unsigned Response — base64 of a minimal document.
    let unsigned = "PHNhbWxwOlJlc3BvbnNlIHhtbG5zOnNhbWxwPSJ1cm46b2FzaXM6bmFtZXM6dGM6U0FNTDoyLjA6cHJvdG9jb2wiIElEPSJfeCIgVmVyc2lvbj0iMi4wIiBJc3N1ZUluc3RhbnQ9IjIwMjYtMDEtMDFUMDA6MDA6MDBaIi8+";

    let cases: [(&str, &str, Option<&str>, &str); 5] = [
        ("no form body at all", "", Some(good_cookie), "sso=error"),
        (
            "a RelayState that does not match the cookie",
            "SAMLResponse=abc&RelayState=someone-elses-state",
            Some(good_cookie),
            "sso=error",
        ),
        (
            "a response that is not base64",
            "SAMLResponse=%21%21%21&RelayState=state-1",
            Some(good_cookie),
            "sso=invalid_assertion",
        ),
        (
            "an unsigned response with a matching RelayState",
            &format!("SAMLResponse={unsigned}&RelayState=state-1"),
            Some(good_cookie),
            "sso=invalid_assertion",
        ),
        // IdP-initiated: no cookie, no RelayState. Allowed by config, so the
        // ladder proceeds to signature verification and fails there.
        (
            "an unsolicited unsigned response",
            &format!("SAMLResponse={unsigned}"),
            None,
            "sso=invalid_assertion",
        ),
    ];

    let mut failures = Vec::new();
    for (label, body, cookie, marker) in cases {
        let call = Call {
            method: "post",
            path: ACS_PATH,
            principal: Principal::Anonymous,
            content_type: Some("application/x-www-form-urlencoded"),
            body: Some(body),
        };
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

    db.cleanup().await;
    assert!(
        failures.is_empty(),
        "{} ADFS callback case(s) failed:\n{}",
        failures.len(),
        failures.join("\n")
    );
}
