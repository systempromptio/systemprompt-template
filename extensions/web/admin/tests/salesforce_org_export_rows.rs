//! `salesforce_org::export` — the row-set mappers.
//!
//! Hosted MCP servers and permission sets are exported from query *results*
//! rather than single records, and both do a join the API does not do for
//! them: the MCP endpoint URL is not a field on `McpServerAccess`, and a
//! permission set's app grant is a `SetupEntityAccess` row pointing at an id
//! that only means something after a second query. Getting either join wrong
//! reads as drift the operator cannot resolve.

#![allow(
    clippy::expect_used,
    clippy::panic,
    clippy::unwrap_used,
    clippy::separated_literal_suffix,
    reason = "test code: panics are the assertion mechanism"
)]

use systemprompt_web_admin::salesforce_org::export::{
    UNREADABLE_PLACEHOLDER, hosted_mcp_from_rows, permission_sets_from_rows,
};
use systemprompt_web_admin::salesforce_org::scope::OauthScope;
use systemprompt_web_admin::salesforce_org::spec::{
    ExternalClientApp, HostedMcpServer, IpRelaxation, OauthSpec, OrgSpec, PolicySpec,
};

fn server(developer_name: &str, endpoint: &str) -> HostedMcpServer {
    HostedMcpServer {
        name: developer_name.to_owned(),
        developer_name: developer_name.to_owned(),
        endpoint: endpoint.to_owned(),
        active: true,
    }
}

fn baseline(servers: Vec<HostedMcpServer>) -> OrgSpec {
    OrgSpec {
        external_client_app: ExternalClientApp {
            developer_name: "Systemprompt_SSO".to_owned(),
            label: "Systemprompt SSO".to_owned(),
            description: None,
            contact_email: "ed@systemprompt.io".to_owned(),
            distribution_state: "Local".to_owned(),
            oauth: OauthSpec {
                callback_url: "https://example.test/callback".to_owned(),
                scopes: vec![OauthScope::Mcp],
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
        hosted_mcp_servers: servers,
    }
}

fn mcp_rows() -> Vec<serde_json::Value> {
    vec![
        serde_json::json!({
            "DeveloperName": "platform_sobject_all",
            "MasterLabel": "sobject-all",
            "Active": true,
        }),
        serde_json::json!({
            "DeveloperName": "industries_engagement_interaction",
            "MasterLabel": "engagement-interaction",
            "Active": false,
        }),
    ]
}

// With no baseline the whole inventory comes back, which is what makes a
// first export of an unknown org useful.
#[test]
fn without_a_baseline_every_server_is_reported() {
    let servers = hosted_mcp_from_rows(&mcp_rows(), None);
    assert_eq!(servers.len(), 2);
}

// Scoped to the baseline's servers, because an org offers standard servers
// this deployment does not manage — reporting them would be drift no apply
// will ever resolve.
#[test]
fn a_baseline_scopes_the_result_to_managed_servers() {
    let base = baseline(vec![server(
        "platform_sobject_all",
        "https://example.test/s",
    )]);
    let servers = hosted_mcp_from_rows(&mcp_rows(), Some(&base));
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].developer_name, "platform_sobject_all");
}

// The endpoint is not a field on `McpServerAccess`, so it comes from the
// baseline where there is one.
#[test]
fn the_endpoint_comes_from_the_baseline() {
    let base = baseline(vec![server(
        "platform_sobject_all",
        "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all",
    )]);
    let servers = hosted_mcp_from_rows(&mcp_rows(), Some(&base));
    assert_eq!(
        servers[0].endpoint,
        "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all"
    );
}

#[test]
fn an_unreadable_endpoint_is_flagged_rather_than_invented() {
    let servers = hosted_mcp_from_rows(&mcp_rows(), None);
    for reported in &servers {
        assert_eq!(reported.endpoint, UNREADABLE_PLACEHOLDER);
    }
}

// Matching is on the developer name, not the label. Labels are translatable
// and an org in another language would otherwise export as empty.
#[test]
fn servers_match_on_developer_name_not_label() {
    let base = baseline(vec![HostedMcpServer {
        name: "objets-tous".to_owned(),
        developer_name: "platform_sobject_all".to_owned(),
        endpoint: "https://example.test/s".to_owned(),
        active: true,
    }]);
    let servers = hosted_mcp_from_rows(&mcp_rows(), Some(&base));
    assert_eq!(servers.len(), 1);
    assert_eq!(servers[0].name, "sobject-all");
}

#[test]
fn the_active_flag_is_read_from_the_row() {
    let servers = hosted_mcp_from_rows(&mcp_rows(), None);
    let engagement = servers
        .iter()
        .find(|s| s.developer_name == "industries_engagement_interaction")
        .expect("row is present");
    assert!(!engagement.active);
    assert!(servers.iter().any(|s| s.active));
}

#[test]
fn a_missing_active_flag_reads_as_inactive() {
    let rows = vec![serde_json::json!({ "DeveloperName": "platform_sobject_all" })];
    assert!(!hosted_mcp_from_rows(&rows, None)[0].active);
}

