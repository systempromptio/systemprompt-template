//! The Salesforce JWT-bearer `sub` must be the Salesforce Username (userinfo
//! `preferred_username`), not the login email — the two differ for orgs where
//! the Username is a generated handle. These tests pin the selection +
//! fallback.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::test_support::select_sf_username;

#[test]
fn prefers_preferred_username_over_email() {
    let got = select_sf_username(Some("ed.aa5967144c6c@agentforce.com"), "ed@systemprompt.io");
    assert_eq!(got, "ed.aa5967144c6c@agentforce.com");
}

#[test]
fn falls_back_to_email_when_absent() {
    assert_eq!(
        select_sf_username(None, "ed@systemprompt.io"),
        "ed@systemprompt.io"
    );
}

#[test]
fn falls_back_to_email_when_blank() {
    assert_eq!(
        select_sf_username(Some("   "), "ed@systemprompt.io"),
        "ed@systemprompt.io"
    );
}

#[test]
fn trims_surrounding_whitespace() {
    assert_eq!(
        select_sf_username(Some("  user@example.com  "), "ed@systemprompt.io"),
        "user@example.com"
    );
}
