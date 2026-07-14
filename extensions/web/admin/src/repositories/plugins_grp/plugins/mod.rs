//! Marketplace catalog and overview queries backed by `services/` YAML.
//!
//! `catalog` walks the filesystem for read-only skill / agent / plugin rows;
//! `overview` filters plugins by the caller's roles and counts them.

mod catalog;
mod overview;

pub use catalog::{
    list_agent_catalog, list_all_skill_ids, list_plugin_catalog, list_plugin_skill_ids,
    list_skill_catalog, update_plugin_skills,
};
pub use overview::{
    MarketplaceCounts, count_marketplace_items, list_plugins_for_roles,
    list_plugins_for_roles_full, load_plugin_onboarding_configs,
};
