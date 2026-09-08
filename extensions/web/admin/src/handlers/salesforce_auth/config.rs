//! Salesforce connection config, loaded from
//! `services/web/config/salesforce.yaml`.
//!
//! This is no longer an SSO config — ADFS is the login. What remains is the
//! Connected App identity needed to mint a per-user bearer for the Salesforce
//! MCP server.

use serde::Deserialize;

// Why: env var first, then the encrypted secrets store. Never persisted in
// `salesforce.yaml`, which is a tracked file.
pub(crate) fn salesforce_private_key() -> Option<String> {
    std::env::var("SALESFORCE_PRIVATE_KEY").ok().or_else(|| {
        systemprompt::config::SecretsBootstrap::get()
            .ok()
            .and_then(|s| s.get("salesforce_private_key").cloned())
    })
}

/// Salesforce Connected App config.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceConfig {
    #[serde(default)]
    pub enabled: bool,
    // Why: the org's My Domain base URL, e.g.
    // `https://example.my.salesforce.com`. Doubles as the JWT-bearer audience.
    pub my_domain: String,
    #[serde(alias = "client_id")]
    pub consumer_key: String,
}

impl SalesforceConfig {
    // Why: the placeholder used when no `salesforce.yaml` is present, so the
    // accessor still registers and reports "unavailable" cleanly rather than
    // 404ing and looking like a deployment bug.
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            my_domain: String::new(),
            consumer_key: String::new(),
        }
    }

    #[must_use]
    pub const fn is_usable(&self) -> bool {
        self.enabled && !self.my_domain.is_empty() && !self.consumer_key.is_empty()
    }

    fn base(&self) -> &str {
        self.my_domain.trim_end_matches('/')
    }

    pub(crate) fn token_url(&self) -> String {
        format!("{}/services/oauth2/token", self.base())
    }

    pub(crate) fn jwt_bearer_audience(&self) -> &str {
        self.base()
    }
}
