//! The `governance` plane: `services/governance/config.yaml` staged into
//! `governance_chain` and `governance_chain_settings`.
//!
//! The chain core enforces is built from the file at boot; nothing in the
//! process re-reads it, so this plane validates and stages. *Overwrite from
//! code* makes the tables equal the file (positions, switches, modes and
//! the raw entries), *Insert only* adds the policies the table lacks, and
//! *Export* renders the staged chain as the file — the route by which a
//! staged change reaches git, the image, and, after a restart, enforcement.
//! Parsing goes through core's own parser, so nothing the console stages
//! is something the boot would refuse.

use async_trait::async_trait;
use sqlx::PgPool;

use super::access_control::applied_from;
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DriftKpi, DriftRow, Export, PlaneDrift, SyncMode, SyncPlane,
    resolution,
};
use super::state::{Applied, record_applied, record_declared};
use crate::error::AdminResult;
use crate::repositories::governance_chain::declared::{
    DeclaredChain, GOVERNANCE_FILE, declared_chain_now,
};
use crate::repositories::governance_chain::export::{
    ChainDrift, compute_chain_drift, render_chain_export, summarise_stage,
};
use crate::repositories::governance_chain::rows::{
    SOURCE_CODE, count_chain_policies, delete_chain_policy, find_chain_settings,
    list_chain_policies, staged_chain, upsert_chain_policy, upsert_chain_settings,
};

pub const PLANE_ID: &str = "governance";

pub const RUNTIME_NOTE: &str = "Core builds the policy chain from services/governance/config.yaml \
    at boot and never reads governance_chain. A change staged here is enforced only after the \
    file is exported, committed and the server restarted.";

#[derive(Debug, Clone, Copy, Default)]
pub struct GovernancePlane;

fn row(kind: &'static str, id: &str, in_code: String, in_db: String) -> DriftRow {
    let (kind_label, kind_tone, insert_applies, overwrite_effect) = match kind {
        "missing" => ("Added in code", "ok", true, "inserted"),
        "orphan" => ("Removed from code", "err", false, "deleted"),
        "reordered" => ("Order changed in code", "warn", false, "reordered"),
        _ => ("Changed in code", "warn", false, "updated"),
    };
    let (resolve, resolve_tone) = resolution(insert_applies, true, overwrite_effect);
    DriftRow {
        kind,
        kind_label,
        kind_tone,
        entity_type: "governance_policy".to_owned(),
        entity_type_label: "governance policy".to_owned(),
        entity_id: id.to_owned(),
        band: String::new(),
        band_label: "policy",
        subject: id.to_owned(),
        in_code,
        in_db,
        origin: if kind == "missing" { "" } else { "staged" },
        origin_tone: "muted",
        governed: true,
        insert_applies,
        overwrite_applies: true,
        overwrite_effect,
        resolve,
        resolve_tone,
    }
}

fn rows(drift: &ChainDrift, declared: &DeclaredChain, staged: &DeclaredChain) -> Vec<DriftRow> {
    let mut out = Vec::new();
    if !drift.settings_changed.is_empty() {
        let mut r = row(
            "changed",
            "chain",
            summarise_stage(declared.enabled, &declared.mode),
            summarise_stage(staged.enabled, &staged.mode),
        );
        r.subject = drift.settings_changed.join(", ");
        out.push(r);
    }
    out.extend(
        drift
            .missing_in_db
            .iter()
            .map(|id| row("missing", id, "declared".to_owned(), "—".to_owned())),
    );
    out.extend(
        drift
            .only_in_db
            .iter()
            .map(|id| row("orphan", id, "—".to_owned(), "staged".to_owned())),
    );
    out.extend(drift.changed.iter().map(|c| {
        let mut r = row("changed", &c.id, c.in_code.clone(), c.in_db.clone());
        r.subject = c.fields.join(", ");
        r
    }));
    if let Some((code, db)) = &drift.reordered {
        out.push(row("reordered", "order", code.join(" › "), db.join(" › ")));
    }
    out
}

async fn staged_now(pool: &PgPool) -> AdminResult<DeclaredChain> {
    let settings = find_chain_settings(pool).await?;
    let rows = list_chain_policies(pool).await?;
    Ok(staged_chain(settings.as_ref(), &rows))
}

#[async_trait]
impl SyncPlane for GovernancePlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Governance chain"
    }

    fn source_file(&self) -> &'static str {
        GOVERNANCE_FILE
    }

    fn projection(&self) -> &'static str {
        "governance_chain"
    }

    fn runtime_note(&self) -> Option<&'static str> {
        Some(RUNTIME_NOTE)
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_chain_policies(pool).await?).unwrap_or(0);
        let declared = match declared_chain_now() {
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
        let staged = staged_now(pool).await?;
        let drift = compute_chain_drift(&declared, &staged);
        let differ = drift.total() - drift.missing_in_db.len() - drift.only_in_db.len();
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.policies.len(),
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
                    value: differ,
                    note: "switch, mode, parameters or order disagree",
                    tone: "warn",
                },
            ],
            actions: ActionCounts {
                insert: drift.missing_in_db.len(),
                insert_entities: 0,
                update: differ,
                delete: drift.only_in_db.len(),
                delete_console: 0,
                kept: 0,
            },
            is_clean: drift.is_clean(),
            rows: rows(&drift, &declared, &staged),
            unreadable: None,
        })
    }

    async fn apply(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_chain_now()?;
        let existing = list_chain_policies(pool).await?;
        let mut outcome = ApplyOutcome::default();
        if mode == SyncMode::Overwrite || find_chain_settings(pool).await?.is_none() {
            upsert_chain_settings(pool, declared.enabled, &declared.mode).await?;
        }
        for (i, p) in declared.policies.iter().enumerate() {
            let present = existing.iter().any(|r| r.policy_id == p.id);
            if present && mode == SyncMode::InsertOnly {
                continue;
            }
            let position = match mode {
                SyncMode::Overwrite => i,
                SyncMode::InsertOnly => existing.len() + i,
            };
            let position = i32::try_from(position).unwrap_or(i32::MAX);
            upsert_chain_policy(pool, p, position, SOURCE_CODE).await?;
            if present {
                outcome.updated += 1;
            } else {
                outcome.inserted += 1;
            }
        }
        if mode == SyncMode::Overwrite {
            for r in &existing {
                if declared.find(&r.policy_id).is_none() {
                    outcome.deleted +=
                        usize::try_from(delete_chain_policy(pool, &r.policy_id).await?)
                            .unwrap_or(0);
                }
            }
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
        let staged = staged_now(pool).await?;
        Ok(Some(Export {
            filename: "config.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: staged.policies.len(),
            body: render_chain_export(&staged, chrono::Utc::now()),
        }))
    }
}
