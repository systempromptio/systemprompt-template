//! A single statement shares filtered metrics across the page and its KPIs.

use serde::Deserialize;
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContextId, UserId};

use super::{
    ConversationFilter, ConversationPage, ConversationRow, ConversationTotals,
    UserConversationSummary,
};

#[derive(Debug, Default)]
pub struct ConversationPageResult {
    pub conversations: Vec<ConversationRow>,
    pub user_summaries: Vec<UserConversationSummary>,
    pub totals: ConversationTotals,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversationPageMode {
    All,
    Users,
    Totals,
}

pub async fn load_conversation_page(
    pool: &PgPool,
    filter: &ConversationFilter,
    page: ConversationPage,
    mode: ConversationPageMode,
) -> Result<ConversationPageResult, sqlx::Error> {
    let started = std::time::Instant::now();
    let mut connection = crate::repositories::dashboard_read::begin(pool).await?;
    let pool_wait_ms = started.elapsed().as_secs_f64() * 1000.0;
    let query_started = std::time::Instant::now();
    let pattern = filter.free_text_pattern();
    let result = sqlx::query_file!(
        "src/repositories/analytics/conversation_rows/page.sql",
        filter.user_id.as_ref().map(UserId::as_str),
        filter.subject_ids.as_deref(),
        filter.model,
        pattern,
        filter.since,
        filter.until,
        filter.include_side_calls,
        filter.error_only,
        page.sort.as_str(),
        page.descending,
        page.limit,
        page.offset,
        mode == ConversationPageMode::Users,
        mode == ConversationPageMode::Totals,
    )
    .fetch_one(&mut *connection)
    .await?;
    tracing::debug!(
        ?mode,
        pool_wait_ms,
        query_ms = query_started.elapsed().as_secs_f64() * 1000.0,
        "conversation page loaded"
    );
    connection.commit().await?;
    let ids = result.context_ids;
    let wire = result.payload.0;
    Ok(ConversationPageResult {
        conversations: wire
            .conversations
            .into_iter()
            .map(|r| decode_row(r, &ids))
            .collect::<Result<_, _>>()?,
        user_summaries: wire
            .user_summaries
            .into_iter()
            .map(|s| {
                Ok(UserConversationSummary {
                    user_id: s.user_id,
                    display_name: s.display_name,
                    conversation_count: s.conversation_count,
                    turn_count: s.turn_count,
                    total_tokens: s.total_tokens,
                    side_call_count: s.side_call_count,
                    total_cost_microdollars: s.total_cost_microdollars,
                    last_at: s.last_at,
                    models: s.models,
                    latest: s.latest.map(|r| decode_row(r, &ids)).transpose()?,
                })
            })
            .collect::<Result<_, sqlx::Error>>()?,
        totals: wire.totals,
    })
}

// Why: IDs are decoded by SQLx from a companion array. JSON input validation is
// stricter than the SQLx decoder and must not reject IDs already in the store.
#[derive(Debug, Deserialize)]
struct ConversationPageWire {
    conversations: Vec<ConversationRow<String>>,
    user_summaries: Vec<UserConversationSummary<String>>,
    totals: ConversationTotals,
}

fn decode_row(
    r: ConversationRow<String>,
    ids: &[ContextId],
) -> Result<ConversationRow, sqlx::Error> {
    Ok(ConversationRow {
        context_id: crate::repositories::dashboard_read::context_id(&r.context_id, ids)?,
        title: r.title,
        user_id: r.user_id,
        display_name: r.display_name,
        session_id: r.session_id,
        client_session_id: r.client_session_id,
        group_name: r.group_name,
        project_name: r.project_name,
        model: r.model,
        turn_count: r.turn_count,
        side_call_count: r.side_call_count,
        side_call_cost_microdollars: r.side_call_cost_microdollars,
        tool_call_count: r.tool_call_count,
        error_count: r.error_count,
        total_input_tokens: r.total_input_tokens,
        total_output_tokens: r.total_output_tokens,
        total_cost_microdollars: r.total_cost_microdollars,
        first_at: r.first_at,
        last_at: r.last_at,
        status: r.status,
    })
}
