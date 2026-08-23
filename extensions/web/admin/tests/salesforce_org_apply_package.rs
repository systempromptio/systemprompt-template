//! `salesforce_org::apply::build_package` — the component set and the values
//! inside it.
//!
//! A metadata deploy is declarative and this package is hand-built XML, so both
//! halves are load-bearing: a component whose file name or manifest member does
//! not line up is rejected outright, and an element that is in scope but absent
//! takes its default rather than being left alone. The suite already pins the
//! schema version, `isNamedUserJwtEnabled` and the certificate; these tests
//! cover the derived names, the escaping and the optional elements.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::salesforce_org::apply::build_package;
use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    ExternalClientApp, IpRelaxation, OauthSpec, OrgSpec, PolicySpec,
};

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
                scopes: vec![OauthScope::Basic, OauthScope::Api, OauthScope::Mcp],
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
                refresh_token_validity: None,
                required_session_level: None,
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
        .unwrap_or_else(|| panic!("package has no {suffix} file"))
        .1
        .clone()
}

fn paths(package: &[(String, String)]) -> Vec<String> {
    package.iter().map(|(path, _)| path.clone()).collect()
}

// All four components plus the manifest. A deploy missing one of these is not
// a partial deploy — the Metadata API rejects the package.
#[test]
fn the_package_carries_the_manifest_and_four_components() {
    let package = build_package(&spec(), None);
    assert_eq!(package.len(), 5, "{:?}", paths(&package));
}

// The dependent components' names are derived from the app's developer name.
// Salesforce matches them by name, so a changed suffix creates a second,
// unlinked component rather than updating the first.
#[test]
fn component_paths_are_derived_from_the_developer_name() {
    let package = build_package(&spec(), None);
    assert_eq!(
        paths(&package),
        vec![
            "package.xml".to_owned(),
            "externalClientApps/Systemprompt_SSO.eca".to_owned(),
            "extlClntAppGlobalOauthSets/Systemprompt_SSO_glbloauth.ecaGlblOauth".to_owned(),
            "extlClntAppOauthSettings/Systemprompt_SSO_oauth.ecaOauth".to_owned(),
            "extlClntAppOauthPolicies/Systemprompt_SSO_oauthPlcy.ecaOauthPlcy".to_owned(),
        ]
    );
}

// Every manifest member must name a file actually in the package, or the
// deploy fails on a component it was told to expect.
#[test]
fn the_manifest_names_every_component() {
    let package = build_package(&spec(), None);
    let manifest = file(&package, "package.xml");
    for (type_name, member) in [
        ("ExternalClientApplication", "Systemprompt_SSO"),
        (
            "ExtlClntAppGlobalOauthSettings",
            "Systemprompt_SSO_glbloauth",
        ),
        ("ExtlClntAppOauthSettings", "Systemprompt_SSO_oauth"),
        (
            "ExtlClntAppOauthConfigurablePolicies",
            "Systemprompt_SSO_oauthPlcy",
        ),
    ] {
        assert!(
            manifest.contains(&format!("<name>{type_name}</name>")),
            "manifest omits {type_name}: {manifest}"
        );
        assert!(
            manifest.contains(&format!("<members>{member}</members>")),
            "manifest omits member {member}: {manifest}"
        );
    }
}

#[test]
fn every_component_declares_the_metadata_namespace() {
    let package = build_package(&spec(), None);
    for (path, body) in &package {
        assert!(
            body.contains("xmlns=\"http://soap.sforce.com/2006/04/metadata\""),
            "{path} is missing the metadata namespace"
        );
        assert!(
            body.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n"),
            "{path} is missing its XML declaration"
        );
    }
}

// Each dependent component points back at the app by name. Without it the
// component deploys unattached to anything.
#[test]
fn each_dependent_component_names_its_app() {
    let package = build_package(&spec(), None);
    for suffix in [".ecaGlblOauth", ".ecaOauth", ".ecaOauthPlcy"] {
        let body = file(&package, suffix);
        assert!(
            body.contains(
                "<externalClientApplication>Systemprompt_SSO</externalClientApplication>"
            ),
            "{suffix} does not reference the app: {body}"
        );
    }
}

#[test]
fn the_app_component_carries_its_identity_fields() {
    let package = build_package(&spec(), None);
    let eca = file(&package, ".eca");
    assert!(
        eca.contains("<contactEmail>ed@systemprompt.io</contactEmail>"),
        "{eca}"
    );
    assert!(
        eca.contains("<distributionState>Local</distributionState>"),
        "{eca}"
    );
    assert!(eca.contains("<label>Systemprompt SSO</label>"), "{eca}");
}

#[test]
fn an_absent_description_is_omitted() {
    let package = build_package(&spec(), None);
    assert!(!file(&package, ".eca").contains("<description>"));
}

