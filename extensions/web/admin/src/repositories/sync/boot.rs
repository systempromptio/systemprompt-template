//! What boot does with every plane: seed an empty projection, otherwise look
//! and report.
//!
//! The database is what is enforced and what the console edits; the file is
//! what the database is expected to contain. Boot therefore writes exactly
//! once per plane — when there is nothing yet to disagree with — and every
//! later start computes the drift and says so in the log, leaving the
//! decision to reconcile to an administrator on `/admin/sync`. Upserting on
//! every start was how console edits vanished overnight and how rules for
//! deleted entities lived on for months; one loop over the registry is what
//! stops a plane from drifting back to that.
//!
//! A declaration that cannot be read fails the boot. The page turns the same
//! condition into a card headline because an operator came to look; a server
//! starting with half its declarations is a server nobody asked for.

use sqlx::PgPool;

use super::plane::{Actor, SyncMode, SyncPlane};
use super::registry::find_plane;
use crate::error::{AdminError, AdminResult};

// Why: groups before access control because the rules name group and
// project ids; the last three name nothing and are named by nothing. The two
// trailing ids are literals so this file compiles whether or not those plane
// modules are present yet — `reconcile_all` fails loudly on an unregistered
// id, which is the check that matters.
pub const BOOT_ORDER: [&str; 5] = [
    super::groups::PLANE_ID,
    super::access_control::PLANE_ID,
    super::gateway_policies::PLANE_ID,
    "gateway_routes",
    "governance",
];

/// One plane's boot outcome, for the log line.
#[derive(Debug, Clone, Copy, Default)]
pub struct PlaneBoot {
    pub declared: usize,
    pub in_db: usize,
    pub seeded: bool,
    pub inserted: usize,
    pub drift_rows: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BootReport {
    pub groups: PlaneBoot,
    pub access_control: PlaneBoot,
    pub gateway_policies: PlaneBoot,
    pub gateway_routes: PlaneBoot,
    pub governance: PlaneBoot,
}

pub async fn reconcile_plane(pool: &PgPool, plane: &dyn SyncPlane) -> AdminResult<PlaneBoot> {
    let drift = plane.drift(pool).await?;
    if let Some(reason) = drift.unreadable {
        return Err(AdminError::invalid(
            "a declaration could not be read at boot",
            format!("{}: {reason}", plane.source_file()),
        ));
    }
    let mut boot = PlaneBoot {
        declared: drift.declared_count,
        in_db: drift.in_db,
        drift_rows: drift.rows.len(),
        ..PlaneBoot::default()
    };
    if drift.in_db == 0 && drift.declared_count > 0 {
        let outcome = plane.apply(pool, SyncMode::Overwrite, Actor::Boot).await?;
        boot.seeded = true;
        boot.inserted = outcome.inserted;
        boot.drift_rows = 0;
        tracing::info!(
            plane = plane.id(),
            file = plane.source_file(),
            declared = boot.declared,
            inserted = outcome.inserted,
            "sync_seeded: empty projection populated from code"
        );
        return Ok(boot);
    }
    if drift.is_clean {
        tracing::info!(
            plane = plane.id(),
            declared = boot.declared,
            in_db = boot.in_db,
            "sync_in_sync: code and database agree; nothing written"
        );
    } else {
        tracing::warn!(
            plane = plane.id(),
            file = plane.source_file(),
            declared = boot.declared,
            in_db = boot.in_db,
            drift_rows = boot.drift_rows,
            "sync_drift: code and database differ; nothing written — review and \
             synchronise on /admin/sync"
        );
    }
    Ok(boot)
}

pub async fn reconcile_all(pool: &PgPool) -> AdminResult<BootReport> {
    let mut report = BootReport::default();
    for id in BOOT_ORDER {
        let plane = find_plane(id)
            .ok_or_else(|| AdminError::internal(format!("sync plane '{id}' is not registered")))?;
        let boot = reconcile_plane(pool, plane.as_ref()).await?;
        match id {
            super::groups::PLANE_ID => report.groups = boot,
            super::access_control::PLANE_ID => report.access_control = boot,
            super::gateway_policies::PLANE_ID => report.gateway_policies = boot,
            "gateway_routes" => report.gateway_routes = boot,
            _ => report.governance = boot,
        }
    }
    Ok(report)
}