// The label falls back to the developer name so a server never exports
// nameless.
#[test]
fn a_missing_label_falls_back_to_the_developer_name() {
    let rows = vec![serde_json::json!({ "DeveloperName": "platform_sobject_all" })];
    assert_eq!(
        hosted_mcp_from_rows(&rows, None)[0].name,
        "platform_sobject_all"
    );
}

#[test]
fn a_row_without_a_developer_name_is_skipped() {
    let rows = vec![
        serde_json::json!({ "MasterLabel": "orphan", "Active": true }),
        serde_json::json!({ "DeveloperName": "", "Active": true }),
    ];
    assert!(hosted_mcp_from_rows(&rows, None).is_empty());
}

// Sorted so two exports of the same org produce the same file and the diff
// stays reviewable.
#[test]
fn servers_are_sorted_by_developer_name() {
    let servers = hosted_mcp_from_rows(&mcp_rows(), None);
    let names: Vec<&str> = servers.iter().map(|s| s.developer_name.as_str()).collect();
    assert_eq!(
        names,
        vec!["industries_engagement_interaction", "platform_sobject_all"]
    );
}

#[test]
fn an_empty_row_set_exports_no_servers() {
    assert!(hosted_mcp_from_rows(&[], None).is_empty());
    let base = baseline(vec![server(
        "platform_sobject_all",
        "https://example.test/s",
    )]);
    assert!(hosted_mcp_from_rows(&[], Some(&base)).is_empty());
}

fn grant(name: &str, label: &str, entity_id: &str) -> serde_json::Value {
    serde_json::json!({
        "SetupEntityId": entity_id,
        "Parent": { "Name": name, "Label": label },
    })
}

fn apps() -> Vec<serde_json::Value> {
    vec![serde_json::json!({
        "Id": "0Ci000000000001",
        "DeveloperName": "Systemprompt_SSO",
    })]
}

#[test]
fn a_grant_resolves_to_the_app_it_pre_authorizes() {
    let grants = vec![grant(
        "Salesforce_MCP_Access",
        "Salesforce MCP Access",
        "0Ci000000000001",
    )];
    let sets = permission_sets_from_rows(&grants, &apps());
    assert_eq!(sets.len(), 1);
    assert_eq!(sets[0].name, "Salesforce_MCP_Access");
    assert_eq!(sets[0].label, "Salesforce MCP Access");
    assert_eq!(sets[0].grants_app.as_deref(), Some("Systemprompt_SSO"));
}

// The description is not queried, so it exports as absent rather than as an
// empty string that would diff against the committed one.
#[test]
fn the_description_is_not_invented() {
    let grants = vec![grant("Salesforce_MCP_Access", "MCP", "0Ci000000000001")];
    assert!(
        permission_sets_from_rows(&grants, &apps())[0]
            .description
            .is_none()
    );
}

// A grant pointing at an app this query did not return still exports the
// permission set — losing it would hide a set users hold.
#[test]
fn an_unresolvable_grant_keeps_the_permission_set() {
    let grants = vec![grant("Orphan_Set", "Orphan", "0Ci000000000999")];
    let sets = permission_sets_from_rows(&grants, &apps());
    assert_eq!(sets.len(), 1);
    assert!(sets[0].grants_app.is_none());
}

#[test]
fn a_grant_without_a_parent_is_skipped() {
    let grants = vec![serde_json::json!({ "SetupEntityId": "0Ci000000000001" })];
    assert!(permission_sets_from_rows(&grants, &apps()).is_empty());
}

#[test]
fn a_parent_without_a_name_is_skipped() {
    let grants = vec![serde_json::json!({
        "SetupEntityId": "0Ci000000000001",
        "Parent": { "Label": "Nameless" },
    })];
    assert!(permission_sets_from_rows(&grants, &apps()).is_empty());
}

#[test]
fn a_missing_label_falls_back_to_the_api_name() {
    let grants = vec![serde_json::json!({
        "SetupEntityId": "0Ci000000000001",
        "Parent": { "Name": "Salesforce_MCP_Access" },
    })];
    let sets = permission_sets_from_rows(&grants, &apps());
    assert_eq!(sets[0].label, "Salesforce_MCP_Access");
}

#[test]
fn permission_sets_are_sorted_by_api_name() {
    let grants = vec![
        grant("Zeta_Set", "Zeta", "0Ci000000000001"),
        grant("Alpha_Set", "Alpha", "0Ci000000000001"),
    ];
    let sets = permission_sets_from_rows(&grants, &apps());
    let names: Vec<&str> = sets.iter().map(|s| s.name.as_str()).collect();
    assert_eq!(names, vec!["Alpha_Set", "Zeta_Set"]);
}

#[test]
fn an_org_with_no_grants_exports_no_permission_sets() {
    assert!(permission_sets_from_rows(&[], &apps()).is_empty());
}
