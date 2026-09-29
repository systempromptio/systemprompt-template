//! `/admin/sync` — Code sync: where declarations come from and how they
//! move between the repository and this instance.
//!
//! Three tabs. *Sources* lists every place a declaration comes from in one
//! table — the internal base tree this repository ships and each external
//! bundle the profile pins — with the hash each declares and what it owns,
//! and offers the in-place import that recomposes them. *Access review* is
//! the access-control plane, entity by entity: what code and this database
//! disagree about, settled one row at a time with a stated reason, above the
//! whole-plane card. *Export & import*
//! moves the projected planes as one archive: the database rendered back
//! to its files for a commit, or an uploaded archive staged for a preview
//! on `/admin/sync/import/{stage}`. What each plane's database says
//! against its file lives on the plane's owner page, under its Sync tab.
//!
//! This instance has no marketplace participant tier, so the page is the
//! console's alone and every write on it is an administrator's.

mod access_review;
mod view;

use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;

use self::view::{ExportPlaneView, SyncHistoryView, SyncPageData};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::sync_plane::HashView;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::sync::archive::staging::STAGE_TTL_MINUTES;
use crate::repositories::sync::attention::access_attention;
use crate::repositories::sync::history::{SyncHistoryScope, list_sync_history};
use crate::repositories::sync::registry::planes;
use crate::repositories::sync::sources::{SourcesView, build_sources};
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) const DOCS_URL: &str = "/documentation/services-sync";
pub(crate) const BASE_URL: &str = "/admin/sync";
pub(crate) const EXPORT_ZIP_URL: &str = "/api/public/admin/sync/export.zip";
pub(crate) const IMPORT_URL: &str = "/api/public/admin/sync/import";
const HISTORY_LIMIT: i64 = 50;

#[derive(Debug, Default, Deserialize)]
pub(crate) struct SyncQuery {
    pub tab: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SyncTab {
    Sources,
    Access,
    Archive,
}

impl SyncTab {
    fn parse(tab: Option<&str>) -> Self {
        match tab {
            Some("access") => Self::Access,
            Some("archive") => Self::Archive,
            _ => Self::Sources,
        }
    }
}

fn tabs(active: SyncTab, attention: usize) -> Vec<TabLinkView> {
    vec![
        TabLinkView {
            slug: "sources",
            label: "Sources",
            href: BASE_URL.to_owned(),
            is_active: active == SyncTab::Sources,
            count: None,
        },
        TabLinkView {
            slug: "access",
            label: "Access review",
            href: format!("{BASE_URL}?tab=access"),
            is_active: active == SyncTab::Access,
            count: i64::try_from(attention).ok().filter(|n| *n > 0),
        },
        TabLinkView {
            slug: "archive",
            label: "Export & import",
            href: format!("{BASE_URL}?tab=archive"),
            is_active: active == SyncTab::Archive,
            count: None,
        },
    ]
}

// Why: an unreadable profile or cache is the panel's headline, not a
// 500 — the rest of the page still renders.
fn visible_sources() -> (SourcesView, Option<String>) {
    match build_sources() {
        Ok(s) => (s, None),
        Err(e) => (empty_sources(), Some(e.to_string())),
    }
}

pub(crate) async fn sync_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<SyncQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Console access required.".to_owned()).into());
    }
    let (sources, sources_unreadable) = visible_sources();
    let history = list_sync_history(&pool, SyncHistoryScope::All, HISTORY_LIMIT).await?;
    let tab = SyncTab::parse(query.tab.as_deref());
    let access_review = if tab == SyncTab::Access {
        Some(access_review::access_review(&pool).await?)
    } else {
        None
    };
    let export_planes = planes()
        .iter()
        .map(|p| ExportPlaneView {
            id: p.id(),
            label: p.label(),
            source_file: p.source_file(),
            owner_url: p.owner_url(),
        })
        .collect();

    let page = SyncPageData {
        page: "sync",
        title: "Code sync",
        can_write: user_ctx.is_admin,
        can_refresh: user_ctx.is_admin,
        history: history.into_iter().map(SyncHistoryView::from).collect(),
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::link("Platform", "/admin/configuration"),
            BreadcrumbView::current("Code sync"),
        ],
        tabs: tabs(tab, access_attention()),
        show_sources: tab == SyncTab::Sources,
        show_access: tab == SyncTab::Access,
        access_review,
        docs_url: DOCS_URL,
        configuration_url: "/admin/configuration",
        observability_url: crate::handlers::ssr::ssr_observability::PAGE_URL,
        export_zip_url: EXPORT_ZIP_URL,
        import_url: IMPORT_URL,
        composed_hash: sources.composed_hash.as_deref().and_then(HashView::of),
        last_reconciled_hash: sources
            .last_reconciled_hash
            .as_deref()
            .and_then(HashView::of),
        base_tree_hash: sources.base.tree_hash.as_deref().and_then(HashView::of),
        sources,
        sources_unreadable,
        export_planes,
        stage_ttl_minutes: STAGE_TTL_MINUTES,
    };
    Ok(super::render_typed_page(
        &engine, "sync", &page, &user_ctx, &mkt_ctx,
    ))
}

pub(crate) const fn empty_sources() -> SourcesView {
    SourcesView {
        base: crate::repositories::sync::sources::BaseSourceView {
            version: env!("CARGO_PKG_VERSION"),
            tree_hash: None,
            tree_path: String::new(),
            provenance: "unknown",
            provenance_error: None,
        },
        bundles: Vec::new(),
        composed_hash: None,
        last_reconciled_hash: None,
        restart_pending: false,
        on_fetch_failure: String::new(),
        refresh_hint: String::new(),
    }
}
