//! `salesforce_org::spec` — the desired-state document.
//!
//! Every default in this struct is a value that gets *deployed*. The metadata
//! deploy is declarative, so a field omitted from `org.yaml` and defaulted to
//! the wrong thing rewrites org configuration on the next apply rather than
//! leaving it alone. These tests pin the defaults, the strictness that rejects
//! a typo'd key, and the YAML round-trip.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    IpRelaxation, OrgSpec, SPEC_RELATIVE_PATH, Validity, ValidityUnit,
};

const MINIMAL: &str = r"external_client_app:
  developer_name: Systemprompt_SSO
  label: Systemprompt SSO
  contact_email: ed@systemprompt.io
  distribution_state: Local
  oauth:
    callback_url: https://example.test/callback
    scopes:
      - api
      - mcp
  policies:
    permitted_users: AdminApprovedPreAuthorized
    ip_relaxation: Enforce
    refresh_token_policy: SpecificLifetime
";

const FULL: &str = r"external_client_app:
  developer_name: Systemprompt_SSO
  label: Systemprompt SSO
  description: Systemprompt <=> Astound
  contact_email: ed@systemprompt.io
  distribution_state: Local
  oauth:
    callback_url: https://example.test/callback
    scopes:
      - basic
      - refresh_token
      - open_id
      - mcp
    first_party_app_enabled: true
    pkce_required: false
    consumer_secret_optional: true
    named_user_jwt: false
    single_logout_url: https://example.test/logout
  policies:
    permitted_users: AdminApprovedPreAuthorized
    ip_relaxation: Bypass_2factor
    refresh_token_policy: SpecificLifetime
    refresh_token_validity:
      period: 365
      unit: Days
    required_session_level: STANDARD
permission_sets:
  - name: Salesforce_MCP_Access
    label: Salesforce MCP Access
    description: Grants the app
    grants_app: Systemprompt_SSO
hosted_mcp_servers:
  - name: sobject-all
    developer_name: platform_sobject_all
    endpoint: https://api.salesforce.com/platform/mcp/v1/platform/sobject-all
";

fn parse(yaml: &str) -> OrgSpec {
    serde_yaml::from_str(yaml).unwrap_or_else(|e| panic!("spec should parse: {e}"))
}

#[test]
fn the_spec_path_is_relative_to_the_services_root() {
    assert_eq!(SPEC_RELATIVE_PATH, "salesforce/org.yaml");
}

#[test]
fn a_minimal_spec_parses() {
    let spec = parse(MINIMAL);
    assert_eq!(spec.external_client_app.developer_name, "Systemprompt_SSO");
    assert_eq!(spec.external_client_app.label, "Systemprompt SSO");
    assert_eq!(spec.external_client_app.contact_email, "ed@systemprompt.io");
    assert_eq!(spec.external_client_app.distribution_state, "Local");
    assert_eq!(
        spec.external_client_app.oauth.scopes,
        vec![OauthScope::Api, OauthScope::Mcp]
    );
}

#[test]
fn omitted_collections_default_to_empty() {
    let spec = parse(MINIMAL);
    assert!(spec.permission_sets.is_empty());
    assert!(spec.hosted_mcp_servers.is_empty());
}

#[test]
fn omitted_optionals_default_to_none() {
    let spec = parse(MINIMAL);
    assert!(spec.external_client_app.description.is_none());
    assert!(spec.external_client_app.oauth.single_logout_url.is_none());
    assert!(
        spec.external_client_app
            .policies
            .refresh_token_validity
            .is_none()
    );
    assert!(
        spec.external_client_app
            .policies
            .required_session_level
            .is_none()
    );
}

// Both default *on*. They are emitted on every deploy, so defaulting either
// to `false` would silently disable PKCE or the JWT-format access tokens the
// REST metadata deploy runs on.
#[test]
fn pkce_and_named_user_jwt_default_on() {
    let oauth = parse(MINIMAL).external_client_app.oauth;
    assert!(oauth.pkce_required);
    assert!(oauth.named_user_jwt);
}

#[test]
fn the_permissive_flags_default_off() {
    let oauth = parse(MINIMAL).external_client_app.oauth;
    assert!(!oauth.first_party_app_enabled);
    assert!(!oauth.consumer_secret_optional);
}

#[test]
fn a_hosted_mcp_server_defaults_to_active() {
    let spec = parse(FULL);
    assert_eq!(spec.hosted_mcp_servers.len(), 1);
    assert!(spec.hosted_mcp_servers[0].active);
    assert_eq!(spec.hosted_mcp_servers[0].name, "sobject-all");
    assert_eq!(
        spec.hosted_mcp_servers[0].developer_name,
        "platform_sobject_all"
    );
}

