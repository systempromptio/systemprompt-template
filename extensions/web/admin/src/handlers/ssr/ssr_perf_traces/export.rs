//! The trace list as the export surface reads it.

use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::scope::ScopeRequest;
use crate::repositories::traces::{TracePage, TraceSummary, list_traces};
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use super::{TraceListQuery, build_filter, sort_from_query};

// Why: the export reads the rows the explorer shows — same scope, filter and
// sort — through the page's own query type.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: TraceListQuery,
    range: TimeRange,
    limit: i64,
) -> AdminResult<(Vec<TraceSummary>, i64)> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let filter = build_filter(&query, subjects.as_sql());
    let page = TracePage {
        sort: sort_from_query(&query),
        limit,
        offset: 0,
    };
    Ok(list_traces(pool, filter, range, page).await?)
}
