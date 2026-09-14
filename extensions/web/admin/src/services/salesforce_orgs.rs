//! Salesforce org registry: `services/salesforce/orgs.yaml`, one record per
//! hosted-MCP server id.
//!
//! Salesforce has no central authorization server. Every org is its own
//! issuer with its own External Client App, its own user records and — for
//! sandboxes — a different hosted MCP endpoint, so "which org" has to be
//! decided before consent and remembered per grant. The registry is that
//! decision: each org is declared against the MCP server that fronts it
//! (`salesforce` for the default production org, `salesforce-<slug>` for the
//! rest), and every Salesforce branch of the connector reads its domain,
//! credentials and endpoint from here rather than from global secrets.
//!
//! Boot rules: a malformed file, an org whose server is missing, disabled,
//! internal, or pointed at the wrong environment endpoint all fail boot. A
//! missing *secret* does not — that org is reported as `not_provisioned` on
//! the profile page so an org a Salesforce admin has not set up yet is a
//! visible row, not a failing tool call.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;
use systemprompt::identifiers::McpServerId;

use serde::Deserialize;
use systemprompt::models::mcp::{Deployment, McpServerType};

use crate::error::{AdminError, AdminResult};
use crate::services::connector_oauth::config::{secret, validate_my_domain};

pub const ORGS_FILE: &str = "salesforce/orgs.yaml";
const PRODUCTION_ENDPOINT: &str = "https://api.salesforce.com/platform/mcp/v1/platform/sobject-all";
const SANDBOX_ENDPOINT: &str =
    "https://api.salesforce.com/platform/mcp/v1/sandbox/platform/sobject-all";
const SANDBOX_DOMAIN_SUFFIX: &str = ".sandbox.my.salesforce.com";

static REGISTRY: OnceLock<SalesforceOrgRegistry> = OnceLock::new();

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SalesforceEnvironment {
    Production,
    Sandbox,
}

/// One org: its login, its app, and who may connect to it.
///
/// `client_id_secret`, `client_secret` and `private_key_secret` are the
/// *names* of entries in the secret store, never values.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SalesforceOrg {
    pub label: String,
    pub my_domain: String,
    pub environment: SalesforceEnvironment,
    #[serde(default)]
    pub org_id: Option<String>,
    pub client_id_secret: String,
    pub client_secret: String,
    #[serde(default)]
    pub private_key_secret: Option<String>,
    #[serde(default)]
    pub groups: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct SalesforceOrgsDoc {
    #[serde(default)]
    orgs: BTreeMap<McpServerId, SalesforceOrg>,
}

#[derive(Debug, Default)]
pub struct SalesforceOrgRegistry {
    orgs: BTreeMap<McpServerId, SalesforceOrg>,
}

pub fn is_salesforce_server_id(id: &str) -> bool {
    id == "salesforce"
        || id
            .strip_prefix("salesforce-")
            .is_some_and(|slug| !slug.is_empty())
}

