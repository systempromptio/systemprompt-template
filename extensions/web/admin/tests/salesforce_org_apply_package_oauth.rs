//! `salesforce_org::apply::build_package` — the OAuth settings and policy
//! components.
//!
//! These two components carry the values that decide who may authenticate and
//! for how long. The deploy is declarative, so an optional element emitted when
//! it should be absent is as damaging as one omitted when it should be present:
//! either way the org ends up in a state nobody wrote down.

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
    ExternalClientApp, IpRelaxation, OauthSpec, OrgSpec, PolicySpec, Validity, ValidityUnit,
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

// The scope list is deployed as one comma-separated element in the spec's
// order, using the Metadata API tokens rather than the YAML names.
#[test]
fn scopes_deploy_as_their_metadata_tokens() {
    let oauth = file(&build_package(&spec(), None), ".ecaOauth");
    assert!(
        oauth.contains("<commaSeparatedOauthScopes>Basic,Api,MCP</commaSeparatedOauthScopes>"),
        "{oauth}"
    );
}

#[test]
fn an_empty_scope_set_deploys_as_an_empty_element() {
    let mut spec = spec();
    spec.external_client_app.oauth.scopes.clear();
    let oauth = file(&build_package(&spec, None), ".ecaOauth");
    assert!(
        oauth.contains("<commaSeparatedOauthScopes></commaSeparatedOauthScopes>"),
        "{oauth}"
    );
}

#[test]
fn the_single_logout_url_is_emitted_only_when_set() {
    let oauth = file(&build_package(&spec(), None), ".ecaOauth");
    assert!(!oauth.contains("<singleLogoutUrl>"), "{oauth}");

    let mut spec = spec();
    spec.external_client_app.oauth.single_logout_url = Some("https://example.test/slo".to_owned());
    let oauth = file(&build_package(&spec, None), ".ecaOauth");
    assert!(
        oauth.contains("<singleLogoutUrl>https://example.test/slo</singleLogoutUrl>"),
        "{oauth}"
    );
}

#[test]
fn the_policy_component_carries_the_policy_strings() {
    let policies = file(&build_package(&spec(), None), ".ecaOauthPlcy");
    assert!(
        policies.contains(
            "<permittedUsersPolicyType>AdminApprovedPreAuthorized</permittedUsersPolicyType>"
        ),
        "{policies}"
    );
    assert!(
        policies.contains("<refreshTokenPolicyType>SpecificLifetime</refreshTokenPolicyType>"),
        "{policies}"
    );
}

#[test]
fn every_ip_relaxation_deploys_its_metadata_token() {
    for relaxation in [
        IpRelaxation::Enforce,
        IpRelaxation::Bypass,
        IpRelaxation::Bypass2Factor,
        IpRelaxation::EnforceRelaxRefresh,
    ] {
        let mut spec = spec();
        spec.external_client_app.policies.ip_relaxation = relaxation;
        let policies = file(&build_package(&spec, None), ".ecaOauthPlcy");
        assert!(
            policies.contains(&format!(
                "<ipRelaxationPolicyType>{}</ipRelaxationPolicyType>",
                relaxation.metadata_token()
            )),
            "{relaxation:?}: {policies}"
        );
    }
}

// Both halves or neither. A period without its unit is not a value the API
// accepts.
#[test]
fn refresh_token_validity_deploys_as_a_pair() {
    let policies = file(&build_package(&spec(), None), ".ecaOauthPlcy");
    assert!(
        !policies.contains("<refreshTokenValidityPeriod>"),
        "{policies}"
    );
    assert!(
        !policies.contains("<refreshTokenValidityUnit>"),
        "{policies}"
    );

    let mut spec = spec();
    spec.external_client_app.policies.refresh_token_validity = Some(Validity {
        period: 8760,
        unit: ValidityUnit::Hours,
    });
    let policies = file(&build_package(&spec, None), ".ecaOauthPlcy");
    assert!(
        policies.contains("<refreshTokenValidityPeriod>8760</refreshTokenValidityPeriod>"),
        "{policies}"
    );
    assert!(
        policies.contains("<refreshTokenValidityUnit>Hours</refreshTokenValidityUnit>"),
        "{policies}"
    );
}

#[test]
fn the_required_session_level_is_emitted_only_when_set() {
    let policies = file(&build_package(&spec(), None), ".ecaOauthPlcy");
    assert!(!policies.contains("<requiredSessionLevel>"), "{policies}");

    let mut spec = spec();
    spec.external_client_app.policies.required_session_level = Some("HIGH_ASSURANCE".to_owned());
    let policies = file(&build_package(&spec, None), ".ecaOauthPlcy");
    assert!(
        policies.contains("<requiredSessionLevel>HIGH_ASSURANCE</requiredSessionLevel>"),
        "{policies}"
    );
}

// Building the same spec twice must produce the same bytes, or a re-run of
// apply looks like a change.
#[test]
fn the_package_is_deterministic() {
    assert_eq!(
        build_package(&spec(), Some("QkJCQg==")),
        build_package(&spec(), Some("QkJCQg=="))
    );
}
