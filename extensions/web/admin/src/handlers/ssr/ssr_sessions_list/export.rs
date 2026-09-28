//! The sessions ledger as the export surface reads it.

use sqlx::PgPool;

use crate::error::AdminResult;
use crate::repositories::analytics::conversation_rows::{
    ConversationFilter, ConversationPage, ConversationPageMode, ConversationRow, ConversationSort,
    load_conversation_page,
};
use crate::repositories::scope::ScopeRequest;
use crate::types::UserContext;
use crate::util::time_range::TimeRange;

use super::SessionListQuery;

// Why: the export reads the rows the ledger shows — same scope, filter and
// sort — through the page's own query type.
pub(crate) async fn export_rows(
    pool: &PgPool,
    user_ctx: &UserContext,
    query: SessionListQuery,
    range: TimeRange,
    limit: i64,
) -> AdminResult<(Vec<ConversationRow>, i64)> {
    let request =
        ScopeRequest::from_query(user_ctx, query.group.as_deref(), query.project.as_deref());
    let subjects =
        crate::repositories::scope::membership::get_subject_scope(pool, &request).await?;
    let filter = ConversationFilter {
        user_id: query.user_id.clone().filter(|u| !u.as_str().is_empty()),
        subject_ids: subjects.as_sql().map(<[String]>::to_vec),
        model: None,
        free_text: None,
        since: Some(range.from),
        until: Some(range.to),
        include_side_calls: query.side.as_deref() == Some("1"),
        error_only: query.error_only.as_deref() == Some("true"),
    };
    let page = ConversationPage {
        sort: ConversationSort::parse_conversation_sort(query.sort.as_deref()),
        descending: query.dir.as_deref() != Some("asc"),
        limit,
        offset: 0,
    };
    let result = load_conversation_page(pool, &filter, page, ConversationPageMode::All).await?;
    Ok((result.conversations, result.totals.conversations))
}