#[test]
fn every_declared_field_parses() {
    let spec = parse(FULL);
    let app = &spec.external_client_app;
    assert_eq!(app.description.as_deref(), Some("Systemprompt <=> Astound"));
    assert!(app.oauth.first_party_app_enabled);
    assert!(!app.oauth.pkce_required);
    assert!(app.oauth.consumer_secret_optional);
    assert!(!app.oauth.named_user_jwt);
    assert_eq!(
        app.oauth.single_logout_url.as_deref(),
        Some("https://example.test/logout")
    );
    assert_eq!(app.policies.ip_relaxation, IpRelaxation::Bypass2Factor);
    assert_eq!(
        app.policies.refresh_token_validity,
        Some(Validity {
            period: 365,
            unit: ValidityUnit::Days,
        })
    );
    assert_eq!(
        app.policies.required_session_level.as_deref(),
        Some("STANDARD")
    );
    assert_eq!(spec.permission_sets[0].name, "Salesforce_MCP_Access");
    assert_eq!(
        spec.permission_sets[0].description.as_deref(),
        Some("Grants the app")
    );
    assert_eq!(
        spec.permission_sets[0].grants_app.as_deref(),
        Some("Systemprompt_SSO")
    );
}

#[test]
fn a_permission_set_without_a_grant_parses() {
    let yaml = format!("{MINIMAL}permission_sets:\n  - name: Plain\n    label: Plain Set\n");
    let spec = parse(&yaml);
    assert!(spec.permission_sets[0].grants_app.is_none());
    assert!(spec.permission_sets[0].description.is_none());
}

// A misspelled key must fail loudly. Silently ignoring it would deploy the
// default for the field the operator thought they had set.
#[test]
fn an_unknown_key_is_rejected() {
    let cases = [
        format!("{MINIMAL}unexpected_root: true\n"),
        MINIMAL.replace("  label: Systemprompt SSO", "  labell: Systemprompt SSO"),
        MINIMAL.replace("    callback_url:", "    callbackUrl:"),
    ];
    for yaml in cases {
        assert!(
            serde_yaml::from_str::<OrgSpec>(&yaml).is_err(),
            "unknown keys must be rejected: {yaml}"
        );
    }
}

#[test]
fn a_missing_required_field_is_rejected() {
    let yaml = MINIMAL.replace("  contact_email: ed@systemprompt.io\n", "");
    assert!(serde_yaml::from_str::<OrgSpec>(&yaml).is_err());
}

#[test]
fn an_unknown_scope_is_rejected() {
    let yaml = MINIMAL.replace("      - mcp", "      - telepathy");
    assert!(serde_yaml::from_str::<OrgSpec>(&yaml).is_err());
}

// The scope vocabulary is snake_case in YAML even where Salesforce's own
// tokens are not.
#[test]
fn scopes_use_the_snake_case_vocabulary() {
    let replacement = "      - open_id\n      - refresh_token\n      - einstein_gpt\n      - data_cloud_user_claims";
    let yaml = MINIMAL.replace("      - api\n      - mcp", replacement);
    assert_eq!(
        parse(&yaml).external_client_app.oauth.scopes,
        vec![
            OauthScope::OpenId,
            OauthScope::RefreshToken,
            OauthScope::EinsteinGpt,
            OauthScope::DataCloudUserClaims,
        ]
    );
}

#[test]
fn ip_relaxation_accepts_every_salesforce_token() {
    let cases = [
        ("Enforce", IpRelaxation::Enforce),
        ("Bypass", IpRelaxation::Bypass),
        ("Bypass_2factor", IpRelaxation::Bypass2Factor),
        ("Enforce_relaxrefresh", IpRelaxation::EnforceRelaxRefresh),
    ];
    for (token, expected) in cases {
        let yaml = MINIMAL.replace("ip_relaxation: Enforce", &format!("ip_relaxation: {token}"));
        assert_eq!(
            parse(&yaml).external_client_app.policies.ip_relaxation,
            expected,
            "{token}"
        );
        assert_eq!(expected.metadata_token(), token);
    }
}

#[test]
fn validity_units_round_trip_their_metadata_tokens() {
    for (token, unit) in [
        ("Hours", ValidityUnit::Hours),
        ("Days", ValidityUnit::Days),
        ("Months", ValidityUnit::Months),
    ] {
        assert_eq!(unit.metadata_token(), token);
        let yaml = FULL.replace("unit: Days", &format!("unit: {token}"));
        assert_eq!(
            parse(&yaml)
                .external_client_app
                .policies
                .refresh_token_validity
                .map(|v| v.unit),
            Some(unit)
        );
    }
}
