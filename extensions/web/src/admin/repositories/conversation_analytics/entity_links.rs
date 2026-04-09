
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::admin::types::conversation_analytics::{EntityUsageSummary, SessionEntityLink};

pub async fn fetch_session_entities(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &str,
) -> Result<Vec<SessionEntityLink>, sqlx::Error> {
    sqlx::query_as!(
        SessionEntityLink,
        r"SELECT entity_type, entity_name, usage_count
        FROM session_entity_links
        WHERE user_id = $1 AND session_id = $2
        ORDER BY usage_count DESC",
        user_id.as_str(),
        session_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_session_entity_links(
    pool: &PgPool,
    session_id: &str,
) -> Result<Vec<SessionEntityLink>, sqlx::Error> {
    sqlx::query_as!(
        SessionEntityLink,
        r"SELECT entity_type, entity_name, usage_count
          FROM session_entity_links
          WHERE session_id = $1
          ORDER BY usage_count DESC",
        session_id,
    )
    .fetch_all(pool)
    .await
}

pub async fn fetch_all_session_entity_links(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<(String, SessionEntityLink)>, sqlx::Error> {
    #[derive(sqlx::FromRow)]
    struct Row {
        session_id: String,
        entity_type: String,
        entity_name: String,
        usage_count: i32,
    }

    let rows = sqlx::query_as!(
        Row,
        r"SELECT session_id, entity_type, entity_name, usage_count
        FROM session_entity_links
        WHERE user_id = $1
        ORDER BY session_id, usage_count DESC
        LIMIT 200",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| {
            (
                r.session_id,
                SessionEntityLink {
                    entity_type: r.entity_type,
                    entity_name: r.entity_name,
                    usage_count: r.usage_count,
                },
            )
        })
        .collect())
}

pub async fn fetch_entity_usage_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<EntityUsageSummary>, sqlx::Error> {
    sqlx::query_as!(
        EntityUsageSummary,
        r#"SELECT
            entity_type,
            entity_name,
            COALESCE(entity_id, entity_name) AS "entity_id!",
            COALESCE(SUM(usage_count), 0)::BIGINT AS "total_uses!",
            COUNT(DISTINCT session_id)::BIGINT AS "session_count!"
        FROM session_entity_links
        WHERE user_id = $1
        GROUP BY entity_type, entity_name, entity_id
        ORDER BY 4 DESC
        LIMIT 200"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn upsert_session_entity_link(
    pool: &PgPool,
    user_id: &UserId,
    session_id: &str,
    entity_type: &str,
    entity_name: &str,
    entity_id: Option<&str>,
) -> Result<(), sqlx::Error> {
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

pub async fn fetch_unused_skills(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!(
        r"SELECT us.skill_id
          FROM user_skills us
          WHERE us.user_id = $1 AND us.enabled = true
            AND us.skill_id NOT IN (
                SELECT DISTINCT COALESCE(entity_id, entity_name) FROM session_entity_links
                WHERE user_id = $1 AND entity_type = 'skill'
            )
          ORDER BY us.skill_id ASC",
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
