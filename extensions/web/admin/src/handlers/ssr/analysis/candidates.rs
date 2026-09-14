//! Human-authored candidates preserve the baseline and require a new review.

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::evaluation_experiments::require_write_origin;
use crate::routes::managed_state::ManagedState;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};
use axum::extract::Path;
use axum::http::HeaderMap;
use axum::response::{IntoResponse, Redirect, Response};
use axum::{Extension, Form};
use serde::Serialize;
use std::sync::Arc;
use systemprompt::identifiers::ResourceRevisionId;
use systemprompt::marketplace::managed::{
    RevisionComparison, RevisionFiles, TextCandidate, normalize_form_text,
};

pub(crate) async fn edit_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(id): Path<ResourceRevisionId>,
) -> AdminHtmlResult<Response> {
    require_manage(&user)?;
    let mut files = state
        .repository
        .get_revision_files(&state.owner, &id)
        .await
        .map_err(AdminError::from)?;
    let file = files
        .0
        .remove("SKILL.md")
        .ok_or_else(|| AdminError::BadRequest("This revision has no SKILL.md file".to_owned()))?;
    if file.bytes.len() > 1024 * 1024 {
        return Err(
            AdminError::BadRequest("This file exceeds the text editor limit".to_owned()).into(),
        );
    }
    let content = String::from_utf8(file.bytes).map_err(|_error| {
        AdminError::BadRequest("Instruction file is not UTF-8 text".to_owned())
    })?;
    let content = normalize_form_text(&content, &content).map_err(AdminError::from)?;
    let rendered = CandidateFormContext {
        page: "analysis-versions",
        title: "Draft a skill revision",
        baseline: id,
        content,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-candidate",
        &rendered,
        &user,
        &marketplace,
    ))
}

pub(crate) async fn save_candidate(
    Extension(user): Extension<UserContext>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path(baseline): Path<ResourceRevisionId>,
    headers: HeaderMap,
    Form(mut input): Form<TextCandidate>,
) -> AdminHtmlResult<Response> {
    require_manage(&user)?;
    require_write_origin(&headers)?;
    let files = state
        .repository
        .get_revision_files(&state.owner, &baseline)
        .await
        .map_err(AdminError::from)?;
    let original = files
        .0
        .get(&input.path)
        .ok_or_else(|| AdminError::BadRequest("Candidate file is absent".to_owned()))?;
    let original = std::str::from_utf8(&original.bytes).map_err(|_error| {
        AdminError::BadRequest("Instruction file is not UTF-8 text".to_owned())
    })?;
    input.content = normalize_form_text(original, &input.content).map_err(AdminError::from)?;
    let candidate = state
        .repository
        .create_text_candidate(&state.owner, &baseline, &input)
        .await
        .map_err(AdminError::from)?;
    Ok(Redirect::to(&format!(
        "/admin/analysis/revisions/{baseline}/compare/{candidate}"
    ))
    .into_response())
}

pub(crate) async fn comparison_page(
    Extension(user): Extension<UserContext>,
    Extension(marketplace): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    Extension(state): Extension<Arc<ManagedState>>,
    Path((baseline, candidate)): Path<(ResourceRevisionId, ResourceRevisionId)>,
) -> AdminHtmlResult<Response> {
    require_manage(&user)?;
    let comparison = state
        .repository
        .compare_revisions(&state.owner, &baseline, &candidate)
        .await
        .map_err(AdminError::from)?;
    let before = instruction_preview(
        state
            .repository
            .get_revision_files(&state.owner, &baseline)
            .await
            .map_err(AdminError::from)?,
    );
    let after = instruction_preview(
        state
            .repository
            .get_revision_files(&state.owner, &candidate)
            .await
            .map_err(AdminError::from)?,
    );
    let rendered = RevisionComparisonContext {
        page: "analysis-versions",
        title: "Compare skill revisions",
        comparison,
        before,
        after,
    };
    Ok(super::super::render_typed_page(
        &engine,
        "analysis-comparison",
        &rendered,
        &user,
        &marketplace,
    ))
}

#[derive(Serialize)]
struct Preview {
    text: Option<String>,
    truncated: bool,
}

fn instruction_preview(mut files: RevisionFiles) -> Preview {
    let text = files
        .0
        .remove("SKILL.md")
        .and_then(|file| String::from_utf8(file.bytes).ok());
    text.map_or(
        Preview {
            text: None,
            truncated: false,
        },
        |text| {
            let mut end = text.len().min(64 * 1024);
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            Preview {
                truncated: end < text.len(),
                text: Some(text[..end].to_owned()),
            }
        },
    )
}

fn require_manage(user: &UserContext) -> Result<(), AdminError> {
    if !user.is_admin {
        return Err(AdminError::Forbidden(
            "Administrator access required".to_owned(),
        ));
    }
    Ok(())
}

#[derive(Serialize)]
struct CandidateFormContext {
    page: &'static str,
    title: &'static str,
    baseline: ResourceRevisionId,
    content: String,
}

#[derive(Serialize)]
struct RevisionComparisonContext {
    page: &'static str,
    title: &'static str,
    comparison: RevisionComparison,
    before: Preview,
    after: Preview,
}
