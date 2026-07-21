//! Session-to-entity link reads and upserts.

use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

use crate::types::conversation_analytics::SessionEntityLink;

pub async fn list_session_entity_links(
    pool: &PgPool,
    session_id: &SessionId,
) -> Result<Vec<SessionEntityLink>, sqlx::Error> {
    sqlx::query_as!(
        SessionEntityLink,
        r"SELECT entity_type, entity_name, usage_count
          FROM session_entity_links
          WHERE session_id = $1
          ORDER BY usage_count DESC",
        session_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug)]
pub struct EntityLinkInput<'a> {
    pub session_id: &'a str,
    pub entity_type: &'a str,
    pub entity_name: &'a str,
    pub entity_id: Option<&'a str>,
}

pub async fn upsert_session_entity_link(
    pool: &PgPool,
    user_id: &UserId,
    input: EntityLinkInput<'_>,
) -> Result<(), sqlx::Error> {
    let EntityLinkInput {
        session_id,
        entity_type,
        entity_name,
        entity_id,
    } = input;
    sqlx::query!(
        r"INSERT INTO session_entity_links (id, user_id, session_id, entity_type, entity_name, entity_id, usage_count, first_seen_at, last_seen_at)
        VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, 1, NOW(), NOW())
        ON CONFLICT (user_id, session_id, entity_type, entity_name)
        DO UPDATE SET usage_count = session_entity_links.usage_count + 1, last_seen_at = NOW(),
            entity_id = COALESCE(EXCLUDED.entity_id, session_entity_links.entity_id)",
        user_id.as_str(),
        session_id,
        entity_type,
        entity_name,
        entity_id,
    )
    .execute(pool)
    .await?;
    Ok(())
}
