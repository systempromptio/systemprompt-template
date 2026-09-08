//! Compatibility entry point; rows and their count come from one snapshot.
use super::{
    ConversationFilter, ConversationPage, ConversationPageMode, ConversationRow,
    load_conversation_page,
};
use sqlx::PgPool;

pub async fn list_conversations_paged(
    pool: &PgPool,
    filter: &ConversationFilter,
    page: ConversationPage,
) -> Result<(Vec<ConversationRow>, i64), sqlx::Error> {
    let result = load_conversation_page(pool, filter, page, ConversationPageMode::All).await?;
    Ok((result.conversations, result.totals.conversations))
}
