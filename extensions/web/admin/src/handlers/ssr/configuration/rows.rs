//! Builds the configuration table: one row per kind, projected kinds from
//! their planes and everything else from the tree itself.

use std::path::Path;

use sqlx::PgPool;

use super::view::ConfigRowView;
use crate::error::AdminResult;
use crate::handlers::ssr::sync_plane::{HashView, PlaneCardView, SYNC_TAB, plane_card_by_id};
use crate::repositories::sync::inventory::{CONFIG_KINDS, ConfigKind};
use crate::repositories::sync::provenance::{BundleFiles, source_for_path};
use crate::repositories::sync::registry::planes;
use crate::repositories::sync::tree_hash::subtree_hash;

fn projected_state(card: &PlaneCardView) -> (&'static str, &'static str, String) {
    if card.applied.is_none() {
        (
            "never applied",
            "warn",
            "no apply recorded on this database".to_owned(),
        )
    } else if card.is_clean {
        (
            "in step",
            "ok",
            format!(
                "{} declared · {} in this database",
                card.declared_count, card.in_db
            ),
        )
    } else {
        (
            "drift",
            "warn",
            format!(
                "{} difference(s) · {} declared · {} in this database",
                card.row_count, card.declared_count, card.in_db
            ),
        )
    }
}

fn projected_row(kind: &ConfigKind, card: &PlaneCardView) -> ConfigRowView {
    let (state, tone, note) = card.unreadable.as_ref().map_or_else(
        || projected_state(card),
        |reason| ("unreadable", "err", reason.clone()),
    );
    ConfigRowView {
        id: kind.id,
        label: kind.label,
        purpose: kind.purpose,
        path: format!("services/{}", kind.path),
        is_dir: kind.is_dir,
        files: 1,
        source: "base".to_owned(),
        source_tone: "ok",
        hash: card.declared_hash.clone(),
        is_projected: true,
        projection: Some(card.projection),
        state,
        state_tone: tone,
        state_note: note,
        applied: card.applied.clone(),
        link: Some(format!("{}?tab={SYNC_TAB}", card.owner_url)),
        link_label: "Sync",
    }
}

fn served_row(
    kind: &ConfigKind,
    active_root: &Path,
    baked_root: &Path,
    bundles: &[BundleFiles],
) -> ConfigRowView {
    let hashed = subtree_hash(active_root, kind.path, kind.is_dir);
    let base_has = baked_root.join(kind.path).exists();
    let source = source_for_path(bundles, kind.path, kind.is_dir, base_has);
    let (state, tone, note) = hashed.as_ref().map_or_else(
        || ("absent", "muted", "not in the composed tree".to_owned()),
        |h| {
            (
                "served",
                "muted",
                format!("read from the tree; {} file(s)", h.files),
            )
        },
    );
    ConfigRowView {
        id: kind.id,
        label: kind.label,
        purpose: kind.purpose,
        path: format!("services/{}", kind.path),
        is_dir: kind.is_dir,
        files: hashed.as_ref().map_or(0, |h| h.files),
        source: source.source,
        source_tone: source.tone,
        hash: hashed.and_then(|h| HashView::of(&h.hash)),
        is_projected: false,
        projection: None,
        state,
        state_tone: tone,
        state_note: note,
        applied: None,
        link: kind.page_url.map(str::to_owned),
        link_label: "Open",
    }
}

// Why: every row, projected kinds first, in inventory order.
pub(super) async fn build_rows(
    pool: &PgPool,
    active_root: &Path,
    baked_root: &Path,
    bundles: &[BundleFiles],
) -> AdminResult<Vec<ConfigRowView>> {
    let registry = planes();
    let mut rows = Vec::with_capacity(CONFIG_KINDS.len());
    for kind in CONFIG_KINDS {
        // Why: a plane is the authority on its own kind; the inventory only
        // knows the kind exists. A registered plane the inventory does not
        // list is added below so a new plane shows up with no page change.
        if let Some(plane_id) = kind.plane {
            if let Some(card) = plane_card_by_id(pool, plane_id, "").await? {
                rows.push(projected_row(kind, &card));
            }
            continue;
        }
        rows.push(served_row(kind, active_root, baked_root, bundles));
    }
    for plane in &registry {
        if CONFIG_KINDS.iter().any(|k| k.plane == Some(plane.id())) {
            continue;
        }
        if let Some(card) = plane_card_by_id(pool, plane.id(), "").await? {
            let kind = ConfigKind {
                id: plane.id(),
                label: plane.label(),
                purpose: plane.projection(),
                path: plane.source_file(),
                is_dir: false,
                plane: Some(plane.id()),
                page_url: Some(plane.owner_url()),
            };
            rows.insert(
                rows.iter().filter(|r| r.is_projected).count(),
                projected_row(&kind, &card),
            );
        }
    }
    Ok(rows)
}
