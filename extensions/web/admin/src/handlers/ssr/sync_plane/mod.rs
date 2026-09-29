//! The sync-plane component: one plane's declared hash, last apply, drift
//! totals, three directions and diff table, built the same way wherever it
//! is rendered.
//!
//! An owner page's Sync tab reads the plane from disk and posts to the
//! plane's apply; the import preview reads it from staged text and posts
//! to the stage's apply. The template partial `components/sync-plane` and
//! the JS module `components/sp-sync-plane.js` know only the view built
//! here, so a page adds the component with one call and one partial.

pub(crate) mod view;

use sqlx::PgPool;

pub(crate) use view::{AppliedView, HashView, PlaneCardView, SyncRecommendation};

use crate::error::AdminResult;
use crate::handlers::ssr::types::TabLinkView;
use crate::repositories::sync::plane::{DeclarationSource, SyncPlane};
use crate::repositories::sync::registry::find_plane;
use crate::repositories::sync::state::{SyncStateRow, find_sync_state};

pub(crate) const SYNC_TAB: &str = "sync";

// Why: how a page wants the component built.
#[derive(Debug, Clone, Default)]
pub(crate) struct PlaneCardOpts<'a> {
    // Why: `<kind>/<id>` narrows the diff table to one entity — the
    // access-control ledger's deep link.
    pub entity_filter: &'a str,
    pub apply_url: Option<String>,
    pub show_export: bool,
    pub source_label: Option<&'static str>,
}

pub(crate) fn applied_view(
    state: Option<&SyncStateRow>,
    declared_hash: &str,
) -> Option<AppliedView> {
    let s = state?;
    let hash = s.applied_hash.as_deref()?;
    Some(AppliedView {
        hash: HashView::of(hash),
        mode: s.applied_mode.clone().unwrap_or_default().replace('_', " "),
        at: s.applied_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        by: s.applied_by.clone().unwrap_or_default(),
        base_tree_hash: s.base_tree_hash.as_deref().and_then(HashView::of),
        composed_hash: s.composed_hash.as_deref().and_then(HashView::of),
        declared_changed: !declared_hash.is_empty() && declared_hash != hash,
    })
}

pub(crate) async fn plane_card(
    pool: &PgPool,
    plane: &dyn SyncPlane,
    from: DeclarationSource<'_>,
    opts: &PlaneCardOpts<'_>,
) -> AdminResult<PlaneCardView> {
    let drift = plane.drift_from(pool, from).await?;
    let state = find_sync_state(pool, plane.id()).await?;
    let mut rows = drift.rows;
    if let Some((kind, id)) = opts.entity_filter.split_once('/') {
        rows.retain(|r| r.entity_type == kind && r.entity_id == id);
    }
    let owner_url = plane.owner_url();
    let is_clean = drift.is_clean && drift.unreadable.is_none();
    Ok(PlaneCardView {
        id: plane.id(),
        label: plane.label(),
        source_file: plane.source_file(),
        projection: plane.projection(),
        owner_url,
        runtime_note: plane.runtime_note(),
        declared_hash: HashView::of(&drift.declared_hash),
        declared_count: drift.declared_count,
        in_db: drift.in_db,
        kpis: drift.kpis,
        recommendation: SyncRecommendation::for_actions(&drift.actions, is_clean),
        actions: drift.actions,
        row_count: rows.len(),
        rows,
        is_clean,
        unreadable: drift.unreadable,
        applied: applied_view(state.as_ref(), &drift.declared_hash),
        export_url: format!("/api/public/admin/sync/planes/{}/export", plane.id()),
        apply_url: opts
            .apply_url
            .clone()
            .unwrap_or_else(|| format!("/api/public/admin/sync/planes/{}/apply", plane.id())),
        show_export: opts.show_export,
        entity_filter: opts.entity_filter.to_owned(),
        clear_filter_url: format!("{owner_url}?tab={SYNC_TAB}"),
        source_label: opts.source_label.unwrap_or("the file"),
    })
}

// Why: the owner-page case — disk declaration, the plane's own apply, export
// offered.
pub(crate) async fn plane_card_by_id(
    pool: &PgPool,
    plane_id: &str,
    entity_filter: &str,
) -> AdminResult<Option<PlaneCardView>> {
    let Some(plane) = find_plane(plane_id) else {
        return Ok(None);
    };
    let opts = PlaneCardOpts {
        entity_filter,
        apply_url: None,
        show_export: true,
        source_label: None,
    };
    plane_card(pool, plane.as_ref(), DeclarationSource::Disk, &opts)
        .await
        .map(Some)
}

pub(crate) fn sync_tab(base_href: &str, is_active: bool) -> TabLinkView {
    TabLinkView {
        slug: SYNC_TAB,
        label: "Sync",
        href: format!("{base_href}?tab={SYNC_TAB}"),
        is_active,
        count: None,
    }
}

pub(crate) fn is_sync_tab(tab: Option<&str>) -> bool {
    tab == Some(SYNC_TAB)
}
