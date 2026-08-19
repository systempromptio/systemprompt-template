//! `salesforce_org::diff` — drift detection between a desired org spec and the
//! spec exported from a live org.
//!
//! The distinction these tests pin down is between *drift* (a readable field
//! that disagrees) and *always-applied* (a field no API exposes, which is
//! deployed unconditionally). Folding the second into "no changes" would make
//! the tool claim it verified something it never read.

use systemprompt_web_admin::salesforce_org::diff::{ChangeKind, diff};
use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    ExternalClientApp, HostedMcpServer, IpRelaxation, OauthSpec, OrgSpec, PermissionSetSpec,
    PolicySpec, Validity, ValidityUnit,
};

fn spec() -> OrgSpec {
    OrgSpec {
        external_client_app: ExternalClientApp {
            developer_name: "Systemprompt_SSO".to_owned(),
            label: "Systemprompt SSO".to_owned(),
            description: Some("Systemprompt <=> Astound".to_owned()),
            contact_email: "ed@systemprompt.io".to_owned(),
            distribution_state: "Local".to_owned(),
            oauth: OauthSpec {
                callback_url: "https://example.test/callback".to_owned(),
                scopes: vec![
                    OauthScope::Basic,
                    OauthScope::Api,
                    OauthScope::RefreshToken,
                    OauthScope::OpenId,
                    OauthScope::Mcp,
                ],
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
        permission_sets: vec![PermissionSetSpec {
            name: "Salesforce_MCP_Access".to_owned(),
            label: "Salesforce MCP Access".to_owned(),
            description: None,
            grants_app: Some("Systemprompt_SSO".to_owned()),
        }],
        hosted_mcp_servers: Vec::new(),
    }
}

#[test]
fn identical_specs_report_no_drift() {
    let changes = diff(&spec(), &spec());
    assert!(changes.is_clean(), "drift: {:?}", changes.drift());
}

#[test]
fn unreadable_fields_are_reported_but_are_not_drift() {
    let changes = diff(&spec(), &spec());
    let always: Vec<_> = changes
        .changes
        .iter()
        .filter(|c| c.kind == ChangeKind::AlwaysApplied)
        .map(|c| c.path.as_str())
        .collect();

    // These three live on ExtlClntAppGlobalOauthSettings, which is not a
    // queryable sObject — export cannot read them back to compare.
    assert!(always.contains(&"external_client_app.oauth.callback_url"));
    assert!(always.contains(&"external_client_app.oauth.pkce_required"));
    assert!(always.contains(&"external_client_app.oauth.consumer_secret_optional"));
    assert!(changes.is_clean());
}

#[test]
fn a_scope_only_the_org_has_is_a_removal() {
    // The live dev org grants `full`; the committed spec deliberately does not.
    let mut actual = spec();
    actual
        .external_client_app
        .oauth
        .scopes
        .push(OauthScope::Full);

    let changes = diff(&actual, &spec());
    let drift = changes.drift();
    assert_eq!(drift.len(), 1);
    assert_eq!(drift[0].kind, ChangeKind::Remove);
    assert_eq!(drift[0].actual, "Full");
}

#[test]
fn a_scope_only_the_spec_has_is_an_addition() {
    let mut actual = spec();
    actual
        .external_client_app
        .oauth
        .scopes
        .retain(|s| *s != OauthScope::Mcp);

    let changes = diff(&actual, &spec());
    let drift = changes.drift();
    assert_eq!(drift.len(), 1);
    assert_eq!(drift[0].kind, ChangeKind::Add);
    assert_eq!(drift[0].desired, "MCP");
}

#[test]
fn scope_order_does_not_create_phantom_drift() {
    let mut actual = spec();
    actual.external_client_app.oauth.scopes.reverse();
    assert!(diff(&actual, &spec()).is_clean());
}

#[test]
fn refresh_token_validity_change_is_drift() {
    // The dev org reads 8760 Days; the spec sets 365.
    let mut actual = spec();
    actual.external_client_app.policies.refresh_token_validity = Some(Validity {
        period: 8760,
        unit: ValidityUnit::Days,
    });

    let drift = diff(&actual, &spec());
    let drift = drift.drift();
    assert_eq!(drift.len(), 1);
    assert_eq!(drift[0].actual, "8760 Days");
    assert_eq!(drift[0].desired, "365 Days");
}

#[test]
fn the_two_intended_dev_org_deltas_are_exactly_two_changes() {
    // The state the dev org is actually in today, against the committed spec.
    let mut actual = spec();
    actual
        .external_client_app
        .oauth
        .scopes
        .push(OauthScope::Full);
    actual.external_client_app.policies.refresh_token_validity = Some(Validity {
        period: 8760,
        unit: ValidityUnit::Days,
    });

    let changes = diff(&actual, &spec());
    assert_eq!(
        changes.drift().len(),
        2,
        "expected exactly the full-scope removal and the validity change: {:?}",
        changes.drift()
    );
}

#[test]
fn a_missing_permission_set_is_an_addition() {
    let mut actual = spec();
    actual.permission_sets.clear();

    let changes = diff(&actual, &spec());
    let drift = changes.drift();
    assert_eq!(drift.len(), 1);
    assert_eq!(drift[0].kind, ChangeKind::Add);
    assert_eq!(drift[0].desired, "Salesforce_MCP_Access");
}

#[test]
fn a_permission_set_that_lost_its_app_grant_is_drift() {
    let mut actual = spec();
    if let Some(first) = actual.permission_sets.first_mut() {
        first.grants_app = None;
    }

    let changes = diff(&actual, &spec());
    let drift = changes.drift();
    assert_eq!(drift.len(), 1);
    assert_eq!(
        drift[0].path,
        "permission_sets.Salesforce_MCP_Access.grants_app"
    );
}

fn sobject_all(active: bool) -> HostedMcpServer {
    HostedMcpServer {
        name: "sobject-all".to_owned(),
        developer_name: "platform_sobject_all".to_owned(),
        endpoint: "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all".to_owned(),
        active,
    }
}

// A server switched off in the org is drift, not a note. `apply` PATCHes
// `McpServerAccess.Active` back on, so reporting it as "always applied" would
// hide a change the tool can and will make.
#[test]
fn an_inactive_hosted_mcp_server_is_drift() {
    let mut desired = spec();
    desired.hosted_mcp_servers = vec![sobject_all(true)];
    let mut actual = desired.clone();
    actual.hosted_mcp_servers = vec![sobject_all(false)];

    let drift = diff(&actual, &desired);
    let changes = drift.drift();
    assert_eq!(changes.len(), 1, "expected exactly one delta: {changes:?}");
    assert_eq!(
        changes[0].path,
        "hosted_mcp_servers.platform_sobject_all.active"
    );
    assert_eq!(changes[0].kind, ChangeKind::Update);
    assert_eq!(changes[0].actual, "false");
    assert_eq!(changes[0].desired, "true");
}

// Apply is additive: it never deactivates. A server the org has active but the
// spec does not mention is therefore not drift — calling it drift would report
// a difference no apply will ever resolve.
#[test]
fn an_unmanaged_active_server_is_not_drift() {
    let desired = spec();
    let mut actual = desired.clone();
    actual.hosted_mcp_servers = vec![HostedMcpServer {
        name: "engagement-interaction".to_owned(),
        developer_name: "industries_engagement_interaction".to_owned(),
        endpoint: "https://example.test/engagement".to_owned(),
        active: true,
    }];

    assert!(diff(&actual, &desired).is_clean());
}

// A server the spec names but the org does not offer cannot be fixed by
// activation, so it surfaces as an actionable add rather than being skipped.
#[test]
fn a_server_missing_from_the_org_is_reported() {
    let mut desired = spec();
    desired.hosted_mcp_servers = vec![sobject_all(true)];
    let mut actual = desired.clone();
    actual.hosted_mcp_servers = Vec::new();

    let drift = diff(&actual, &desired);
    let changes = drift.drift();
    assert_eq!(changes.len(), 1, "expected exactly one delta: {changes:?}");
    assert_eq!(changes[0].kind, ChangeKind::Add);
    assert_eq!(changes[0].path, "hosted_mcp_servers.platform_sobject_all");
}

// `named_user_jwt` lives on ExtlClntAppGlobalOauthSettings, which no query API
// exposes, so it is deployed unconditionally rather than compared.
#[test]
fn named_user_jwt_is_always_applied() {
    let desired = spec();
    let actual = desired.clone();
    let changes = diff(&actual, &desired);

    assert!(changes.is_clean());
    assert!(
        changes.changes.iter().any(|c| {
            c.kind == ChangeKind::AlwaysApplied
                && c.path == "external_client_app.oauth.named_user_jwt"
                && c.desired == "true"
        }),
        "named_user_jwt must be reported as always-applied: {:?}",
        changes.changes
    );
}
