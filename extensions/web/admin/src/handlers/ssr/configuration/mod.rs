//! `/admin/configuration` — what this instance is running, where each piece
//! comes from, and whether the database agrees with the code.
//!
//! One row per kind of configuration under `services/`. A projected kind
//! shows its plane's state — in step, drifting, never applied — and links
//! to the Sync tab on the page that owns it; a kind served from code shows
//! the source that ships it and the hash it declares, which is the whole
//! of its state: the tree is read, nothing is copied. The page is the
//! Platform group's home because it is the one honest answer to "what is
//! configured here".

pub(crate) mod retention;
mod rows;
mod view;

use std::path::Path;
use std::sync::Arc;

use axum::extract::{Extension, Query, State};
use axum::response::Response;
use serde::Deserialize;
use sqlx::PgPool;
use systemprompt::config::ProfileBootstrap;
use systemprompt::loader::ServicesBootstrap;

use self::retention::RetentionView;
use self::view::{ConfigKpiView, ConfigRowView, ConfigurationPageData};
use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::ssr_sync::{DOCS_URL, EXPORT_ZIP_URL, empty_sources};
use crate::handlers::ssr::sync_plane::HashView;
use crate::handlers::ssr::types::{BreadcrumbView, TabLinkView};
use crate::repositories::gateway_policies::declared::services_root;
use crate::repositories::sync::provenance::active_bundle_files;
use crate::repositories::sync::sources::build_sources;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

pub(crate) const BASE_URL: &str = "/admin/configuration";

#[derive(Debug, Default, Deserialize)]
pub(crate) struct ConfigurationQuery {
    pub tab: Option<String>,
}

fn tabs(active: &str) -> Vec<TabLinkView> {
    [
        ("all", "All", BASE_URL.to_owned()),
        (
            "projected",
            "Projected",
            format!("{BASE_URL}?tab=projected"),
        ),
        ("code", "Served from code", format!("{BASE_URL}?tab=code")),
    ]
    .into_iter()
    .map(|(slug, label, href)| TabLinkView {
        slug,
        label,
        href,
        is_active: slug == active,
        count: None,
    })
    .collect()
}

fn kpis(all: &[ConfigRowView]) -> Vec<ConfigKpiView> {
    let projected = all.iter().filter(|r| r.is_projected).count();
    let drifting = all
        .iter()
        .filter(|r| r.is_projected && r.state != "in step")
        .count();
    let kit_shipped = all
        .iter()
        .filter(|r| !r.is_projected && r.source != "base")
        .count();
    vec![
        ConfigKpiView {
            label: "Projected into the database",
            value: projected,
            note: "the database is what is enforced",
            tone: "info",
        },
        ConfigKpiView {
            label: "Need attention",
            value: drifting,
            note: "drifting, never applied or unreadable",
            tone: if drifting > 0 { "warn" } else { "ok" },
        },
        ConfigKpiView {
            label: "Served from code",
            value: all.len() - projected,
            note: "read from the composed tree; nothing copied",
            tone: "muted",
        },
        ConfigKpiView {
            label: "Shipped by a bundle",
            value: kit_shipped,
            note: "owned by a pinned kit, in part or whole",
            tone: "info",
        },
    ]
}

pub(crate) async fn configuration_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
    Query(query): Query<ConfigurationQuery>,
) -> AdminHtmlResult<Response> {
    if !user_ctx.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let (sources, sources_unreadable) = match build_sources() {
        Ok(s) => (s, None),
        Err(e) => (empty_sources(), Some(e.to_string())),
    };
    let profile = ProfileBootstrap::get().map_err(AdminError::internal)?;
    let active_root = services_root()?;
    let baked_root = Path::new(&profile.paths.services);
    // Why: discard-ok: an unreadable cache means every row reads as base
    let bundles = active_bundle_files().unwrap_or_default();
    let all = rows::build_rows(&pool, &active_root, baked_root, &bundles).await?;
    // Why: discard-ok: an unreadable services tree leaves the messages window at
    // core's default
    let ai_history_days = ServicesBootstrap::get().map_or(30, |s| s.ai.history.retention_days);
    let retention: RetentionView =
        retention::retention_view(&pool, &profile.retention, ai_history_days).await?;

    let kpis = kpis(&all);

    let tab = query.tab.as_deref().unwrap_or("all");
    let rows = all
        .into_iter()
        .filter(|r| match tab {
            "projected" => r.is_projected,
            "code" => !r.is_projected,
            _ => true,
        })
        .collect();

    let page = ConfigurationPageData {
        page: "configuration",
        title: "Configuration",
        can_write: user_ctx.is_admin,
        breadcrumbs: vec![
            BreadcrumbView::link("Admin", "/admin"),
            BreadcrumbView::current("Configuration"),
        ],
        tabs: tabs(tab),
        release: sources.base.version,
        base_tree_hash: sources.base.tree_hash.as_deref().and_then(HashView::of),
        composed_hash: sources.composed_hash.as_deref().and_then(HashView::of),
        provenance: sources.base.provenance,
        bundle_count: sources.bundles.len(),
        sources_unreadable,
        kpis,
        rows,
        retention,
        sync_url: super::ssr_sync::BASE_URL,
        export_zip_url: EXPORT_ZIP_URL,
        docs_url: DOCS_URL,
    };
    Ok(super::render_typed_page(
        &engine,
        "configuration",
        &page,
        &user_ctx,
        &mkt_ctx,
    ))
}
