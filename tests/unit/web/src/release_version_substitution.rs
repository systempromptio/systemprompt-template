//! Hosted documentation carries `@RELEASE_VERSION@` instead of a literal
//! version, and ingestion substitutes the crate version. These pin the
//! contract: the token is replaced everywhere, text without it is untouched,
//! and the substituted value is the workspace release version itself.

use systemprompt_web_extension::services::release_version::{
    RELEASE_VERSION, RELEASE_VERSION_TOKEN, substitute_release_version, substitute_with,
};

#[test]
fn every_token_is_replaced_with_the_given_version() {
    let text = "pull image:@RELEASE_VERSION@ then SYSTEMPROMPT_TAG=@RELEASE_VERSION@";
    assert_eq!(
        substitute_with(text, "1.2.3"),
        "pull image:1.2.3 then SYSTEMPROMPT_TAG=1.2.3"
    );
}

#[test]
fn text_without_the_token_is_returned_unchanged() {
    let text = "no version here, not even 0.1.0";
    assert_eq!(substitute_with(text, "9.9.9"), text);
}

#[test]
fn the_default_substitution_uses_the_workspace_release_version() {
    let out = substitute_release_version(RELEASE_VERSION_TOKEN);
    assert_eq!(out, RELEASE_VERSION);
    let parts: Vec<&str> = out.split('.').collect();
    assert_eq!(parts.len(), 3, "release version is X.Y.Z, got {out}");
    assert!(parts.iter().all(|p| p.parse::<u32>().is_ok()), "{out}");
}
