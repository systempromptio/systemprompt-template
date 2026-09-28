//! The `gateway_routes` plane: `services/ai/gateway.yaml`'s `routes:`
//! sequence projected into `gateway_routes`.
//!
//! The one plane whose declaration is also its runtime input. Core boots
//! the dispatcher from the file and never reads the table, so every apply
//! ends by regenerating the file from the rows — after *Overwrite* the two
//! are equal by construction; after *Insert only* the file gains the rows
//! the console added, which the next restart dispatches. The boot job seeds
//! an empty table from the file and otherwise only reports; a fresh image
//! whose file moved on shows here as drift until an administrator chooses a
//! direction, and a console route from the previous image is *In database,
//! not in code* until it is applied or exported.

use async_trait::async_trait;
use sqlx::PgPool;

use super::access_control::applied_from;
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DriftKpi, DriftRow, Export, PlaneDrift, SyncMode, SyncPlane,
    resolution,
};
use super::state::{Applied, record_applied, record_declared};
use crate::error::AdminResult;
use crate::repositories::gateway_routes::declared::{
    DeclaredRoutes, GATEWAY_FILE, declared_routes_now, gateway_file_path,
};
use crate::repositories::gateway_routes::drift::{RouteDrift, compute_route_drift};
use crate::repositories::gateway_routes::render::{regenerate_gateway_file, render_routes_export};
use crate::repositories::gateway_routes::rows::{
    RouteWrite, SOURCE_CODE, count_gateway_routes, delete_gateway_route, list_gateway_routes,
    upsert_gateway_route,
};

pub const PLANE_ID: &str = "gateway_routes";

pub const RUNTIME_NOTE: &str = "Core boots its dispatcher from services/ai/gateway.yaml and \
    never reads this table. Every console write and every apply regenerates the file from the \
    rows, so a change here is dispatched at the next restart.";

#[derive(Debug, Clone, Copy, Default)]
pub struct GatewayRoutesPlane;

fn row(kind: &'static str, id: &str, in_code: String, in_db: String, dashboard: bool) -> DriftRow {
    let (kind_label, kind_tone, insert_applies, overwrite_effect) = match (kind, dashboard) {
        ("missing", _) => ("Added in code", "ok", true, "inserted"),
        ("orphan", true) => ("Written in console", "warn", false, "deleted"),
        ("orphan", false) => ("Removed from code", "err", false, "deleted"),
        ("reordered", _) => ("Order changed in code", "warn", false, "reordered"),
        (_, true) => ("Edited in console", "warn", false, "updated"),
        (_, false) => ("Changed in code", "warn", false, "updated"),
    };
    let (resolve, resolve_tone) = resolution(insert_applies, true, overwrite_effect);
    DriftRow {
        kind,
        kind_label,
        kind_tone,
        entity_type: "gateway_route".to_owned(),
        entity_type_label: "gateway route".to_owned(),
        entity_id: id.to_owned(),
        band: String::new(),
        band_label: "route",
        subject: id.to_owned(),
        in_code,
        in_db,
        origin: match kind {
            "missing" => "",
            _ if dashboard => "console",
            _ => "code",
        },
        origin_tone: if dashboard { "info" } else { "muted" },
        governed: true,
        insert_applies,
        overwrite_applies: true,
        overwrite_effect,
        resolve,
        resolve_tone,
    }
}

fn rows(drift: &RouteDrift) -> Vec<DriftRow> {
    let mut out = Vec::new();
    out.extend(
        drift
            .missing_in_db
            .iter()
            .map(|id| row("missing", id, "declared".to_owned(), "—".to_owned(), false)),
    );
    out.extend(drift.only_in_db.iter().map(|o| {
        row(
            "orphan",
            &o.id,
            "—".to_owned(),
            o.summary.clone(),
            o.dashboard,
        )
    }));
    out.extend(drift.changed.iter().map(|c| {
        let mut r = row(
            "changed",
            &c.id,
            c.in_code.clone(),
            c.in_db.clone(),
            c.dashboard,
        );
        r.subject = c.fields.join(", ");
        r
    }));
    if let Some((code, db)) = &drift.reordered {
        out.push(row(
            "reordered",
            "order",
            code.join(" › "),
            db.join(" › "),
            true,
        ));
    }
    out
}

