//! Wire types for the bootstrap downstream-link gate loader
//! (`access-control/salesforce.yaml`).
//!
//! The file lists entities confined to users who have linked that downstream
//! account. Access is not authored per grant: every row it produces is
//! `access = allow`, `rule_value = 'linked'`, `default_included = false`. A
//! grant with any other shape would not be a link gate, so the schema cannot
//! express one — which is also why there is no `values:` field to get wrong.

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkedGrantsDoc {
    #[serde(default)]
    pub grants: Vec<LinkedGrant>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LinkedGrant {
    pub entity_type: String,
    pub entity_id: String,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct LinkedGrantsLoadReport {
    pub grants_projected: usize,
}
