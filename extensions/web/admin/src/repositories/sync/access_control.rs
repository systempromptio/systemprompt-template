//! The `access_control` plane: `services/access-control/rules.yaml` projected
//! into `access_control_rules` and `access_control_entities`.
//!
//! A thin adapter over the engine in [`crate::repositories::access_control`]
//! — declared set, drift, apply, export — that adds the two things the sync
//! page needs from every plane: a declared hash, and a `sync_state` row
//! written on every apply. The database-only rows a kit's own `access:`
//! block wrote (source `bundle:<name>`) surface here as *bundle* orphans.
//!
//! Every request-time caller — the page, the drift API, the apply — reads the
//! declared set through [`declared_from`], from the composed root, and
//! uploaded text is projected by the same code as the file, so a preview's
//! drift is the drift the file would show once committed. An unreadable file
//! is the card's headline, not a 500. A scoped apply records the whole file's
//! hash, since the projection it leaves is still measured against the whole
//! file. Export renders the database for copy-out only; the instance never
//! writes into `services/`.

use async_trait::async_trait;
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::services_root::ServicesRootBootstrap;

use super::access_control_rows::{awaiting_rows, rows};
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DeclarationSource, DriftKpi, EntityScope, Export,
    PlaneDrift, SyncMode, SyncPlane,
};
use super::state::{Applied, AppliedFrom, record_applied, record_declared};
use crate::error::{AdminError, AdminResult};
use crate::repositories::access_control::declared::DeclaredSet;
use crate::repositories::access_control::declared_load::{
    build_declared_from_doc, load_declared_set, marketplace_ids_from_services,
};
use crate::repositories::access_control::drift::{DriftCounts, DriftReport, compute_drift};
use crate::repositories::access_control::export::render_export;
use crate::repositories::access_control::rules::{
    count_band_rules, list_band_rules, list_entity_defaults,
};
use crate::repositories::access_control::sync::{apply_sync, apply_sync_scoped};
use crate::repositories::config::gateway::registered_routes_from_services;
use crate::repositories::config::rules_yaml_loader::{RULES_FILE, parse_rules_doc};

pub const PLANE_ID: &str = "access_control";

#[derive(Debug, Clone, Copy, Default)]
pub struct AccessControlPlane;

pub async fn declared_now() -> AdminResult<DeclaredSet> {
    declared_from(DeclarationSource::Disk).await
}

pub async fn declared_from(from: DeclarationSource<'_>) -> AdminResult<DeclaredSet> {
    let registered = registered_routes_from_services()?;
    let marketplace_ids = marketplace_ids_from_services()
        .map_err(|source| AdminError::invalid("services tree could not be composed", source))?;
    let loaded = match from {
        DeclarationSource::Disk => {
            let profile = ProfileBootstrap::get()?;
            let services_path = ServicesRootBootstrap::active_root_or(&profile.paths.services);
            load_declared_set(&services_path, &marketplace_ids, &registered).await
        },
        DeclarationSource::Text(text) => match parse_rules_doc(text) {
            Ok(doc) => build_declared_from_doc(&doc, &marketplace_ids, &registered),
            Err(e) => Err(e),
        },
    };
    loaded.map_err(|source| AdminError::invalid("rules.yaml could not be loaded", source))
}

pub async fn drift_now(pool: &PgPool, declared: &DeclaredSet) -> AdminResult<DriftReport> {
    let rules = list_band_rules(pool).await?;
    let entities = list_entity_defaults(pool).await?;
    Ok(compute_drift(declared, &rules, &entities))
}

#[must_use]
pub fn applied_from() -> AppliedFrom {
    // Why: discard-ok: an unreadable profile or cache records no hashes
    let sources = super::sources::build_sources().ok();
    AppliedFrom {
        base_tree_hash: sources.as_ref().and_then(|s| s.base.tree_hash.clone()),
        composed_hash: sources.and_then(|s| s.composed_hash),
    }
}

fn unreadable_declaration(in_db: usize, error: &AdminError) -> PlaneDrift {
    PlaneDrift {
        in_db,
        unreadable: Some(error.to_string()),
        ..PlaneDrift::default()
    }
}

