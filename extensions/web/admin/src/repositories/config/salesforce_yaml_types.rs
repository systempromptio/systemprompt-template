//! Wire types for the bootstrap Salesforce-gate loader
//! (`access-control/salesforce.yaml`).
//!
//! The file is a bare list of entities confined to Salesforce-linked users.
//! Access and rule value are not authored per grant: every row this file
//! produces is `rule_type = 'salesforce'`, `rule_value = 'linked'`,
//! `access = allow`, `default_included = false` — a grant with any other shape
//! would not be a Salesforce gate, so the schema cannot express one.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceDoc {
    #[serde(default)]
    pub grants: Vec<SalesforceGrant>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SalesforceGrant {
    pub entity_type: String,
    pub entity_id: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct SalesforceLoadReport {
    pub grants_projected: usize,
}
