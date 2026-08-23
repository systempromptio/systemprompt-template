//! `salesforce_org::export` — turning query results back into an [`OrgSpec`].
//!
//! Export is the left-hand side of every diff, so a field it reads wrongly
//! shows up as drift the operator is invited to "fix" by applying. Four fields
//! are worse still: no API exposes them, so export carries them from a baseline
//! and inventing a value there would deploy it.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::salesforce_org::export::{
    UNREADABLE_PLACEHOLDER, oauth_from_settings, policies_from_record,
};
use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    ExternalClientApp, IpRelaxation, OauthSpec, OrgSpec, PolicySpec, Validity, ValidityUnit,
};

fn baseline() -> OrgSpec {
    OrgSpec {
        external_client_app: ExternalClientApp {
            developer_name: "Systemprompt_SSO".to_owned(),
            label: "Systemprompt SSO".to_owned(),
            description: None,
            contact_email: "ed@systemprompt.io".to_owned(),
            distribution_state: "Local".to_owned(),
            oauth: OauthSpec {
                callback_url: "https://example.test/callback".to_owned(),
                scopes: vec![OauthScope::Api],
                first_party_app_enabled: false,
                pkce_required: false,
                consumer_secret_optional: true,
                named_user_jwt: false,
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

#[test]
fn enabled_columns_become_scopes() {
    let settings = serde_json::json!({
        "OauthScopesAPI": true,
        "OauthScopesMCP_API": true,
        "OauthScopesWEB": false,
    });
    let oauth = oauth_from_settings(&settings, None);
    assert_eq!(oauth.scopes, vec![OauthScope::Api, OauthScope::Mcp]);
}

#[test]
fn scopes_come_back_in_the_declared_order() {
    let settings = serde_json::json!({
        "OauthScopesMCP_API": true,
        "OauthScopesSSO": true,
        "OauthScopesAPI": true,
    });
    let oauth = oauth_from_settings(&settings, None);
    assert_eq!(
        oauth.scopes,
        vec![OauthScope::Basic, OauthScope::Api, OauthScope::Mcp]
    );
}

#[test]
fn an_absent_column_is_not_a_scope() {
    let oauth = oauth_from_settings(&serde_json::json!({}), None);
    assert!(oauth.scopes.is_empty());
}

// A column reported as a string rather than a boolean must not read as
// enabled. Coercing it would invent a grant.
#[test]
fn a_non_boolean_column_is_not_a_scope() {
    let settings = serde_json::json!({ "OauthScopesAPI": "true", "OauthScopesMCP_API": null });
    assert!(oauth_from_settings(&settings, None).scopes.is_empty());
}

// `OauthScopesHUB_API` has no metadata token. It is warned about, never
// represented — a spec cannot carry it, so it must not become a scope.
#[test]
fn the_unmapped_column_never_becomes_a_scope() {
    let settings = serde_json::json!({ "OauthScopesHUB_API": true });
    assert!(oauth_from_settings(&settings, None).scopes.is_empty());
}

#[test]
fn the_first_party_flag_is_read_from_the_record() {
    let on = serde_json::json!({ "ExtlClntAppOauthOptionsFirstPartyAppEnabled": true });
    assert!(oauth_from_settings(&on, None).first_party_app_enabled);
    assert!(!oauth_from_settings(&serde_json::json!({}), None).first_party_app_enabled);
}

#[test]
fn the_single_logout_url_is_read_from_the_record() {
    let record = serde_json::json!({ "SingleLogoutUrl": "https://example.test/slo" });
    assert_eq!(
        oauth_from_settings(&record, None)
            .single_logout_url
            .as_deref(),
        Some("https://example.test/slo")
    );
}

// Salesforce returns an unset text field as `""` as often as `null`. Treating
// the empty string as a value would diff a URL against nothing.
#[test]
fn an_empty_logout_url_is_absent_not_empty() {
    let record = serde_json::json!({ "SingleLogoutUrl": "" });
    assert!(
        oauth_from_settings(&record, None)
            .single_logout_url
            .is_none()
    );
}

// Without a baseline the callback URL is deliberately unusable rather than
// plausible: applying the placeholder fails Salesforce's URL validation
// instead of quietly pointing the org somewhere wrong.
#[test]
fn the_unreadable_callback_url_is_a_refusal_not_a_guess() {
    let oauth = oauth_from_settings(&serde_json::json!({}), None);
    assert_eq!(oauth.callback_url, UNREADABLE_PLACEHOLDER);
    assert!(!UNREADABLE_PLACEHOLDER.starts_with("http"));
}

// With no baseline the two safety flags come back on, so an unattended export
// cannot be applied into an org that then stops requiring PKCE or issuing the
// JWT-format tokens the deploy path depends on.
#[test]
fn unreadable_flags_default_to_the_safe_side() {
    let oauth = oauth_from_settings(&serde_json::json!({}), None);
    assert!(oauth.pkce_required);
    assert!(oauth.named_user_jwt);
    assert!(!oauth.consumer_secret_optional);
}

#[test]
fn the_baseline_supplies_every_unreadable_field() {
    let baseline = baseline();
    let oauth = oauth_from_settings(&serde_json::json!({}), Some(&baseline));
    assert_eq!(oauth.callback_url, "https://example.test/callback");
    assert!(!oauth.pkce_required);
    assert!(!oauth.named_user_jwt);
    assert!(oauth.consumer_secret_optional);
}

// The baseline supplies only what cannot be read. Its scopes are not carried
// forward — that would report the org as compliant with a spec it has drifted
// from.
#[test]
fn the_baseline_does_not_override_readable_fields() {
    let baseline = baseline();
    let settings = serde_json::json!({ "OauthScopesMCP_API": true });
    let oauth = oauth_from_settings(&settings, Some(&baseline));
    assert_eq!(oauth.scopes, vec![OauthScope::Mcp]);
}

#[test]
fn policy_strings_are_read_from_the_record() {
    let record = serde_json::json!({
        "PermittedUsersPolicyType": "AdminApprovedPreAuthorized",
        "RefreshTokenPolicyType": "SpecificLifetime",
        "RequiredSessionLevel": "HIGH_ASSURANCE",
    });
    let policies = policies_from_record(&record);
    assert_eq!(policies.permitted_users, "AdminApprovedPreAuthorized");
    assert_eq!(policies.refresh_token_policy, "SpecificLifetime");
    assert_eq!(
        policies.required_session_level.as_deref(),
        Some("HIGH_ASSURANCE")
    );
}

#[test]
fn absent_policy_strings_are_empty_rather_than_invented() {
    let policies = policies_from_record(&serde_json::json!({}));
    assert!(policies.permitted_users.is_empty());
    assert!(policies.refresh_token_policy.is_empty());
    assert!(policies.required_session_level.is_none());
}

#[test]
fn every_ip_relaxation_token_is_recognised() {
    let cases = [
        ("Enforce", IpRelaxation::Enforce),
        ("Bypass", IpRelaxation::Bypass),
        ("Bypass_2factor", IpRelaxation::Bypass2Factor),
        ("Enforce_relaxrefresh", IpRelaxation::EnforceRelaxRefresh),
    ];
    for (token, expected) in cases {
        let record = serde_json::json!({ "IpRelaxationPolicyType": token });
        assert_eq!(policies_from_record(&record).ip_relaxation, expected);
    }
}

// An unknown or absent value reads as `Enforce`, the strict end. Guessing a
// bypass would present the org as more permissive than it is.
#[test]
fn an_unrecognised_ip_relaxation_falls_back_to_enforce() {
    for value in [
        serde_json::json!({ "IpRelaxationPolicyType": "Something_New" }),
        serde_json::json!({ "IpRelaxationPolicyType": null }),
        serde_json::json!({}),
    ] {
        assert_eq!(
            policies_from_record(&value).ip_relaxation,
            IpRelaxation::Enforce
        );
    }
}

#[test]
fn refresh_token_validity_needs_both_halves() {
    let record = serde_json::json!({
        "RefreshTokenValidityPeriod": 8760,
        "RefreshTokenValidityUnit": "Hours",
    });
    assert_eq!(
        policies_from_record(&record).refresh_token_validity,
        Some(Validity {
            period: 8760,
            unit: ValidityUnit::Hours,
        })
    );
}

#[test]
fn every_validity_unit_is_recognised() {
    for (token, unit) in [
        ("Hours", ValidityUnit::Hours),
        ("Days", ValidityUnit::Days),
        ("Months", ValidityUnit::Months),
    ] {
        let record = serde_json::json!({
            "RefreshTokenValidityPeriod": 1,
            "RefreshTokenValidityUnit": token,
        });
        assert_eq!(
            policies_from_record(&record)
                .refresh_token_validity
                .map(|v| v.unit),
            Some(unit)
        );
    }
}

// Half a validity is no validity. Defaulting the missing half would diff a
// value the org never reported.
#[test]
fn a_half_reported_validity_is_none() {
    let cases = [
        serde_json::json!({ "RefreshTokenValidityPeriod": 365 }),
        serde_json::json!({ "RefreshTokenValidityUnit": "Days" }),
        serde_json::json!({ "RefreshTokenValidityPeriod": 365, "RefreshTokenValidityUnit": "Years" }),
        serde_json::json!({ "RefreshTokenValidityPeriod": -1, "RefreshTokenValidityUnit": "Days" }),
        serde_json::json!({}),
    ];
    for record in cases {
        assert!(
            policies_from_record(&record)
                .refresh_token_validity
                .is_none(),
            "{record}"
        );
    }
}

#[test]
fn a_period_beyond_u32_is_not_truncated() {
    let record = serde_json::json!({
        "RefreshTokenValidityPeriod": 4_294_967_296_u64,
        "RefreshTokenValidityUnit": "Days",
    });
    assert!(
        policies_from_record(&record)
            .refresh_token_validity
            .is_none()
    );
}