#[must_use]
pub fn drift_kpis(counts: &DriftCounts, awaiting_bundle: usize) -> Vec<DriftKpi> {
    vec![
        DriftKpi {
            label: "Added in code",
            value: counts.missing_in_db + counts.entities_missing,
            note: "Apply all new adds these",
            tone: "ok",
        },
        DriftKpi {
            label: "Changed in code",
            value: counts.changed + counts.default_changed,
            note: "Replace database with code corrects these",
            tone: "warn",
        },
        DriftKpi {
            label: "Removed from code",
            value: counts.only_in_db_code,
            note: "Replace database with code deletes these",
            tone: "err",
        },
        DriftKpi {
            label: "Written in console",
            value: counts.only_in_db_dashboard,
            note: "Export carries these to code",
            tone: "info",
        },
        DriftKpi {
            label: "Awaiting a bundle",
            value: awaiting_bundle + counts.only_in_db_bundle,
            note: "owned by a kit; nothing to do here",
            tone: "muted",
        },
    ]
}

impl From<&DriftCounts> for ActionCounts {
    fn from(counts: &DriftCounts) -> Self {
        Self {
            insert: counts.missing_in_db,
            insert_entities: counts.entities_missing,
            update: counts.changed + counts.default_changed,
            delete: counts.only_in_db_retire,
            delete_console: counts.only_in_db_console_retire,
            kept: counts.only_in_db_kept,
        }
    }
}

impl AccessControlPlane {
    async fn record(
        &self,
        pool: &PgPool,
        declared: &DeclaredSet,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<()> {
        record_applied(
            pool,
            &Applied {
                plane: PLANE_ID,
                declared_hash: &declared.declared_hash(),
                mode: actor.recorded_mode(mode),
                actor: actor.as_str(),
                from: applied_from(),
            },
        )
        .await?;
        Ok(())
    }
}

#[async_trait]
impl SyncPlane for AccessControlPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Access control"
    }

    fn source_file(&self) -> &'static str {
        RULES_FILE
    }

    fn projection(&self) -> &'static str {
        "access_control_rules · access_control_entities"
    }

    fn owner_url(&self) -> &'static str {
        "/admin/access-control"
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        self.drift_from(pool, DeclarationSource::Disk).await
    }

    async fn drift_from(
        &self,
        pool: &PgPool,
        from: DeclarationSource<'_>,
    ) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_band_rules(pool).await?).unwrap_or(0);
        let declared = match declared_from(from).await {
            Ok(d) => d,
            Err(e) => return Ok(unreadable_declaration(in_db, &e)),
        };
        let declared_hash = declared.declared_hash();
        if from.is_disk() {
            record_declared(pool, PLANE_ID, &declared_hash).await?;
        }
        let report = drift_now(pool, &declared).await?;
        if from.is_disk() {
            super::attention::reviews_for(pool, &report).await?;
        }
        let counts = report.counts();
        let mut rows = rows(&report);
        rows.extend(awaiting_rows(&declared));
        let is_clean = report.is_clean();
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.rule_count(),
            in_db,
            kpis: drift_kpis(&counts, declared.awaiting.len()),
            actions: ActionCounts::from(&counts),
            rows,
            is_clean,
            unreadable: None,
        })
    }

    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        self.apply_from(pool, mode, actor, DeclarationSource::Disk)
            .await
    }

    async fn apply_scoped(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        scope: EntityScope<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_now().await?;
        let outcome = apply_sync_scoped(pool, &declared, mode, scope).await?;
        self.record(pool, &declared, mode, actor).await?;
        Ok(outcome)
    }

    async fn apply_from(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        from: DeclarationSource<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_from(from).await?;
        let outcome = apply_sync(pool, &declared, mode).await?;
        self.record(pool, &declared, mode, actor).await?;
        Ok(outcome)
    }

    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>> {
        let rules = list_band_rules(pool).await?;
        let entities = list_entity_defaults(pool).await?;
        let owners_or_none_when_unreadable =
            declared_now().await.map(|d| d.owners).unwrap_or_default();
        Ok(Some(Export {
            filename: "rules.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: rules.len(),
            body: render_export(&rules, &entities, &owners_or_none_when_unreadable),
        }))
    }
}
