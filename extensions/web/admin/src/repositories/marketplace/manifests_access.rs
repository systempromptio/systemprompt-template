//! Fills each marketplace manifest's `access` summary from the database.
//!
//! The catalog pages show who reaches a marketplace. That answer lives in
//! `access_control_rules` — seeded from `rules.yaml`, edited in the console —
//! so it is read from there, in one pass for every manifest, rather than from
//! a block the manifests no longer carry.

use sqlx::PgPool;

use super::manifests::{MarketplaceAccessBand, MarketplaceAccessSummary, MarketplaceConfigSummary};
use crate::repositories::access_control::rules::{list_band_rules, list_entity_defaults};

pub async fn attach_access(pool: &PgPool, manifests: &mut [MarketplaceConfigSummary]) {
    let (rules, entities) = match (
        list_band_rules(pool).await,
        list_entity_defaults(pool).await,
    ) {
        (Ok(rules), Ok(entities)) => (rules, entities),
        (Err(e), _) | (_, Err(e)) => {
            tracing::warn!(error = %e, "catalog: marketplace access could not be read");
            return;
        },
    };
    for manifest in manifests.iter_mut() {
        let id = manifest.id.as_str();
        let mut summary = MarketplaceAccessSummary {
            default_included: entities
                .iter()
                .any(|e| e.entity_type == "marketplace" && e.entity_id == id && e.default_included),
            ..MarketplaceAccessSummary::default()
        };
        for rule in rules
            .iter()
            .filter(|r| r.entity_type == "marketplace" && r.entity_id == id)
        {
            let access = rule.access.to_string();
            if access == "allow" {
                match rule.rule_type.as_str() {
                    "role" => summary.roles.push(rule.rule_value.clone()),
                    "group" => summary.groups.push(rule.rule_value.clone()),
                    "project" => summary.projects.push(rule.rule_value.clone()),
                    _ => {},
                }
            }
            if summary.justification.is_none() {
                summary.justification.clone_from(&rule.justification);
            }
            match summary
                .bands
                .iter_mut()
                .find(|b| b.rule_type == rule.rule_type && b.access == access)
            {
                Some(band) => band.values.push(rule.rule_value.clone()),
                None => summary.bands.push(MarketplaceAccessBand {
                    rule_type: rule.rule_type.clone(),
                    values: vec![rule.rule_value.clone()],
                    access,
                    justification: rule.justification.clone(),
                }),
            }
        }
        summary.roles.sort();
        summary.groups.sort();
        summary.projects.sort();
        manifest.access = summary;
    }
}
