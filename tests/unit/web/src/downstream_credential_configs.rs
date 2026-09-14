//! The committed downstream-credential configs are load-bearing and otherwise
//! unvalidated until boot: nothing else in the suite reads
//! `services/web/config/salesforce.yaml` or the external MCP server YAMLs that
//! depend on a per-user accessor. These checks parse every one of them the way
//! the loaders do, so a typo'd key or an accessor path that disagrees with the
//! route it names fails here rather than at a user's first tool call.
//!
//! They also pin the posture: the Salesforce app ships disabled ahead of its
//! credentials, the hosted Atlassian connector is the only Atlassian server,
//! and the retired v1 `jira`/`confluence` servers cannot creep back.

use std::path::PathBuf;

use systemprompt_web_admin::SalesforceConfig;

use crate::support::repo_root;

// Why: must match the routes registered by `connector_api_router` /
// `salesforce_api_router` under the `/api/public` nest. Core resolves the
// accessor as `api_external_url + token_endpoint`, so a disagreement here is a
// 404 at tool-call time and nowhere earlier.
const ATLASSIAN_ACCESSOR: &str = "/api/public/connectors/atlassian/token";
const SALESFORCE_ACCESSOR: &str = "/api/public/connectors/salesforce/token";

fn web_config(name: &str) -> PathBuf {
    repo_root().join("services/web/config").join(name)
}

fn mcp_config(name: &str) -> serde_yaml::Value {
    let path = repo_root().join("services/mcp").join(name);
    let yaml = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("{} is readable: {e}", path.display()));
    serde_yaml::from_str(&yaml).unwrap_or_else(|e| panic!("{} parses as YAML: {e}", path.display()))
}

fn server(file: &str, name: &str) -> serde_yaml::Value {
    mcp_config(file)
        .get("mcp_servers")
        .and_then(|s| s.get(name))
        .unwrap_or_else(|| panic!("{file} declares an `mcp_servers.{name}` entry"))
        .clone()
}

#[test]
fn the_salesforce_config_parses_as_the_loader_reads_it() {
    let path = web_config("salesforce.yaml");
    let yaml = std::fs::read_to_string(&path).expect("salesforce.yaml is readable");
    let config: SalesforceConfig = serde_yaml::from_str(&yaml)
        .unwrap_or_else(|e| panic!("{} parses as SalesforceConfig: {e}", path.display()));

    assert!(
        config.my_domain.starts_with("https://"),
        "my_domain doubles as the JWT-bearer audience and must be the https My Domain base, got {}",
        config.my_domain
    );
}

#[test]
fn the_salesforce_app_ships_disabled_and_its_server_enabled() {
    // Why: the app needs a credential that does not exist in any environment
    // yet; the server is visible only to linked users, so it can stay on.
    assert_eq!(
        server("salesforce.yaml", "salesforce")["enabled"].as_bool(),
        Some(true)
    );
    let yaml = std::fs::read_to_string(web_config("salesforce.yaml")).expect("readable");
    let value: serde_yaml::Value = serde_yaml::from_str(&yaml).expect("parses");
    assert_eq!(
        value.get("enabled").and_then(serde_yaml::Value::as_bool),
        Some(false),
        "salesforce.yaml must ship disabled"
    );
}

#[test]
fn every_external_server_points_at_the_accessor_that_serves_it() {
    for (file, name, accessor) in [
        ("atlassian.yaml", "atlassian", ATLASSIAN_ACCESSOR),
        ("salesforce.yaml", "salesforce", SALESFORCE_ACCESSOR),
    ] {
        let entry = server(file, name);
        let external_auth = entry
            .get("external_auth")
            .unwrap_or_else(|| panic!("{file}: {name} must declare external_auth"));

        assert_eq!(
            external_auth
                .get("token_endpoint")
                .and_then(serde_yaml::Value::as_str),
            Some(accessor),
            "{file}: token_endpoint must name the route that actually serves it"
        );

        // Why: the presence of `external_auth` is what switches core off
        // forwarding our own JWT. `oauth.required: true` alongside it would gate
        // the same call twice against two different credentials.
        assert_eq!(
            entry
                .get("oauth")
                .and_then(|o| o.get("required"))
                .and_then(serde_yaml::Value::as_bool),
            Some(false),
            "{file}: external_auth governs auth, so oauth.required must be false"
        );

        assert_eq!(
            entry.get("type").and_then(serde_yaml::Value::as_str),
            Some("external"),
            "{file}: a server with external_auth must be type: external"
        );
    }
}

#[test]
fn the_atlassian_server_uses_the_v2_streamable_http_endpoint() {
    // Why: core's MCP client speaks only StreamableHttp — it has no HTTP+SSE
    // transport — and Atlassian's /v1 forms are retired with the v1 link.
    let endpoint = server("atlassian.yaml", "atlassian")
        .get("endpoint")
        .and_then(serde_yaml::Value::as_str)
        .expect("atlassian.yaml: atlassian must declare an endpoint")
        .to_owned();
    assert!(
        endpoint.starts_with("https://mcp.atlassian.com/v2/mcp"),
        "atlassian.yaml: the hosted connector is the v2 endpoint, got {endpoint}"
    );
}

#[test]
fn the_retired_v1_atlassian_files_do_not_exist() {
    // Why: the per-server jira/confluence entries and the 3LO app config were
    // deleted with their code (2026-09-08). A file reappearing would be loaded
    // by the aggregator or the link-gate loader and fail at boot, or worse,
    // parse and gate nothing.
    for path in [
        "services/mcp/jira.yaml",
        "services/mcp/confluence.yaml",
        "services/web/config/atlassian.yaml",
        "services/access-control/atlassian.yaml",
    ] {
        assert!(
            !repo_root().join(path).exists(),
            "{path} belongs to the retired v1 Atlassian link"
        );
    }
}

#[test]
fn the_external_mcp_servers_are_listed_in_the_aggregator() {
    // Why: a flat resource file that is not in `includes:` is never loaded, and
    // nothing reports it — the server simply does not exist.
    let aggregator = std::fs::read_to_string(repo_root().join("services/config/config.yaml"))
        .expect("services/config/config.yaml is readable");
    for name in ["atlassian", "salesforce"] {
        assert!(
            aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml must include ../mcp/{name}.yaml"
        );
    }
    for name in ["jira", "confluence"] {
        assert!(
            !aggregator.contains(&format!("../mcp/{name}.yaml")),
            "services/config/config.yaml still includes the retired ../mcp/{name}.yaml"
        );
    }
}