#[test]
fn a_present_description_is_emitted() {
    let mut spec = spec();
    spec.external_client_app.description = Some("Astound bridge".to_owned());
    let package = build_package(&spec, None);
    assert!(
        file(&package, ".eca").contains("<description>Astound bridge</description>"),
        "{:?}",
        file(&package, ".eca")
    );
}

// An unescaped `&` or `<` makes the whole package unparseable, and these are
// operator-supplied strings.
#[test]
fn markup_characters_are_escaped() {
    let mut spec = spec();
    spec.external_client_app.label = "Systemprompt & Astound".to_owned();
    spec.external_client_app.description = Some("a <b> c".to_owned());
    let package = build_package(&spec, None);
    let eca = file(&package, ".eca");
    assert!(
        eca.contains("<label>Systemprompt &amp; Astound</label>"),
        "{eca}"
    );
    assert!(
        eca.contains("<description>a &lt;b&gt; c</description>"),
        "{eca}"
    );
    assert!(!eca.contains("<b>"), "{eca}");
}

#[test]
fn manifest_members_are_escaped_too() {
    let mut spec = spec();
    spec.external_client_app.developer_name = "A&B".to_owned();
    let manifest = file(&build_package(&spec, None), "package.xml");
    assert!(
        manifest.contains("<members>A&amp;B</members>"),
        "{manifest}"
    );
    assert!(
        manifest.contains("<members>A&amp;B_glbloauth</members>"),
        "{manifest}"
    );
}

#[test]
fn the_callback_url_is_deployed_verbatim() {
    let package = build_package(&spec(), None);
    assert!(
        file(&package, ".ecaGlblOauth")
            .contains("<callbackUrl>https://example.test/callback</callbackUrl>")
    );
}

// Salesforce compares the callback character for character, so a URL with a
// query string must not be reformatted or its `&` left raw.
#[test]
fn a_callback_url_with_a_query_string_survives() {
    let mut spec = spec();
    spec.external_client_app.oauth.callback_url = "https://example.test/cb?a=1&b=2".to_owned();
    let global = file(&build_package(&spec, None), ".ecaGlblOauth");
    assert!(
        global.contains("<callbackUrl>https://example.test/cb?a=1&amp;b=2</callbackUrl>"),
        "{global}"
    );
}

// Without a certificate the element is absent rather than empty. An empty one
// would clear the app's signature just as effectively.
#[test]
fn no_certificate_means_no_certificate_element() {
    let global = file(&build_package(&spec(), None), ".ecaGlblOauth");
    assert!(!global.contains("<certificate>"), "{global}");
}

// The operator has a PEM on disk next to the private key; a bare base64 body
// is accepted unchanged so either form works.
#[test]
fn a_bare_certificate_body_passes_through() {
    let global = file(&build_package(&spec(), Some("QkJCQg==")), ".ecaGlblOauth");
    assert!(
        global.contains("<certificate>QkJCQg==</certificate>"),
        "{global}"
    );
}

#[test]
fn a_multi_line_pem_is_joined_without_its_framing() {
    let pem = "-----BEGIN CERTIFICATE-----\nAAAA\nBBBB\n-----END CERTIFICATE-----\n";
    let global = file(&build_package(&spec(), Some(pem)), ".ecaGlblOauth");
    assert!(
        global.contains("<certificate>AAAABBBB</certificate>"),
        "{global}"
    );
}

// The order is the one the Metadata API accepted. It is not alphabetical by
// accident — reordering these has been rejected by the API.
#[test]
fn global_oauth_elements_keep_their_accepted_order() {
    let global = file(&build_package(&spec(), Some("QkJCQg==")), ".ecaGlblOauth");
    let order = [
        "<callbackUrl>",
        "<certificate>",
        "<externalClientApplication>",
        "<isConsumerSecretOptional>",
        "<isNamedUserJwtEnabled>",
        "<isPkceRequired>",
        "<label>",
    ];
    let mut last = 0;
    for element in order {
        let at = global
            .find(element)
            .unwrap_or_else(|| panic!("{element} is missing: {global}"));
        assert!(at > last, "{element} is out of order: {global}");
        last = at;
    }
}

#[test]
fn the_boolean_oauth_flags_are_always_declared() {
    let mut spec = spec();
    spec.external_client_app.oauth.pkce_required = false;
    spec.external_client_app.oauth.consumer_secret_optional = true;
    spec.external_client_app.oauth.first_party_app_enabled = true;
    let package = build_package(&spec, None);
    let global = file(&package, ".ecaGlblOauth");
    assert!(
        global.contains("<isPkceRequired>false</isPkceRequired>"),
        "{global}"
    );
    assert!(
        global.contains("<isConsumerSecretOptional>true</isConsumerSecretOptional>"),
        "{global}"
    );
    assert!(
        file(&package, ".ecaOauth")
            .contains("<isFirstPartyAppEnabled>true</isFirstPartyAppEnabled>")
    );
}
