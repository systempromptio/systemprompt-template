//! `/admin/tools` — every tool call the platform saw, as the ledger shows it:
//! when, which tool on which server, what it was about, whether it ran and
//! how long it took, what governance decided, who made it, the conversation
//! and session it belongs to, and the artifact it produced if the one
//! artifact rule (schema 46) says it did.
//!
//! The Artifacts page (`ssr_artifacts`) is this page seen through the other
//! lens — the same query type, filter, repository statement, rows and
//! context, narrowed to rows with an artifact kind — so the two never
//! disagree about what a tool call or an artifact is.

mod context;
mod figures;
pub(crate) mod help;
mod links;
mod query;
mod ribbon;
pub(crate) mod rows;

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::response::Response;
use sqlx::PgPool;

use crate::error::{AdminError, AdminHtmlResult, AdminResult};
use crate::handlers::ssr::list_view::{ScopeFilterView, scope_filter_view};
use crate::handlers::ssr::page::Page;
use crate::handlers::ssr::types::BreadcrumbView;
use crate::repositories;
use crate::repositories::analysis::tools::{
    ToolActivityPage, ToolActivityResult, load_tool_activity_page,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use crate::handlers::ssr::list_view::DEFAULT_PAGE_SIZE;
pub(crate) use context::{Frame, build_context};
pub(crate) use query::{Lens, ToolsQuery};

// Why: what a lens page reads before it builds its context — the data and
// the scope form, resolved against the caller's visibility.
pub(crate) struct LensRead {
    pub data: ToolActivityResult,
    pub scope_filter: ScopeFilterView,
    pub range: TimeRange,
}

pub(crate) async fn read_lens(
    pool: &PgPool,
    user: &UserContext,
    lens: Lens,
    q: &ToolsQuery,
) -> AdminResult<LensRead> {
    let range = q.range();
    let request = ScopeRequest::from_query(user, q.group.as_deref(), q.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let filter = q.filter(lens, range, scope.as_sql());
    let data = load_tool_activity_page(
        pool,
        &filter,
        ToolActivityPage {
            sort: q.sort(),
            descending: q.descending(),
            limit: DEFAULT_PAGE_SIZE,
            offset: q.page() * DEFAULT_PAGE_SIZE,
            breakdown: q.breakdown(lens),
        },
    )
    .await?;
    let scope_filter = scope_filter_view(
        pool,
        user,
        &request,
        lens.base_url(),
        q.preserved(&["group", "project", "page"]),
    )
    .await;
    Ok(LensRead {
        data,
        scope_filter,
        range,
    })
}

pub(crate) async fn tools_page(
    shell: Page,
    State(pool): State<Arc<PgPool>>,
    Query(q): Query<ToolsQuery>,
) -> AdminHtmlResult<Response> {
    if !shell.user.is_console {
        return Err(AdminError::Forbidden("Console access required".into()).into());
    }
    let read = read_lens(&pool, &shell.user, Lens::Tools, &q).await?;
    let context = build_context(
        Lens::Tools,
        &q,
        &read.data,
        Frame {
            range: read.range,
            scope_filter: read.scope_filter,
            help: help::tools_help(),
            breadcrumbs: vec![
                BreadcrumbView::link("AI activity", "/admin/analytics"),
                BreadcrumbView::link("Tools & artifacts", "/admin/tools"),
                BreadcrumbView::current("Tools"),
            ],
            noun: "tool calls",
            datasets: &["tools", "tools-breakdown"],
        },
    );
    Ok(crate::handlers::ssr::render_typed_page(
        &shell.engine,
        "tools",
        &context,
        &shell.user,
        &shell.marketplace,
    ))
}

// Why: the export reads the rows the page shows — same scope, window and
// filters — through the page's own query type, with `ids=` narrowing to a
// selection when the dialog was opened from the bulk bar.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user: &UserContext,
    lens: Lens,
    q: ToolsQuery,
    (range, limit): (TimeRange, i64),
) -> AdminResult<ToolActivityResult> {
    let request = ScopeRequest::from_query(user, q.group.as_deref(), q.project.as_deref());
    let scope = repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let filter = q.filter(lens, range, scope.as_sql());
    Ok(load_tool_activity_page(
        pool,
        &filter,
        ToolActivityPage {
            sort: q.sort(),
            descending: q.descending(),
            limit,
            offset: 0,
            breakdown: q.breakdown(lens),
        },
    )
    .await?)
}
