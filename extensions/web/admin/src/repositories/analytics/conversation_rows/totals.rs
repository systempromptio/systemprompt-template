//! The KPI strip above a conversation list, computed over the same filter the
//! list pages through; optionally over the preceding window of equal length
//! so each figure can carry a delta.

use sqlx::PgPool;

use super::ConversationFilter;

#[derive(Debug, Clone, Copy, Default, serde::Serialize, serde::Deserialize)]
pub struct ConversationTotals {
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub side_calls: i64,
    pub tool_calls: i64,
    pub error_conversations: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
    pub side_call_cost_microdollars: i64,
}

pub async fn get_conversation_totals(
    pool: &PgPool,
    filter: &ConversationFilter,
) -> Result<ConversationTotals, sqlx::Error> {
    Ok(super::load_conversation_page(
        pool,
        filter,
        super::ConversationPage::default(),
        super::ConversationPageMode::Totals,
    )
    .await?
    .totals)
}

pub async fn list_conversation_models(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT DISTINCT model AS "model!"
        FROM ai_requests
        WHERE model IS NOT NULL
          AND context_id <> '00000000-0000-0000-0000-4c4547414359'
        ORDER BY model
        "#
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|r| r.model).collect())
}
