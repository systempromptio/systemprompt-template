//! Revision evidence: the files one captured revision holds, previewed with
//! their digests. Authoring evidence, reached from a marketplace's
//! Distribution view; separate from measured run outcomes.

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::analysis_urls::ANALYSIS_VERSIONS_URL;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::Path;
use axum::response::Response;
use serde::Serialize;
use std::sync::Arc;
use systemprompt::identifiers::ResourceRevisionId;

#[derive(Serialize)]
struct FilePreview {
    path: String,
    digest: String,
    bytes: usize,
    text: Option<String>,
    truncated: bool,
}

pub(crate) async fn revision_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let manifest = state
        .repository
        .get_revision(&state.owner, &id)
        .await
        .map_err(AdminError::from)?;
    let stored = state
        .repository
        .get_revision_files(&state.owner, &id)
        .await
        .map_err(AdminError::from)?;
    let mut remaining = 256 * 1024;
    let files: Vec<_> = stored
        .0
        .into_iter()
        .map(|(path, file)| {
            let bytes = file.bytes.len();
            let take = bytes.min(64 * 1024).min(remaining);
            let text = std::str::from_utf8(&file.bytes).ok().map(|text| {
                let mut end = take;
                while !text.is_char_boundary(end) {
                    end -= 1;
                }
                remaining -= end;
                text[..end].to_owned()
            });
            let digest = systemprompt::marketplace::managed::AssetDigest::of(&file.bytes)
                .as_str()
                .to_owned();
            FilePreview {
                path,
                digest,
                bytes,
                text,
                truncated: take < bytes,
            }
        })
        .collect();
    let rendered = RevisionEvidenceContext {
        page: "analysis-revision",
        title: "Revision evidence",
        breadcrumbs: vec![
            BreadcrumbView::link("Versions", ANALYSIS_VERSIONS_URL),
            BreadcrumbView::current(format!(
                "Revision {}",
                &id.as_str()[..id.as_str().len().min(8)]
            )),
        ],
        id,
        manifest,
        files,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-revision",
        &rendered,
        &user,
        &marketplace,
    ))
}

fn require_console(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required for shared authoring resources".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct RevisionEvidenceContext {
    page: &'static str,
    title: &'static str,
    breadcrumbs: Vec<BreadcrumbView>,
    id: ResourceRevisionId,
    manifest: systemprompt::marketplace::managed::RevisionManifest,
    files: Vec<FilePreview>,
}
