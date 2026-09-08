//! The reads behind everything under the header on `/admin/mcp/{id}`.
//!
//! Split out of the handler because six loosely related lists and their
//! pagination is most of what the page does, and a handler that also owns the
//! request flow stops being readable at that size.

use std::sync::Arc;

use sqlx::PgPool;
use systemprompt_security::authz::{AccessControlRepository, EntityKind};

use crate::handlers::ssr::list_view::{PageWindow, Pagination};
use crate::repositories;
use crate::repositories::mcp::runtime;
use crate::types::ENTITY_MCP_SERVER;

use super::rows::{BASE_URL, WINDOW_HOURS};
use super::{detail, view};

const EXECUTIONS_PAGE_SIZE: i64 = 50;
const TOOL_LIMIT: i64 = 50;
const SESSION_LIMIT: i64 = 50;

fn build_pagination(page_index: i64, window: PageWindow, base: &str) -> Pagination {
    let (first_row, last_row) = window.bounds();
    let prev_url = (page_index > 0).then(|| format!("{base}?page={}", page_index - 1));
    let next_url =
        (page_index + 1 < window.total_pages).then(|| format!("{base}?page={}", page_index + 1));
    Pagination {
        current_page: page_index + 1,
        total_pages: window.total_pages,
        first_row,
        last_row,
        total_rows: window.total_rows,
        noun: window.noun,
        has_prev: prev_url.is_some(),
        has_next: next_url.is_some(),
        prev_url,
        next_url,
    }
}

// Why: everything below the header on the detail page, in one read. It is a
// struct rather than a tuple because six loosely related lists returned
// positionally is how the wrong one ends up rendered in the wrong section.
pub(super) struct DetailSections {
    pub tools: Vec<view::McpToolRow>,
    pub executions: Vec<view::McpExecutionRowView>,
    pub executions_total: i64,
    pub pagination: Pagination,
    pub sessions: Vec<view::McpSessionRowView>,
    pub grants: Vec<view::McpGrantRow>,
    pub default_included: bool,
}

// Why: every read is best-effort for the same reason the fleet reads are — a
// section that will not load renders its own empty state rather than taking
// the page down with it.
pub(super) async fn detail_sections(
    pool: &Arc<PgPool>,
    mcp_id: &str,
    page_index: i64,
) -> DetailSections {
    let (executions, executions_total) = runtime::list_mcp_executions_paged(
        pool,
        mcp_id,
        EXECUTIONS_PAGE_SIZE,
        page_index * EXECUTIONS_PAGE_SIZE,
    )
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "mcp: execution log read failed"))
    .unwrap_or_default();
    let shown = i64::try_from(executions.len()).unwrap_or(0);

    let tools = runtime::list_mcp_tool_stats(pool, mcp_id, WINDOW_HOURS, TOOL_LIMIT)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: tool stats read failed"))
        .unwrap_or_default();
    let sessions = runtime::list_mcp_sessions_for_server(pool, mcp_id, SESSION_LIMIT)
        .await
        .inspect_err(|e| tracing::warn!(error = %e, "mcp: session read failed"))
        .unwrap_or_default();
    let rules =
        repositories::users::access_control::list_rules_for_entity(pool, ENTITY_MCP_SERVER, mcp_id)
            .await
            .inspect_err(|e| tracing::warn!(error = %e, "mcp: grant read failed"))
            .unwrap_or_default();
    let default_included = AccessControlRepository::from_pool(Arc::clone(pool))
        .get_entity(EntityKind::McpServer, mcp_id)
        .await
        .unwrap_or_default()
        .is_some_and(|e| e.default_included);

    DetailSections {
        pagination: build_pagination(
            page_index,
            PageWindow::new(
                page_index,
                EXECUTIONS_PAGE_SIZE,
                executions_total,
                shown,
                "calls",
            ),
            &format!("{BASE_URL}/{mcp_id}"),
        ),
        tools: detail::tool_rows(tools),
        executions: detail::execution_rows(executions),
        executions_total,
        sessions: detail::session_rows(sessions),
        grants: detail::grant_rows(rules),
        default_included,
    }
}
