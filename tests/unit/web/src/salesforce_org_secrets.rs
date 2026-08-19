//! `salesforce_org` secret-bearing types must never render their secrets.
//!
//! `TargetOrg` holds an RSA private key and `Connection` holds a live bearer
//! token. Both had derived `Debug` implementations at first, which would print
//! that material in full anywhere the value reached a `{:?}` or a tracing
//! field. The hand-written impls redact it; these tests are what stop a future
//! `#[derive(Debug)]` from quietly putting it back.

use systemprompt_web_admin::salesforce_org::TargetOrg;

const PRIVATE_KEY: &str =
    "-----BEGIN PRIVATE KEY-----\nMIIEvQIBADSECRETKEYMATERIAL\n-----END PRIVATE KEY-----";
const CONSUMER_KEY: &str = "3MVG9SENSITIVECONSUMERKEY";
// Public material, so not a secret — but `Debug` still reports only whether it
// is present, because a full certificate in a log line is noise.
const CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----\nCERTBODYMARKER\n-----END CERTIFICATE-----";

fn target() -> TargetOrg {
    TargetOrg {
        my_domain: "https://example.my.salesforce.com".to_owned(),
        consumer_key: CONSUMER_KEY.to_owned(),
        jwt_subject: "admin@example.com".to_owned(),
        private_key_pem: PRIVATE_KEY.to_owned(),
        certificate_pem: Some(CERTIFICATE.to_owned()),
    }
}

#[test]
fn debug_does_not_leak_the_private_key() {
    let rendered = format!("{:?}", target());
    assert!(
        !rendered.contains("SECRETKEYMATERIAL"),
        "private key leaked into Debug output: {rendered}"
    );
    assert!(
        !rendered.contains("BEGIN PRIVATE KEY"),
        "private key PEM leaked into Debug output: {rendered}"
    );
}

#[test]
fn debug_does_not_leak_the_consumer_key() {
    let rendered = format!("{:?}", target());
    assert!(
        !rendered.contains("SENSITIVECONSUMERKEY"),
        "consumer key leaked into Debug output: {rendered}"
    );
}

#[test]
fn debug_still_identifies_which_org_it_is() {
    // Redaction is only useful if what remains is enough to debug with.
    let rendered = format!("{:?}", target());
    assert!(rendered.contains("example.my.salesforce.com"));
    assert!(rendered.contains("admin@example.com"));
    assert!(rendered.contains("<redacted>"));
}