async fn write_declared(
    pool: &PgPool,
    declared: &DeclaredRoutes,
    mode: SyncMode,
) -> AdminResult<ApplyOutcome> {
    let existing = list_gateway_routes(pool).await?;
    let mut outcome = ApplyOutcome::default();
    for (i, d) in declared.routes.iter().enumerate() {
        let present = existing.iter().any(|r| r.route.id == d.route.id);
        if present && mode == SyncMode::InsertOnly {
            continue;
        }
        // Why: insert-only appends after the rows already there — the file's
        // order is only imposed by an overwrite, which is the mode that
        // promises the table equals the file.
        let position = match mode {
            SyncMode::Overwrite => i,
            SyncMode::InsertOnly => existing.len() + i,
        };
        upsert_gateway_route(
            pool,
            &RouteWrite {
                route: &d.route,
                position: i32::try_from(position).unwrap_or(i32::MAX),
                explicit_id: d.explicit_id,
                source: SOURCE_CODE,
            },
        )
        .await?;
        if present {
            outcome.updated += 1;
        } else {
            outcome.inserted += 1;
        }
    }
    if mode == SyncMode::Overwrite {
        for r in &existing {
            if declared.find(&r.route.id).is_none() {
                outcome.deleted +=
                    usize::try_from(delete_gateway_route(pool, &r.route.id).await?).unwrap_or(0);
            }
        }
    }
    Ok(outcome)
}

#[async_trait]
impl SyncPlane for GatewayRoutesPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Gateway routes"
    }

    fn source_file(&self) -> &'static str {
        GATEWAY_FILE
    }

    fn projection(&self) -> &'static str {
        "gateway_routes"
    }

    fn runtime_note(&self) -> Option<&'static str> {
        Some(RUNTIME_NOTE)
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_gateway_routes(pool).await?).unwrap_or(0);
        let declared = match declared_routes_now() {
            Ok(d) => d,
            Err(e) => {
                return Ok(PlaneDrift {
                    in_db,
                    unreadable: Some(e.to_string()),
                    ..PlaneDrift::default()
                });
            },
        };
        let declared_hash = declared.declared_hash();
        record_declared(pool, PLANE_ID, &declared_hash).await?;
        let drift = compute_route_drift(&declared, &list_gateway_routes(pool).await?);
        let console_orphans = drift.only_in_db.iter().filter(|o| o.dashboard).count();
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.routes.len(),
            in_db,
            kpis: vec![
                DriftKpi {
                    label: "Added in code",
                    value: drift.missing_in_db.len(),
                    note: "Insert only or Overwrite adds these",
                    tone: "ok",
                },
                DriftKpi {
                    label: "Removed from code",
                    value: drift.only_in_db.len(),
                    note: "Overwrite deletes these",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Changed in code",
                    value: drift.changed.len() + usize::from(drift.reordered.is_some()),
                    note: "provider, model, fallback or order disagree",
                    tone: "warn",
                },
            ],
            actions: ActionCounts {
                insert: drift.missing_in_db.len(),
                insert_entities: 0,
                update: drift.changed.len() + usize::from(drift.reordered.is_some()),
                delete: drift.only_in_db.len(),
                delete_console: console_orphans,
                kept: 0,
            },
            is_clean: drift.is_clean(),
            rows: rows(&drift),
            unreadable: None,
        })
    }

    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_routes_now()?;
        let outcome = write_declared(pool, &declared, mode).await?;
        // Why: the boot seed came from the file, so the file already says
        // what the rows say; rewriting it would only strip its inline
        // comments. A person's apply may have kept console rows the file
        // lacks, and those must reach the file core boots from.
        if matches!(actor, Actor::User(_)) {
            regenerate_gateway_file(pool, &gateway_file_path()?).await?;
        }
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
        Ok(outcome)
    }

    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>> {
        let rows = list_gateway_routes(pool).await?;
        let file = std::fs::read_to_string(gateway_file_path()?)
            .map_err(|e| crate::error::AdminError::invalid("gateway.yaml could not be read", e))?;
        let body = render_routes_export(&file, &rows).map_err(|e| {
            crate::error::AdminError::invalid("gateway.yaml could not be rendered", e)
        })?;
        Ok(Some(Export {
            filename: "gateway.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: rows.len(),
            body,
        }))
    }
}
