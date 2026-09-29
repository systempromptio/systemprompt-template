//! Activity constructors for `/admin/sync`: each apply, export and source
//! refresh leaves a row saying who, which plane, which mode, and what moved.

use systemprompt::identifiers::UserId;

use super::super::enums::{ActivityAction, ActivityCategory, ActivityEntity};
use super::super::types::{ActivityEntityRef, NewActivity};
use crate::repositories::sync::plane::{ApplyOutcome, EntityScope, SyncMode};

// Why: one apply as the trail records it — the plane, its label, the mode
// and what moved — shared by the disk and archive constructors.
#[derive(Debug, Clone, Copy)]
pub struct PlaneApply<'a> {
    pub plane: &'a str,
    pub label: &'a str,
    pub mode: SyncMode,
    pub outcome: &'a ApplyOutcome,
    pub reason: Option<&'a str>,
}

fn plane_ref(plane: &str, label: &str) -> ActivityEntityRef {
    ActivityEntityRef {
        kind: ActivityEntity::Sync,
        id: Some(plane.to_owned()),
        name: Some(label.to_owned()),
    }
}

impl NewActivity {
    // Why: a scoped apply names the marketplaces it was narrowed to, both in
    // the sentence and in `metadata.marketplaces`, which is what the sync
    // history filters a participant's view by.
    #[must_use]
    pub fn sync_applied(user_id: &UserId, apply: PlaneApply<'_>, scope: EntityScope<'_>) -> Self {
        let PlaneApply {
            plane,
            label,
            mode,
            outcome,
            reason,
        } = apply;
        let marketplaces: Vec<&str> = scope
            .keys()
            .iter()
            .filter_map(|k| k.strip_prefix("marketplace/"))
            .collect();
        let scoped = if marketplaces.is_empty() {
            match scope.keys() {
                [] => String::new(),
                keys => format!(" for {}", keys.join(", ")),
            }
        } else {
            format!(" for {}", marketplaces.join(", "))
        };
        let because = reason.map_or_else(String::new, |r| format!(" — {r}"));
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref(plane, label)),
            description: format!(
                "Synced {label} from code{scoped} ({}): +{} ~{} -{}{because}",
                mode.human(),
                outcome.inserted,
                outcome.updated,
                outcome.deleted
            ),
            // JSON: JSONB column — the plane, scope and counts, so the event
            // page can show exactly what moved without re-deriving it
            metadata: serde_json::json!({
                "plane": plane,
                "mode": mode.label(),
                "entity_scope": scope.keys(),
                "marketplaces": marketplaces,
                "inserted": outcome.inserted,
                "updated": outcome.updated,
                "deleted": outcome.deleted,
                "entities_inserted": outcome.entities_inserted,
                "entities_updated": outcome.entities_updated,
                "reason": reason,
            }),
        }
    }

    // Why: "keep the database" writes no rule, so this row is the whole
    // decision. `kept_entity` and `fingerprint` are what the review reads
    // back to hold the entity out of the to-do list until its diff moves.
    #[must_use]
    pub fn sync_kept(user_id: &UserId, key: &str, fingerprint: &str, reason: &str) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Rejected,
            entity: Some(plane_ref("access_control", "Access control")),
            description: format!("Kept the database version of {key} over code — {reason}"),
            // JSON: JSONB column — the decision the review reads back
            metadata: serde_json::json!({
                "plane": "access_control",
                "kept_entity": key,
                "fingerprint": fingerprint,
                "reason": reason,
            }),
        }
    }

    #[must_use]
    pub fn sync_exported(user_id: &UserId, plane: &str, label: &str, row_count: usize) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Uploaded,
            entity: Some(plane_ref(plane, label)),
            description: format!("Exported {label} ({row_count} rows) as its declaration file"),
            // JSON: JSONB column — the plane and export size
            metadata: serde_json::json!({ "plane": plane, "row_count": row_count }),
        }
    }

    // Why: a console edit to `ai_gateway_policies` is a write to the
    // gateway_policies plane by another door; it is filed under the plane so
    // the sync page's history and this row read as one story.
    #[must_use]
    pub fn gateway_policy_saved(user_id: &UserId, policy: &str, deleted: bool) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: if deleted {
                ActivityAction::Deleted
            } else {
                ActivityAction::Updated
            },
            entity: Some(plane_ref("gateway_policies", "Gateway policies")),
            description: if deleted {
                format!("Deleted gateway policy '{policy}' from the console")
            } else {
                format!("Saved gateway policy '{policy}' from the console; live within a minute")
            },
            // JSON: JSONB column — which policy row moved
            metadata: serde_json::json!({ "plane": "gateway_policies", "policy": policy }),
        }
    }

    #[must_use]
    pub fn configuration_exported(user_id: &UserId, planes: usize, rows: usize) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Uploaded,
            entity: Some(plane_ref("archive", "configuration archive")),
            description: format!(
                "Exported the configuration archive: {planes} plane(s), {rows} rows"
            ),
            // JSON: JSONB column — how much the archive carried
            metadata: serde_json::json!({ "planes": planes, "rows": rows }),
        }
    }

    // Why: an apply from an uploaded archive is filed under the plane like a
    // disk apply, with the stage id so the trail says which upload it was.
    #[must_use]
    pub fn configuration_imported(user_id: &UserId, stage_id: &str, apply: PlaneApply<'_>) -> Self {
        let PlaneApply {
            plane,
            label,
            mode,
            outcome,
            ..
        } = apply;
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref(plane, label)),
            description: format!(
                "Imported {label} from an uploaded archive ({}): +{} ~{} -{}",
                mode.human(),
                outcome.inserted,
                outcome.updated,
                outcome.deleted
            ),
            // JSON: JSONB column — the stage, plane, mode and counts
            metadata: serde_json::json!({
                "stage_id": stage_id,
                "plane": plane,
                "mode": mode.label(),
                "inserted": outcome.inserted,
                "updated": outcome.updated,
                "deleted": outcome.deleted,
            }),
        }
    }

    #[must_use]
    pub fn sources_refreshed(user_id: &UserId, changed: bool, reconciled: bool) -> Self {
        Self {
            user_id: user_id.clone(),
            category: ActivityCategory::UserManagement,
            action: ActivityAction::Imported,
            entity: Some(plane_ref("sources", "services sources")),
            description: if changed {
                "Imported services sources: composition changed and reconciled in place".to_owned()
            } else {
                "Refreshed services sources: unchanged".to_owned()
            },
            // JSON: JSONB column — whether the composition moved and whether
            // the new composition was projected into the authz tables
            metadata: serde_json::json!({ "changed": changed, "reconciled": reconciled }),
        }
    }
}
