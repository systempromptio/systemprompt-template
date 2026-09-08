//! One user's latest conversations — the profile pane's recent list and the
//! roster's "what were they last doing" link. Side-call-only conversations are
//! never listed here: a probe is not something a person talked about.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::ConversationRow;

pub async fn find_latest_conversation(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Option<ConversationRow>, sqlx::Error> {
    Ok(list_recent_conversations(pool, user_id, 1).await?.pop())
}

pub async fn list_recent_conversations(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
) -> Result<Vec<ConversationRow>, sqlx::Error> {
    let filter = super::ConversationFilter {
        user_id: Some(user_id.clone()),
        ..super::ConversationFilter::default()
    };
    let page = super::ConversationPage {
        limit,
        ..super::ConversationPage::default()
    };
    Ok(super::list_conversations_paged(pool, &filter, page)
        .await?
        .0)
}
