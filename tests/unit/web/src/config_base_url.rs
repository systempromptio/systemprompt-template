//! `BlogConfigValidated::validate` runs the private `validate_base_url` over the
//! raw `base_url`: an `http`/`https` URL is accepted, any other scheme or an
//! unparseable string is rejected via the aggregated `ExtensionConfigErrors`.

use std::path::Path;
use systemprompt_web_shared::config::{BlogConfigRaw, BlogConfigValidated};

fn raw_with_base_url(base_url: &str) -> BlogConfigRaw {
    BlogConfigRaw {
        content_sources: Vec::new(),
        base_url: base_url.to_owned(),
        enable_link_tracking: true,
    }
}

#[test]
fn accepts_https_and_exposes_parsed_url() {
    let cfg = BlogConfigValidated::validate(
        raw_with_base_url("https://demo.systemprompt.io"),
        Path::new("."),
    )
    .expect("https base_url validates");
    assert_eq!(cfg.base_url().as_str(), "https://demo.systemprompt.io/");
    assert!(cfg.link_tracking_enabled());
}

#[test]
fn accepts_http() {
    let cfg =
        BlogConfigValidated::validate(raw_with_base_url("http://localhost:8080"), Path::new("."))
            .expect("http base_url validates");
    assert_eq!(cfg.base_url().scheme(), "http");
}

#[test]
fn rejects_non_http_scheme() {
    let err = BlogConfigValidated::validate(
        raw_with_base_url("ftp://files.example.com"),
        Path::new("."),
    )
    .expect_err("ftp scheme is rejected");
    // The scheme error is recorded against the base_url field.
    assert!(
        format!("{err:?}").contains("base_url"),
        "expected a base_url error, got: {err:?}"
    );
}

#[test]
fn rejects_unparseable_url() {
    let err =
        BlogConfigValidated::validate(raw_with_base_url("not a url"), Path::new("."))
            .expect_err("garbage base_url is rejected");
    assert!(
        format!("{err:?}").contains("base_url"),
        "expected a base_url error, got: {err:?}"
    );
}

#[test]
fn link_tracking_flag_is_carried_through() {
    let cfg = BlogConfigValidated::validate(
        BlogConfigRaw {
            content_sources: Vec::new(),
            base_url: "https://example.com".to_owned(),
            enable_link_tracking: false,
        },
        Path::new("."),
    )
    .expect("validates");
    assert!(!cfg.link_tracking_enabled());
}
