//! Code ↔ Instance: one sync system for every declared service, local and
//! external.
//!
//! Two vocabularies. A **source** is where declarations come from and carries
//! a content hash — `base` is this repository's `services/` tree, and
//! `bundle:<name>` is each remote kit the profile pins. A **plane** is what a
//! source's declarations project into the database — `access_control`
//! (`rules.yaml` → `access_control_rules`), `groups` (`groups.yaml` →
//! groups, projects and their AD mappings), `gateway_policies`
//! (`policies.yaml` → `ai_gateway_policies`), `gateway_routes`
//! (`ai/gateway.yaml` → `gateway_routes`) and `governance`
//! (`governance/config.yaml` → `governance_chain`). The last two project a
//! file core reads at boot, so they take effect after a restart and say so
//! on their card. Ownership is by id and decided at
//! composition: a bundle owns its marketplaces, plugins and skills; the base
//! owns everything else, every entitlement included.
//!
//! Nothing here writes on its own. [`sources`] reports what is active and
//! what is pinned; each [`plane::SyncPlane`] reports drift and applies a
//! direction only when an administrator chooses one; [`state`] records what
//! was applied, when, by whom and from which declared hash.

pub mod access_control;
pub mod access_control_rows;
pub mod archive;
pub mod attention;
pub mod boot;
pub mod declaration;
pub mod drift_shape;
pub mod gateway_policies;
pub mod gateway_routes;
pub mod governance;
pub mod groups;
pub mod groups_db;
pub mod groups_drift;
pub mod history;
pub mod inventory;
pub mod marketplace_hash;
pub mod marketplace_versions_db;
pub mod plane;
pub mod provenance;
pub mod registry;
pub mod source_badges;
pub mod sources;
pub mod sources_db;
pub mod state;
pub mod tree_hash;
