//! The Salesforce org registry: parse, boot validation, entitlement.

use std::collections::HashMap;
use systemprompt::identifiers::McpServerId;
use systemprompt::models::mcp::{Deployment, McpServerType};
use systemprompt_web_admin::connector_oauth::Provider;
use systemprompt_web_admin::salesforce_orgs::{
    SalesforceEnvironment, SalesforceOrgRegistry, is_salesforce_server_id,
};

const PROD: &str = "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all";
const SANDBOX: &str = "https://api.salesforce.com/platform/mcp/v1/sandbox/platform/sobject-all";

fn registry() -> SalesforceOrgRegistry {
    SalesforceOrgRegistry::parse(
        r#"
orgs:
  salesforce:
    label: "Astound (production)"
    my_domain: https://astound.my.salesforce.com
    environment: production
    client_id_secret: sf_prod_id
    client_secret: sf_prod_secret
  salesforce-uat:
    label: "Client A UAT"
    my_domain: https://client-a--uat.sandbox.my.salesforce.com
    environment: sandbox
    org_id: 00D000000000001AAA
    client_id_secret: sf_uat_id
    client_secret: sf_uat_secret
    groups: [commerce]
"#,
    )
    .unwrap()
}

fn server(enabled: bool, endpoint: &str) -> Deployment {
    let yaml = format!(
        "type: external\nbinary: ''\nport: 5042\nendpoint: {endpoint}\nenabled: {enabled}\n\
         display_in_web: false\noauth:\n  required: false\n  scopes: [user]\n  audience: mcp\n"
    );
    serde_yaml::from_str(&yaml).unwrap()
}

#[test]
fn org_ids_are_the_salesforce_server_ids() {
    assert!(is_salesforce_server_id("salesforce"));
    assert!(is_salesforce_server_id("salesforce-uat"));
    assert!(!is_salesforce_server_id("salesforce-"));
    assert!(!is_salesforce_server_id("salesforcex"));
    assert!(!is_salesforce_server_id("atlassian"));
    assert!(matches!(
        Provider::try_from("salesforce-uat".to_owned()).unwrap(),
        Provider::Salesforce(id) if id == "salesforce-uat"
    ));
    assert!(matches!(
        Provider::try_from("salesforcex".to_owned()).unwrap(),
        Provider::Generic(_)
    ));
    let provider = Provider::try_from("salesforce".to_owned()).unwrap();
    assert!(provider.is_salesforce());
    assert_eq!(String::from(provider), "salesforce");
}

#[test]
fn registry_parses_and_selects_the_endpoint_by_environment() {
    let registry = registry();
    assert_eq!(
        registry.list_ids().collect::<Vec<_>>(),
        ["salesforce", "salesforce-uat"]
    );
    let prod = registry.find(&McpServerId::new("salesforce")).unwrap();
    assert_eq!(prod.environment, SalesforceEnvironment::Production);
    assert_eq!(prod.expected_endpoint(), PROD);
    assert!(prod.org_id.is_none());
    let uat = registry.find(&McpServerId::new("salesforce-uat")).unwrap();
    assert_eq!(uat.expected_endpoint(), SANDBOX);
    assert_eq!(uat.org_id.as_deref(), Some("00D000000000001AAA"));
    assert!(
        SalesforceOrgRegistry::parse("")
            .unwrap()
            .find(&McpServerId::new("salesforce"))
            .is_none()
    );
}

#[test]
fn registry_rejects_unknown_fields_and_bad_environments() {
    assert!(SalesforceOrgRegistry::parse("orgs:\n  salesforce:\n    label: x\n    my_domain: https://a.my.salesforce.com\n    environment: production\n    client_id_secret: a\n    client_secret: b\n    instance_url: nope\n").is_err());
    assert!(SalesforceOrgRegistry::parse("orgs:\n  salesforce:\n    label: x\n    my_domain: https://a.my.salesforce.com\n    environment: staging\n    client_id_secret: a\n    client_secret: b\n").is_err());
}

#[test]
fn boot_validation_binds_each_org_to_a_matching_external_server() {
    let registry = registry();
    let mut servers = HashMap::new();
    servers.insert("salesforce".to_owned(), server(true, PROD));
    servers.insert("salesforce-uat".to_owned(), server(true, SANDBOX));
    assert!(registry.validate_against(&servers).is_empty());

    servers.insert("salesforce-uat".to_owned(), server(true, PROD));
    let errors = registry.validate_against(&servers);
    assert_eq!(errors.len(), 1);
    assert!(errors[0].contains("salesforce-uat") && errors[0].contains("endpoint"));

    servers.insert("salesforce-uat".to_owned(), server(false, SANDBOX));
    assert!(registry.validate_against(&servers)[0].contains("disabled"));

    servers.remove("salesforce-uat");
    assert!(registry.validate_against(&servers)[0].contains("no MCP server"));

    let mut internal = server(true, SANDBOX);
    internal.server_type = McpServerType::Internal;
    servers.insert("salesforce-uat".to_owned(), internal);
    assert!(registry.validate_against(&servers)[0].contains("type: external"));
}

#[test]
fn boot_validation_requires_the_domain_to_match_the_environment() {
    let registry = SalesforceOrgRegistry::parse(
        "orgs:\n  salesforce-x:\n    label: x\n    my_domain: https://x.my.salesforce.com\n    environment: sandbox\n    client_id_secret: a\n    client_secret: b\n  salesforce-y:\n    label: y\n    my_domain: http://y.my.salesforce.com\n    environment: production\n    client_id_secret: a\n    client_secret: b\n",
    )
    .unwrap();
    let mut servers = HashMap::new();
    servers.insert("salesforce-x".to_owned(), server(true, SANDBOX));
    servers.insert("salesforce-y".to_owned(), server(true, PROD));
    let errors = registry.validate_against(&servers);
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("salesforce-x") && e.contains("environment"))
    );
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("salesforce-y") && e.contains("HTTPS"))
    );
}

#[test]
fn entitlement_is_open_without_groups_and_group_scoped_with_them() {
    let registry = registry();
    let prod = registry.find(&McpServerId::new("salesforce")).unwrap();
    let uat = registry.find(&McpServerId::new("salesforce-uat")).unwrap();
    assert!(prod.is_entitled(&[]));
    assert!(prod.is_entitled(&["core".to_owned()]));
    assert!(!uat.is_entitled(&[]));
    assert!(!uat.is_entitled(&["core".to_owned()]));
    assert!(uat.is_entitled(&["core".to_owned(), "commerce".to_owned()]));
    assert!(!uat.provisioned());
}
