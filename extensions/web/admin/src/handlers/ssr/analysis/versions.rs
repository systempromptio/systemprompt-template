//! Revision history is authoring evidence, separate from measured run outcomes.

use crate::error::{AdminError, AdminHtmlResult};
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::Extension;
use axum::extract::{Path, Query};
use axum::response::Response;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use systemprompt::identifiers::{ManagedResourceId, ResourceRevisionId};
use systemprompt::marketplace::managed::{ResourceSummary, RevisionSummary};

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct RevisionPagination {
    page: Option<u32>,
}

impl RevisionPagination {
    fn offset(&self) -> Result<i64, AdminError> {
        let page = self.page.unwrap_or(1);
        if !(1..=10_001).contains(&page) {
            return Err(AdminError::BadRequest("Invalid page".to_owned()));
        }
        Ok(i64::from(page - 1) * 50)
    }
}

#[derive(Serialize)]
struct VersionPage<T> {
    page: &'static str,
    title: &'static str,
    resources: Vec<T>,
    previous: Option<String>,
    next: Option<String>,
    can_manage: bool,
}

pub(crate) async fn resources_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    Query(query): Query<RevisionPagination>,
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let offset = query.offset()?;
    let mut resources = state
        .repository
        .list_resources(&state.owner, offset)
        .await
        .map_err(AdminError::from)?;
    let next = (resources.len() > 50).then(|| format!("?page={}", offset / 50 + 2));
    resources.truncate(50);
    let rendered = VersionPage::<ManagedResourceView> {
        page: "analysis-versions",
        title: "Skill versions",
        resources: resources
            .into_iter()
            .map(ManagedResourceView::from)
            .collect(),
        previous: (offset > 0).then(|| format!("?page={}", offset / 50)),
        next,
        can_manage: user.is_admin,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-versions",
        &rendered,
        &user,
        &marketplace,
    ))
}

pub(crate) async fn resource_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    (Path(id), Query(query)): (Path<ManagedResourceId>, Query<RevisionPagination>),
) -> AdminHtmlResult<Response> {
    require_console(&user)?;
    let offset = query.offset()?;
    let mut resources = state
        .repository
        .list_revisions(&state.owner, &id, offset)
        .await
        .map_err(AdminError::from)?;
    let next = (resources.len() > 50).then(|| format!("?page={}", offset / 50 + 2));
    resources.truncate(50);
    let rendered = VersionPage::<RevisionHistoryView> {
        page: "analysis-versions",
        title: "Revision history",
        resources: resources
            .into_iter()
            .map(RevisionHistoryView::from)
            .collect(),
        previous: (offset > 0).then(|| format!("?page={}", offset / 50)),
        next,
        can_manage: false,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-revisions",
        &rendered,
        &user,
        &marketplace,
    ))
}

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
        page: "analysis-versions",
        title: "Revision evidence",
        id,
        manifest,
        files,
        can_manage: user.is_admin,
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
struct ManagedResourceView {
    id: ManagedResourceId,
    resource_key: String,
    kind: String,
    source_name: String,
    revision_count: i64,
    latest_revision: Option<ResourceRevisionId>,
}

impl From<ResourceSummary> for ManagedResourceView {
    fn from(row: ResourceSummary) -> Self {
        Self {
            id: row.id,
            resource_key: row.resource_key,
            kind: row.kind,
            source_name: row.source_name,
            revision_count: row.revision_count,
            latest_revision: row.latest_revision,
        }
    }
}

#[derive(Serialize)]
struct RevisionHistoryView {
    id: ResourceRevisionId,
    digest: String,
    parent_id: Option<ResourceRevisionId>,
    rationale: String,
    created_at: String,
}

impl From<RevisionSummary> for RevisionHistoryView {
    fn from(row: RevisionSummary) -> Self {
        Self {
            id: row.id,
            digest: row.digest.as_str().to_owned(),
            parent_id: row.parent_id,
            rationale: row.rationale,
            created_at: row.created_at,
        }
    }
}

#[derive(Serialize)]
struct RevisionEvidenceContext {
    page: &'static str,
    title: &'static str,
    id: ResourceRevisionId,
    manifest: systemprompt::marketplace::managed::RevisionManifest,
    files: Vec<FilePreview>,
    can_manage: bool,
}
