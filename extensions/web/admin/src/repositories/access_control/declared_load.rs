//! Assembles the declared set from disk and the catalog.
//!
//! The I/O half of [`super::declared`]: reads `rules.yaml` and hands it, with
//! the live gateway routes the caller registered, to the pure projection.
//! The route catalog is the services configuration, not the
//! `access_control_entities` rows: those rows are what the apply writes, so
//! reading them back to decide what to declare made an empty catalog declare
//! nothing and report no drift. Marketplace ids
//! arrive as a parameter — the boot path and the page take them from the
//! composed services tree, a test from a fixture tree — so the same function
//! serves both without the process-wide cache leaking into tests.
//!
//! A tree that does not compose is its own error. Turning it into an empty
//! catalog once made a dangling plugin reference read as "rules.yaml names a
//! marketplace that does not exist" — the wrong file blamed for the wrong
//! reason, on every entity of the page.

use std::path::Path;

use systemprompt::loader::ConfigLoader;
use systemprompt_security::authz::{EntityKind, RegisteredEntities};
use systemprompt_web_shared::error::MarketplaceError;

use super::declared::{DeclaredInputs, DeclaredSet, build_declared_set};
use crate::repositories::config::rules_yaml_loader::read_rules_doc;
use crate::repositories::config::rules_yaml_types::RulesDoc;

pub async fn load_declared_set(
    services_path: &Path,
    marketplace_ids: &[String],
    registered: &RegisteredEntities,
) -> Result<DeclaredSet, MarketplaceError> {
    let Some(doc) = read_rules_doc(services_path).await? else {
        return Ok(DeclaredSet::default());
    };
    build_declared_from_doc(&doc, marketplace_ids, registered)
}

// Why: the projection half on its own, so an uploaded `rules.yaml` (an
// import preview) is declared exactly as the file on disk would be — same
// route catalog, same marketplace ids, same expiry pass.
pub fn build_declared_from_doc(
    doc: &RulesDoc,
    marketplace_ids: &[String],
    registered: &RegisteredEntities,
) -> Result<DeclaredSet, MarketplaceError> {
    let gateway_routes: Vec<String> = registered
        .known_ids(EntityKind::GatewayRoute)
        .into_iter()
        .map(str::to_owned)
        .collect();
    let mut set = build_declared_set(
        doc,
        &DeclaredInputs {
            gateway_routes: &gateway_routes,
            marketplace_ids,
            registered,
        },
    )
    .map_err(|e| MarketplaceError::config_file("access-control/rules.yaml", e))?;
    set.drop_expired(chrono::Utc::now());
    Ok(set)
}

// Why: the marketplace ids the composed tree defines — the same root
// `rules.yaml` is read from, so a kit's marketplace validates the moment its
// bundle is active, without a restart. The loader's cache is keyed by root,
// so this is a read of the composition, not a parse per request.
pub fn marketplace_ids_from_services() -> Result<Vec<String>, MarketplaceError> {
    let services = ConfigLoader::load()?;
    Ok(services
        .marketplaces
        .keys()
        .map(|id| id.as_str().to_owned())
        .collect())
}
