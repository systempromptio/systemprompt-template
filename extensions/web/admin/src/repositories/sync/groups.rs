//! The `groups` plane: `services/web/config/groups.yaml` projected into
//! `groups`, `projects` and their AD-group mappings.
//!
//! *Overwrite from code* runs the loader — upsert every declared set,
//! reconcile the `yaml`-sourced mappings — and *Insert only* adds what the
//! file declares and the tables lack. Boot runs the overwrite once, on an
//! empty database, and otherwise only compares. Neither deletes a group or
//! project: the loader never does, and the drift table says so on every orphan.
//! *Export* renders every dashboard-created set and mapping back into the
//! file's shape, which is how a group made in the console reaches git.

use async_trait::async_trait;
use sqlx::PgPool;

use super::access_control::applied_from;
use super::declaration::read_or_text;
use super::groups_db::{
    count_member_sets, insert_missing, list_mappings, list_member_sets, parse_groups_doc,
    render_groups_export,
};
use super::groups_drift::{GroupsDrift, compute_groups_drift, declared_count, declared_hash};
use super::plane::{
    ActionCounts, Actor, ApplyOutcome, DeclarationSource, DriftKpi, Export, PlaneDrift, SyncMode,
    SyncPlane,
};
use super::state::{Applied, record_applied, record_declared};
use crate::error::{AdminError, AdminResult};
use crate::repositories::config::groups_yaml_loader::{GROUPS_FILE, apply_groups_doc};
use crate::repositories::config::groups_yaml_types::GroupsDoc;

pub const PLANE_ID: &str = "groups";

#[derive(Debug, Clone, Copy, Default)]
pub struct GroupsPlane;

fn declared_groups_from(from: DeclarationSource<'_>) -> AdminResult<GroupsDoc> {
    // Why: no file declares nothing — the loader treats it the same way.
    let Some(yaml) = read_or_text(from, GROUPS_FILE)? else {
        return Ok(GroupsDoc::default());
    };
    parse_groups_doc(&yaml).map_err(|e| AdminError::invalid("groups.yaml could not be parsed", e))
}

async fn drift_now(pool: &PgPool, doc: &GroupsDoc) -> AdminResult<GroupsDrift> {
    let sets = list_member_sets(pool).await?;
    let mappings = list_mappings(pool).await?;
    Ok(compute_groups_drift(doc, &sets, &mappings))
}

#[async_trait]
impl SyncPlane for GroupsPlane {
    fn id(&self) -> &'static str {
        PLANE_ID
    }

    fn label(&self) -> &'static str {
        "Groups and projects"
    }

    fn source_file(&self) -> &'static str {
        GROUPS_FILE
    }

    fn projection(&self) -> &'static str {
        "groups · projects · group_ad_mappings · project_ad_mappings"
    }

    fn owner_url(&self) -> &'static str {
        "/admin/groups"
    }

    async fn drift(&self, pool: &PgPool) -> AdminResult<PlaneDrift> {
        self.drift_from(pool, DeclarationSource::Disk).await
    }

    async fn drift_from(
        &self,
        pool: &PgPool,
        from: DeclarationSource<'_>,
    ) -> AdminResult<PlaneDrift> {
        let in_db = usize::try_from(count_member_sets(pool).await?).unwrap_or(0);
        let doc = match declared_groups_from(from) {
            Ok(d) => d,
            Err(e) => {
                return Ok(PlaneDrift {
                    in_db,
                    unreadable: Some(e.to_string()),
                    ..PlaneDrift::default()
                });
            },
        };
        let hash = declared_hash(&doc);
        if from.is_disk() {
            record_declared(pool, PLANE_ID, &hash).await?;
        }
        let drift = drift_now(pool, &doc).await?;
        Ok(PlaneDrift {
            declared_hash: hash,
            declared_count: declared_count(&doc),
            in_db,
            kpis: vec![
                DriftKpi {
                    label: "Added in code",
                    value: drift.missing_sets + drift.missing_mappings,
                    note: "Insert only or Overwrite adds these",
                    tone: "ok",
                },
                DriftKpi {
                    label: "Changed in code",
                    value: drift.changed_sets,
                    note: "Overwrite corrects these",
                    tone: "warn",
                },
                DriftKpi {
                    label: "Removed from code",
                    value: drift.orphan_yaml_mappings,
                    note: "Overwrite deletes these mappings",
                    tone: "err",
                },
                DriftKpi {
                    label: "Written in console",
                    value: drift.orphan_sets + drift.orphan_console_mappings,
                    note: "Export carries these to code",
                    tone: "info",
                },
            ],
            actions: ActionCounts {
                insert: drift.missing_sets + drift.missing_mappings,
                insert_entities: 0,
                update: drift.changed_sets,
                delete: drift.orphan_yaml_mappings,
                delete_console: 0,
                kept: drift.orphan_sets + drift.orphan_console_mappings,
            },
            is_clean: drift.is_clean(),
            rows: drift.rows,
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
        let doc = declared_groups_from(from)?;
        let before = drift_now(pool, &doc).await?;
        let outcome = match mode {
            SyncMode::InsertOnly => {
                let done = insert_missing(pool, &doc).await?;
                ApplyOutcome {
                    inserted: done.sets + done.mappings,
                    ..ApplyOutcome::default()
                }
            },
            SyncMode::Overwrite => {
                apply_groups_doc(pool, &doc).await?;
                ApplyOutcome {
                    inserted: before.missing_sets + before.missing_mappings,
                    updated: before.changed_sets,
                    deleted: before.orphan_yaml_mappings,
                    ..ApplyOutcome::default()
                }
            },
        };
        record_applied(
            pool,
            &Applied {
                plane: PLANE_ID,
                declared_hash: &declared_hash(&doc),
                mode: actor.recorded_mode(mode),
                actor: actor.as_str(),
                from: applied_from(),
            },
        )
        .await?;
        Ok(outcome)
    }

    async fn export(&self, pool: &PgPool) -> AdminResult<Option<Export>> {
        let sets = list_member_sets(pool).await?;
        let mappings = list_mappings(pool).await?;
        Ok(Some(Export {
            filename: "groups.yaml",
            content_type: "application/yaml; charset=utf-8",
            row_count: sets.len() + mappings.len(),
            body: render_groups_export(&sets, &mappings),
        }))
    }
}
