//! The `gateway_policies` plane: `services/gateway/policies.yaml` projected
//! into `ai_gateway_policies`.
//!
//! *Overwrite from code* upserts every declared policy and deletes every
//! undeclared row; *Insert only* adds the declared policies the table lacks.
//! Boot runs the overwrite once, on an empty table, and otherwise only
//! compares. Both finish by running the month-window
//! rewrite, so a calendar-month window declared with its sentinel is live
//! on today's value before the response returns. The console editor at
//! `/admin/gateway/policies` is the other writer; what it changes shows
//! here as drift until it is exported or overwritten.

use async_trait::async_trait;
use sqlx::PgPool;

use super::access_control::applied_from;
use super::declaration::read_or_text;
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DeclarationSource, DriftKpi, DriftRow, Export, PlaneDrift,
    SyncMode, SyncPlane, resolution,
};
use super::state::{Applied, record_applied, record_declared};
use crate::error::{AdminError, AdminResult};
use crate::repositories::gateway_policies::declared::{
    DeclaredPolicies, POLICIES_FILE, parse_declared_policies,
};
use crate::repositories::gateway_policies::drift::{PolicyDrift, compute_policy_drift};
use crate::repositories::gateway_policies::export::render_policies_export;
use crate::repositories::gateway_policies::month_window_db::refresh_month_windows;
use crate::repositories::gateway_policies::rows::{
    PolicyWrite, count_policies, delete_policy, list_policies, upsert_policy,
};

pub const PLANE_ID: &str = "gateway_policies";

#[derive(Debug, Clone, Copy, Default)]
pub struct GatewayPoliciesPlane;

// Why: no file is a legitimate declaration of "no policies" — core boots
// permissive on it — so it is an empty set, not an error.
fn declared_policies_from(from: DeclarationSource<'_>) -> AdminResult<DeclaredPolicies> {
    let Some(yaml) = read_or_text(from, POLICIES_FILE)? else {
        return Ok(DeclaredPolicies::default());
    };
    parse_declared_policies(&yaml)
        .map_err(|e| AdminError::invalid("policies.yaml could not be parsed", e))
}

fn row(kind: &'static str, name: &str, in_code: String, in_db: String) -> DriftRow {
    let (kind_label, kind_tone, insert_applies, overwrite_effect) = match kind {
        "missing" => ("Added in code", "ok", true, "inserted"),
        "orphan" => ("Written in console", "warn", false, "deleted"),
        _ => ("Edited in console", "warn", false, "updated"),
    };
    let (resolve, resolve_tone) = resolution(insert_applies, true, overwrite_effect);
    DriftRow {
        kind,
        kind_label,
        kind_tone,
        entity_type: "gateway_policy".to_owned(),
        entity_type_label: "gateway policy".to_owned(),
        entity_id: name.to_owned(),
        band: String::new(),
        band_label: "policy",
        subject: name.to_owned(),
        in_code,
        in_db,
        origin: if kind == "missing" { "" } else { "console" },
        origin_tone: "info",
        governed: true,
        insert_applies,
        overwrite_applies: true,
        overwrite_effect,
        resolve,
        resolve_tone,
    }
}

fn rows(drift: &PolicyDrift) -> Vec<DriftRow> {
    let mut out = Vec::new();
    out.extend(
        drift
            .missing_in_db
            .iter()
            .map(|n| row("missing", n, "declared".to_owned(), "—".to_owned())),
    );
    out.extend(
        drift
            .only_in_db
            .iter()
            .map(|n| row("orphan", n, "—".to_owned(), "present".to_owned())),
    );
    out.extend(drift.changed.iter().map(|c| {
        let mut r = row("changed", &c.name, c.in_code.clone(), c.in_db.clone());
        r.subject = c.fields.join(", ");
        r
    }));
    out
}

#[async_trait]
impl SyncPlane for GatewayPoliciesPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Gateway policies"
    }

    fn source_file(&self) -> &'static str {
        POLICIES_FILE
    }

    fn projection(&self) -> &'static str {
        "ai_gateway_policies"
    }

    fn owner_url(&self) -> &'static str {
        "/admin/gateway/policies"
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        self.drift_from(pool, DeclarationSource::Disk).await
    }

    async fn drift_from(
        &self,
        pool: &PgPool,
        from: DeclarationSource<'_>,
    ) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_policies(pool).await?).unwrap_or(0);
        let declared = match declared_policies_from(from) {
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
        if from.is_disk() {
            record_declared(pool, PLANE_ID, &declared_hash).await?;
        }
        let drift = compute_policy_drift(&declared, &list_policies(pool).await?);
        Ok(PlaneDrift {
            declared_hash,
            declared_count: declared.entries.len(),
            in_db,
            kpis: vec![
                DriftKpi {
                    label: "Added in code",
                    value: drift.missing_in_db.len(),
                    note: "Insert only or Overwrite adds these",
                    tone: "ok",
                },
                DriftKpi {
                    label: "Written in console",
                    value: drift.only_in_db.len(),
                    note: "Overwrite deletes these",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Edited in console",
                    value: drift.changed.len(),
                    note: "windows, scanners or modes disagree",
                    tone: "warn",
                },
            ],
            actions: ActionCounts {
                insert: drift.missing_in_db.len(),
                insert_entities: 0,
                update: drift.changed.len(),
                delete: drift.only_in_db.len(),
                delete_console: drift.only_in_db.len(),
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
        self.apply_from(pool, mode, actor, DeclarationSource::Disk)
            .await
    }

    async fn apply_from(
        &self,
        pool: &PgPool,
        mode: SyncMode,
        actor: Actor<'_>,
        from: DeclarationSource<'_>,
    ) -> AdminResult<ApplyOutcome> {
        let declared = declared_policies_from(from)?;
        let existing = list_policies(pool).await?;
        let mut outcome = ApplyOutcome::default();
        for entry in &declared.entries {
            let present = existing.iter().any(|r| r.name == entry.name);
            if present && mode == SyncMode::InsertOnly {
                continue;
            }
            upsert_policy(
                pool,
                &PolicyWrite {
                    name: &entry.name,
                    spec: &entry.spec,
                    enabled: entry.enabled,
                    priority: entry.priority,
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
                if declared.find(&r.name).is_none() {
                    outcome.deleted +=
                        usize::try_from(delete_policy(pool, &r.name).await?).unwrap_or(0);
                }
            }
        }
        refresh_month_windows(pool, chrono::Utc::now()).await?;
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
        let rows = list_policies(pool).await?;
        Ok(Some(Export {
            filename: "policies.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: rows.len(),
            body: render_policies_export(&rows, chrono::Utc::now()),
        }))
    }
}
