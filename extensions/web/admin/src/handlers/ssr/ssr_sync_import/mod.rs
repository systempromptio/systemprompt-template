//! `/admin/sync/import/{stage}` — what an uploaded archive would do to this
//! database, plane by plane, before any of it is applied.
//!
//! The stage holds each plane's file as text; every plane's drift is
//! computed from that text through the same code the owner page uses for
//! the file on disk, so the preview is the exact picture an apply would
//! act on. Entries the instance does not project are listed for what they
//! are — things to commit to the repository or publish as a bundle — and
//! projected planes the archive did not carry are named so a partial
//! archive is never mistaken for a full one.

mod view;

use std::sync::Arc;

use axum::extract::{Extension, Path, State};
use axum::response::Response;
use sqlx::PgPool;

use self::view::{AbsentPlaneView, ImportPreviewPageData, ManifestView, PreviewKpiView};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::sync_plane::{HashView, PlaneCardOpts, PlaneCardView, plane_card};
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories::sync::archive::staging::StagingStore;
use crate::repositories::sync::plane::DeclarationSource;
use crate::repositories::sync::registry::planes;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

fn kpis(cards: &[PlaneCardView], other: usize) -> Vec<PreviewKpiView> {
    let clean = cards.iter().filter(|c| c.is_clean).count();
    vec![
        PreviewKpiView {
            label: "Planes in the archive",
            value: cards.len(),
            note: "each previewed against this database",
            tone: "info",
        },
        PreviewKpiView {
            label: "Already in step",
            value: clean,
            note: "an apply would move nothing",
            tone: "ok",
        },
        PreviewKpiView {
            label: "Would change",
            value: cards.len() - clean,
            note: "choose a direction per plane",
            tone: if cards.len() > clean { "warn" } else { "ok" },
        },
        PreviewKpiView {
            label: "Served from code",
            value: other,
            note: "not applied here; commit or publish",
            tone: "muted",
        },
    ]
}

pub(crate) async fn import_preview_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Path(stage_id): Path<String>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let Some(stage) = StagingStore::global().get(&stage_id) else {
        return Err(AdminError::NotFound(
            "No staged import with that id — it may have expired.".to_owned(),
        )
        .into());
    };
    let apply_url = format!("/api/public/admin/sync/import/{}/apply", stage.id);
    let mut cards = Vec::new();
    let mut absent = Vec::new();
    for plane in planes() {
        match stage.planes.get(plane.id()) {
            Some(text) => {
                let opts = PlaneCardOpts {
                    entity_filter: "",
                    apply_url: Some(apply_url.clone()),
                    show_export: false,
                    source_label: Some("the archive"),
                };
                cards.push(
                    plane_card(&pool, plane.as_ref(), DeclarationSource::Text(text), &opts).await?,
                );
            },
            None => absent.push(AbsentPlaneView {
                label: plane.label(),
                source_file: plane.source_file(),
                owner_url: plane.owner_url(),
            }),
        }
    }
    let kpis = kpis(&cards, stage.other.len());
    let manifest = stage.manifest.as_ref().map(|m| ManifestView {
        release: m.release.clone(),
        exported_at: m.exported_at.to_rfc3339(),
        exported_by: m.exported_by.clone(),
        base_tree_hash: m.base_tree_hash.as_deref().and_then(HashView::of),
        composed_hash: m.composed_hash.as_deref().and_then(HashView::of),
        planes: m.planes.len(),
    });
    let page = ImportPreviewPageData {
        page: "sync",
        title: "Import preview",
        can_write: user_ctx.is_admin,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Code sync", super::ssr_sync::BASE_URL),
            BreadcrumbView::current("Import preview"),
        ],
        stage_id: stage.id.clone(),
        staged_by: stage.actor.clone(),
        staged_at: stage.created_at.to_rfc3339(),
        expires_at: stage.expires_at().to_rfc3339(),
        manifest,
        manifest_error: stage.manifest_error.clone(),
        kpis,
        planes: cards,
        absent,
        other_count: stage.other.len(),
        other: stage.other.clone(),
        apply_all_url: apply_url,
        discard_url: format!("/api/public/admin/sync/import/{}", stage.id),
        sync_url: super::ssr_sync::BASE_URL,
        docs_url: super::ssr_sync::DOCS_URL,
    };
    Ok(super::render_typed_page(
        &engine,
        "sync-import",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
