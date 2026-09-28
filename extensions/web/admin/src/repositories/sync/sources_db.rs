//! Records the active composition in the database: one row per source with
//! its content hash, one row per owned marketplace, plugin and skill.
//!
//! Written at every boot, replaced wholesale, so the feedback projection can
//! answer "which published tree did this skill run from" for every
//! invocation from SQL alone. Ownership is decided the way composition
//! decides it — a bundle owns what its manifest says, the base owns the
//! rest — so the answer here is the answer the loader gave.

use std::collections::BTreeMap;

use sqlx::{PgPool, Postgres, Transaction};
use systemprompt::config::ProfileBootstrap;
use systemprompt::identifiers::MarketplaceId;
use systemprompt::loader::ConfigLoader;
use systemprompt::loader::services_root::ServicesRootBootstrap;
use systemprompt::models::services::ServicesConfig;
use systemprompt_web_shared::error::MarketplaceError;

use super::marketplace_hash::compute_marketplace_versions;
use super::marketplace_versions_db::{VersionsRecorded, record_marketplace_versions};
use super::sources::{SourcesView, build_sources};

pub const BASE_SOURCE: &str = "base";

#[derive(Debug, Default, Clone, Copy)]
pub struct SourcesRecorded {
    pub sources: usize,
    pub owned_ids: usize,
    pub versions: VersionsRecorded,
}

struct Owned {
    kind: &'static str,
    id: String,
    source: String,
    marketplace: Option<MarketplaceId>,
}

pub async fn record_service_sources(pool: &PgPool) -> Result<SourcesRecorded, MarketplaceError> {
    let sources = build_sources().map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    let services = ConfigLoader::load().map_err(|e| MarketplaceError::Internal(e.to_string()))?;
    let owned = owned_ids(&sources, &services);
    // Why: the composed root is what consumers are served, so it is what a
    // version hashes — the baked tree alone would miss every bundled
    // marketplace.
    let root = ServicesRootBootstrap::active_root_or(
        &ProfileBootstrap::get()
            .map_err(|e| MarketplaceError::Internal(e.to_string()))?
            .paths
            .services,
    );
    let versions = compute_marketplace_versions(&root, &services, &sources);

    let mut tx = pool.begin().await?;
    sqlx::query!("DELETE FROM service_owned_ids")
        .execute(&mut *tx)
        .await?;
    sqlx::query!("DELETE FROM service_sources")
        .execute(&mut *tx)
        .await?;
    insert_source(
        &mut tx,
        &SourceRow {
            name: BASE_SOURCE,
            kind: "base",
            content_hash: sources.base.tree_hash.as_deref(),
            digest: None,
            version: Some(sources.base.version),
            provenance: sources.base.provenance,
        },
    )
    .await?;
    for b in &sources.bundles {
        insert_source(
            &mut tx,
            &SourceRow {
                name: &b.owner_key,
                kind: "bundle",
                content_hash: b.content_hash.as_deref(),
                digest: b.active_digest.as_deref(),
                version: b.version.as_deref(),
                provenance: sources.base.provenance,
            },
        )
        .await?;
    }
    for o in &owned {
        sqlx::query!(
            r"INSERT INTO service_owned_ids (kind, id, source, marketplace_id)
              VALUES ($1, $2, $3, $4)
              ON CONFLICT (kind, id) DO NOTHING",
            o.kind,
            o.id,
            o.source,
            o.marketplace.as_ref().map(MarketplaceId::as_str),
        )
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    let versions = record_marketplace_versions(pool, &versions).await?;
    Ok(SourcesRecorded {
        sources: 1 + sources.bundles.len(),
        owned_ids: owned.len(),
        versions,
    })
}

struct SourceRow<'a> {
    name: &'a str,
    kind: &'a str,
    content_hash: Option<&'a str>,
    digest: Option<&'a str>,
    version: Option<&'a str>,
    provenance: &'a str,
}

async fn insert_source(
    tx: &mut Transaction<'_, Postgres>,
    row: &SourceRow<'_>,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r"INSERT INTO service_sources (name, kind, content_hash, digest, version, provenance)
          VALUES ($1, $2, $3, $4, $5, $6)",
        row.name,
        row.kind,
        row.content_hash,
        row.digest,
        row.version,
        row.provenance,
    )
    .execute(&mut **tx)
    .await?;
    Ok(())
}

// Why: a bundle's manifest lists what it owns; everything the composed
// config defines that no bundle claims is the base's. Marketplace membership
// comes from the composed config so a bundled plugin is filed under the
// marketplace that includes it, wherever that marketplace came from.
fn owned_ids(sources: &SourcesView, services: &ServicesConfig) -> Vec<Owned> {
    let mut source_of: BTreeMap<(&str, &str), &str> = BTreeMap::new();
    for b in &sources.bundles {
        for m in &b.owns.marketplaces {
            source_of.insert(("marketplace", m), &b.owner_key);
        }
        for p in &b.owns.plugins {
            source_of.insert(("plugin", p), &b.owner_key);
        }
        for s in &b.owns.skills {
            source_of.insert(("skill", s), &b.owner_key);
        }
    }
    let source = |kind: &str, id: &str| {
        source_of
            .get(&(kind, id))
            .map_or(BASE_SOURCE, |s| s)
            .to_owned()
    };

    let mut marketplace_of_plugin: BTreeMap<&str, &str> = BTreeMap::new();
    let mut out = Vec::new();
    let mut marketplaces: Vec<_> = services.marketplaces.iter().collect();
    marketplaces.sort_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
    for (id, m) in marketplaces {
        out.push(Owned {
            kind: "marketplace",
            id: id.as_str().to_owned(),
            source: source("marketplace", id.as_str()),
            marketplace: None,
        });
        for plugin in &m.plugins.include {
            marketplace_of_plugin
                .entry(plugin.as_str())
                .or_insert(id.as_str());
        }
    }
    let mut plugins: Vec<_> = services.plugins.values().collect();
    plugins.sort_by(|a, b| a.id.as_str().cmp(b.id.as_str()));
    for p in plugins {
        let marketplace = marketplace_of_plugin
            .get(p.id.as_str())
            .map(|m| MarketplaceId::new(*m));
        out.push(Owned {
            kind: "plugin",
            id: p.id.as_str().to_owned(),
            source: source("plugin", p.id.as_str()),
            marketplace: marketplace.clone(),
        });
        for skill in &p.skills.include {
            out.push(Owned {
                kind: "skill",
                id: skill.clone(),
                source: source("skill", skill),
                marketplace: marketplace.clone(),
            });
        }
    }
    out
}