impl SalesforceOrg {
    pub const fn expected_endpoint(&self) -> &'static str {
        match self.environment {
            SalesforceEnvironment::Production => PRODUCTION_ENDPOINT,
            SalesforceEnvironment::Sandbox => SANDBOX_ENDPOINT,
        }
    }

    pub fn domain(&self) -> AdminResult<String> {
        validate_my_domain(&self.my_domain)
    }

    pub fn client_id(&self) -> AdminResult<String> {
        secret(&self.client_id_secret)
    }

    pub fn client_secret_value(&self) -> AdminResult<String> {
        secret(&self.client_secret)
    }

    pub fn private_key(&self) -> AdminResult<String> {
        let name = self.private_key_secret.as_deref().ok_or_else(|| {
            AdminError::Unavailable("Salesforce org has no signing key configured".into())
        })?;
        secret(name)
    }

    pub fn provisioned(&self) -> bool {
        self.client_id().is_ok() && self.client_secret_value().is_ok()
    }

    pub fn is_entitled(&self, user_groups: &[String]) -> bool {
        self.groups.is_empty() || self.groups.iter().any(|g| user_groups.contains(g))
    }

    fn validate(
        &self,
        id: &str,
        servers: &std::collections::HashMap<String, Deployment>,
    ) -> Vec<String> {
        let mut errors = Vec::new();
        if !is_salesforce_server_id(id) {
            errors.push(format!(
                "{id}: org ids must be `salesforce` or `salesforce-<slug>`"
            ));
        }
        match validate_my_domain(&self.my_domain) {
            Ok(domain) => {
                let sandbox_domain = domain.ends_with(SANDBOX_DOMAIN_SUFFIX);
                if sandbox_domain != (self.environment == SalesforceEnvironment::Sandbox) {
                    errors.push(format!(
                        "{id}: my_domain does not match environment (sandbox domains end with {SANDBOX_DOMAIN_SUFFIX})"
                    ));
                }
            },
            Err(_) => errors.push(format!("{id}: my_domain must be an HTTPS My Domain origin")),
        }
        let Some(server) = servers.get(id) else {
            errors.push(format!("{id}: no MCP server with this id in services/mcp"));
            return errors;
        };
        if !server.enabled {
            errors.push(format!("{id}: MCP server is disabled"));
        }
        if server.server_type != McpServerType::External {
            errors.push(format!("{id}: MCP server must be `type: external`"));
        }
        if server.endpoint.as_deref() != Some(self.expected_endpoint()) {
            errors.push(format!(
                "{id}: MCP server endpoint must be {} for a {:?} org",
                self.expected_endpoint(),
                self.environment
            ));
        }
        errors
    }
}

impl SalesforceOrgRegistry {
    pub fn parse(yaml: &str) -> Result<Self, String> {
        if yaml.trim().is_empty() {
            return Ok(Self::default());
        }
        let doc: SalesforceOrgsDoc = serde_yaml::from_str(yaml).map_err(|e| e.to_string())?;
        Ok(Self { orgs: doc.orgs })
    }

    pub async fn init_from_path(services_path: &Path) -> Result<(), String> {
        let path = services_path.join(ORGS_FILE);
        let registry = match tokio::fs::read_to_string(&path).await {
            Ok(yaml) => Self::parse(&yaml).map_err(|e| format!("{ORGS_FILE}: {e}"))?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Self::default(),
            Err(e) => return Err(format!("{ORGS_FILE}: {e}")),
        };
        if REGISTRY.set(registry).is_err() {
            tracing::debug!("salesforce_orgs_already_initialised");
        }
        Ok(())
    }

    pub fn get() -> &'static Self {
        static EMPTY: SalesforceOrgRegistry = SalesforceOrgRegistry {
            orgs: BTreeMap::new(),
        };
        REGISTRY.get().unwrap_or(&EMPTY)
    }

    pub fn find(&self, server_id: &McpServerId) -> Option<&SalesforceOrg> {
        self.orgs.get(server_id)
    }

    pub fn list_ids(&self) -> impl Iterator<Item = &str> {
        self.orgs.keys().map(McpServerId::as_str)
    }

    pub fn validate_against(
        &self,
        servers: &std::collections::HashMap<String, Deployment>,
    ) -> Vec<String> {
        self.orgs
            .iter()
            .flat_map(|(id, org)| org.validate(id.as_str(), servers))
            .collect()
    }
}

// Why: the bootstrap job is the one place that has both the services path and
// the loaded MCP catalog; an org declared against a server the catalog does
// not carry is a misconfiguration nothing downstream can repair.
pub async fn salesforce_orgs_boot_check(services_path: &Path) -> Result<(), String> {
    SalesforceOrgRegistry::init_from_path(services_path).await?;
    let services = systemprompt::loader::ServicesBootstrap::get().map_err(|e| e.to_string())?;
    let errors = SalesforceOrgRegistry::get().validate_against(&services.mcp_servers);
    if errors.is_empty() {
        tracing::info!(
            orgs = SalesforceOrgRegistry::get().list_ids().count(),
            "salesforce_orgs_loaded"
        );
        return Ok(());
    }
    Err(format!("{ORGS_FILE}: {}", errors.join("; ")))
}
