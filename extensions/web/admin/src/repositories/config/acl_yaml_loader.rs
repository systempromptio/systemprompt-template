//! Bootstrap loader: `services/access-control/*.yaml` → DB.
//!
//! YAML is the declarative source of truth committed to the source repo. On
//! every server startup the governance bootstrap calls [`load_from_yaml`],
//! which upserts entity and rule rows into the runtime database. There is no
//! write-back: dashboard edits live only in the DB of the instance that
//! received them, so deployments sharing the same YAML baseline can drift
//! independently without trampling each other.
//!
//! Three files are read (all optional; missing-file = no-op), plus one
//! projection out of the services config:
//!
//! - `services/access-control/roles.yaml` — role-scoped allow/deny rules.
//!   Parsed into core's [`AccessControlConfig`] and projected by core
//!   ingestion, which owns `entity_match` glob expansion, `default_included`,
//!   and `entity_id` self-materialisation for the kinds the caller does not
//!   enforce through [`RegisteredEntities`] — a `gateway_route` id outside the
//!   profile's routes is rejected, never materialised.
//! - `services/access-control/{groups,projects}.yaml` — entities confined to
//!   members of a named group or project. Owned by
//!   [`super::member_grants_yaml_loader`].
//! - each marketplace's own `access` block in `services/marketplaces/*` —
//!   ingested by core so a marketplace's entitlement is authored beside the
//!   marketplace rather than duplicated into roles.yaml.
//! - `services/slack/*.yaml` — each app's `authz.allowed_roles`, projected onto
//!   its `slack_workspace` entity. Core writes the rows; nothing called it, so
//!   an app's `allowed_roles` documented an intention it never enforced.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::identifiers::MarketplaceId;
use systemprompt::loader::ConfigLoader;
use systemprompt::models::services::MarketplaceConfig;
use systemprompt_security::authz::{
    AccessControlConfig, AccessControlIngestionService, IngestOptions, RegisteredEntities,
};
use systemprompt_web_shared::error::MarketplaceError;

use super::acl_yaml_types::LoadReport;

const ROLES_FILE: &str = "access-control/roles.yaml";

// Why: `registered` is the caller's because only the caller knows where its
// truth lives — for `gateway_route` that is the live profile, reconciled by
// the governance bootstrap immediately before this runs. An empty
// `RegisteredEntities` enforces nothing.
pub async fn load_from_yaml(
    pool: &PgPool,
    services_path: &Path,
    registered: &RegisteredEntities,
) -> Result<LoadReport, MarketplaceError> {
    let mut report = LoadReport::default();

    load_roles_file(pool, services_path, registered, &mut report).await?;

    // Why: after the roles pass so a marketplace's own `access` block owns the
    // last word on its entity, and before the membership gates, which are
    // narrower still. `delete_orphans` stays false: a rule an operator added
    // in the dashboard at a band the file does not mention must survive a
    // restart.
    let marketplace_rules = ingest_marketplaces(pool).await?;

    let member_grants =
        super::member_grants_yaml_loader::load_member_grants_from_yaml(pool, services_path).await?;

    // Why: same reasoning as the membership gates above — these force
    // default_included=false on their entities and must run after roles.
    let link_gates =
        super::linked_yaml_loader::load_link_gates_from_yaml(pool, services_path).await?;

    let slack_workspaces = load_slack_apps(pool).await?;

    tracing::info!(
        rules = report.rules_upserted,
        marketplace_rules,
        member_grants = member_grants.grants_projected,
        link_gate_grants = link_gates.grants_projected,
        slack_workspaces,
        "bootstrap_yaml_loaded"
    );
    Ok(report)
}

async fn read_yaml<T: for<'de> Deserialize<'de> + Default>(
    services_path: &Path,
    rel: &str,
) -> Result<Option<T>, MarketplaceError> {
    let path = services_path.join(rel);
    match tokio::fs::read_to_string(&path).await {
        Ok(s) if s.trim().is_empty() => Ok(Some(T::default())),
        Ok(s) => serde_yaml::from_str::<T>(&s)
            .map(Some)
            .map_err(|e| MarketplaceError::config_file(rel, e)),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(e.into()),
    }
}

async fn load_roles_file(
    pool: &PgPool,
    services_path: &Path,
    registered: &RegisteredEntities,
    report: &mut LoadReport,
) -> Result<(), MarketplaceError> {
    let Some(cfg) = read_yaml::<AccessControlConfig>(services_path, ROLES_FILE).await? else {
        return Ok(());
    };

    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_config(
            &cfg,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
                ..IngestOptions::default()
            },
            registered,
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    report.rules_upserted = ingested.inserted + ingested.updated;
    Ok(())
}

// Why: the marketplace configs are the source of truth for who reaches a
// marketplace, so the rules ride with them instead of being restated in
// roles.yaml where the two could drift. Same tolerance as the Slack pass
// below: an unreadable services config is not this loader's failure to report.
async fn ingest_marketplaces(pool: &PgPool) -> Result<usize, MarketplaceError> {
    // Why: both arms below return "nothing to ingest", which is
    // indistinguishable from "every marketplace is already up to date" in the
    // bootstrap tally. Silence here left two private workspaces closed to the
    // groups they exist for and reported it as a clean start, so each arm says
    // which one it took.
    let services = match ConfigLoader::load() {
        Ok(services) => services,
        Err(e) => {
            tracing::error!(
                error = %e,
                "marketplace_access_skipped: services config unreadable"
            );
            return Ok(0);
        },
    };
    if services.marketplaces.is_empty() {
        tracing::warn!("marketplace_access_skipped: the services tree declares no marketplaces");
        return Ok(0);
    }
    ingest_marketplace_map(pool, &services.marketplaces).await
}

// Why: separate from `ingest_marketplaces` so a caller can name the map. The
// bootstrap reads the active profile's services tree through a process-wide
// cache; a test that has to read a fixture tree cannot go through it.
#[expect(
    clippy::implicit_hasher,
    reason = "core's ingest_marketplace_access takes the concrete map type"
)]
pub async fn ingest_marketplace_map(
    pool: &PgPool,
    marketplaces: &HashMap<MarketplaceId, MarketplaceConfig>,
) -> Result<usize, MarketplaceError> {
    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_marketplace_access(
            marketplaces,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
                ..IngestOptions::default()
            },
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    tracing::info!(
        marketplaces = marketplaces.len(),
        inserted = ingested.inserted,
        updated = ingested.updated,
        skipped = ingested.skipped,
        "marketplace_access_ingested"
    );
    Ok(ingested.inserted + ingested.updated)
}

// Why: an unreadable services config is not this loader's failure to report —
// the server does not start without one, and treating it as fatal here would
// turn every unrelated config error into "access control failed to load".
async fn load_slack_apps(pool: &PgPool) -> Result<usize, MarketplaceError> {
    let Ok(services) = ConfigLoader::load() else {
        return Ok(0);
    };
    if services.slack_apps.is_empty() {
        return Ok(0);
    }

    let svc = AccessControlIngestionService::from_pool(Arc::new(pool.clone()));
    let ingested = svc
        .ingest_slack_apps(
            &services.slack_apps,
            IngestOptions {
                override_existing: true,
                delete_orphans: false,
                ..IngestOptions::default()
            },
        )
        .await
        .map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    Ok(ingested.inserted + ingested.updated)
}
