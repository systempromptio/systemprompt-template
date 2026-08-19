//! `salesforce_org::apply::build_package` — the metadata package `apply` sends.
//!
//! The `<version>` in `package.xml` selects a *schema*, not a URL: it decides
//! which elements a deployed component may carry. Because a metadata deploy is
//! declarative, an element that is in scope at the deployed version and omitted
//! from the package takes its default rather than being left alone — so a wrong
//! version silently rewrites org configuration.
//!
//! Nothing else in the suite pins this. Without these tests a bad version fails
//! only against a live org, which is exactly where it is most expensive.

use systemprompt_web_admin::salesforce_org::apply::{build_package, check_certificate_present};
use systemprompt_web_admin::salesforce_org::client::METADATA_VERSION;
use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    ExternalClientApp, IpRelaxation, OauthSpec, OrgSpec, PolicySpec, Validity, ValidityUnit,
};

// A stand-in PEM. Only its framing matters here — the element carries the
// base64 body with the BEGIN/END lines stripped.
const CERT: &str =
    "-----BEGIN CERTIFICATE-----\nMIIBderived\nQkJCQg==\n-----END CERTIFICATE-----\n";

fn spec() -> OrgSpec {
    OrgSpec {
        external_client_app: ExternalClientApp {
            developer_name: "Systemprompt_SSO".to_owned(),
            label: "Systemprompt SSO".to_owned(),
            description: None,
            contact_email: "ed@systemprompt.io".to_owned(),
            distribution_state: "Local".to_owned(),
            oauth: OauthSpec {
                callback_url: "https://example.test/callback".to_owned(),
                scopes: vec![OauthScope::Api, OauthScope::Mcp],
                first_party_app_enabled: false,
                pkce_required: true,
                consumer_secret_optional: false,
                named_user_jwt: true,
                single_logout_url: None,
            },
            policies: PolicySpec {
                permitted_users: "AdminApprovedPreAuthorized".to_owned(),
                ip_relaxation: IpRelaxation::Enforce,
                refresh_token_policy: "SpecificLifetime".to_owned(),
                refresh_token_validity: Some(Validity {
                    period: 365,
                    unit: ValidityUnit::Days,
                }),
                required_session_level: Some("STANDARD".to_owned()),
            },
        },
        permission_sets: Vec::new(),
        hosted_mcp_servers: Vec::new(),
    }
}

fn file(package: &[(String, String)], suffix: &str) -> String {
    package
        .iter()
        .find(|(path, _)| path.ends_with(suffix))
        .unwrap_or_else(|| panic!("package has no {suffix} file: {package:?}"))
        .1
        .clone()
}

// The deployed schema version is the one the element set was probed against.
#[test]
fn package_xml_declares_the_metadata_version() {
    let package = build_package(&spec(), Some(CERT));
    let manifest = file(&package, "package.xml");

    assert!(
        manifest.contains(&format!("<version>{METADATA_VERSION}</version>")),
        "package.xml must declare <version>{METADATA_VERSION}</version>: {manifest}"
    );
}

// Pinned as a literal as well as against the constant. Asserting only against
// `METADATA_VERSION` would pass for any value, including one nobody probed.
#[test]
fn the_metadata_version_is_the_probed_one() {
    assert_eq!(
        METADATA_VERSION, "67.0",
        "changing this requires re-deriving the accepted element set — see \
         deploy/salesforce/README.md"
    );
}

// The element that keeps JWT-format access tokens switched on. It is in schema
// from 67.0, and the REST metadata deploy this whole tool runs on is the only
// deploy path that accepts those tokens — so a package that omits it disables
// the mechanism used to deploy it.
#[test]
fn global_oauth_settings_carry_named_user_jwt() {
    let package = build_package(&spec(), Some(CERT));
    let global = file(&package, ".ecaGlblOauth");

    assert!(
        global.contains("<isNamedUserJwtEnabled>true</isNamedUserJwtEnabled>"),
        "global OAuth settings must state isNamedUserJwtEnabled explicitly: {global}"
    );
}

// `false` must be emitted as `false`, not omitted. An absent element takes the
// org default, which is not the same as declaring the value off.
#[test]
fn named_user_jwt_false_is_emitted_not_omitted() {
    let mut spec = spec();
    spec.external_client_app.oauth.named_user_jwt = false;
    let package = build_package(&spec, Some(CERT));
    let global = file(&package, ".ecaGlblOauth");

    assert!(
        global.contains("<isNamedUserJwtEnabled>false</isNamedUserJwtEnabled>"),
        "a false value must still be declared: {global}"
    );
}

// The certificate is in schema on `ExtlClntAppGlobalOauthSettings`, and a
// declarative deploy that omits it clears the app's digital signature — which
// is the credential the JWT-bearer grant runs on. It must be in the package.
#[test]
fn global_oauth_settings_carry_the_certificate() {
    let package = build_package(&spec(), Some(CERT));
    let global = file(&package, ".ecaGlblOauth");

    assert!(
        global.contains("<certificate>MIIBderivedQkJCQg==</certificate>"),
        "the PEM body must be emitted with its framing lines stripped: {global}"
    );
}

// Refusing beats deploying. Without a certificate to send, the deploy would
// silently revoke the app's signature and lock the tool out of the org.
#[test]
fn a_missing_certificate_is_refused() {
    assert!(check_certificate_present(None).is_err());
    assert!(check_certificate_present(Some("   ")).is_err());
    assert!(check_certificate_present(Some(CERT)).is_ok());
}
