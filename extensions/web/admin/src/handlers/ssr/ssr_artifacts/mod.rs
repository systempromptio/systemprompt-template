//! `/admin/artifacts` — every tool result a person can view or retrieve
//! afterwards, as one entity: a file edited, written or read; a typed card;
//! an MCP Apps UI resource; a retained body with a preview. Which tool
//! results count is the one artifact rule (`artifact_kind`, schema 46), so a
//! Bash, Grep or search result is never listed here and every page counts
//! artifacts the same way.
//!
//! The page is the Tools page (`ssr_tools`) through the artifact lens: same
//! query type, filter, statement, rows and context, narrowed to rows with a
//! kind, plus a kind facet and the preview affordance per row.

mod detail;
mod preview;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::ssr_tools::{Frame, Lens, ToolsQuery, build_context, help, read_lens};
use crate::handlers::ssr::types::BreadcrumbView;

pub(crate) use detail::artifact_detail_page;
pub(crate) use preview::artifact_preview;

pub(crate) async fn artifacts_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(q): Query<ToolsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Admin access required.".to_owned()).into());
    }
    let read = read_lens(&pool, &shell.user, Lens::Artifacts, &q).await?;
    let context = build_context(
        Lens::Artifacts,
        &q,
        &read.data,
        Frame {
            range: read.range,
            scope_filter: read.scope_filter,
            help: help::artifacts_help(),
            breadcrumbs: vec![
                BreadcrumbView::link("AI activity", "/admin/analytics"),
                BreadcrumbView::link("Tools & artifacts", "/admin/tools"),
                BreadcrumbView::current("Artifacts"),
            ],
            noun: "artifacts",
            datasets: &["artifacts", "artifacts-breakdown"],
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "artifacts",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}
