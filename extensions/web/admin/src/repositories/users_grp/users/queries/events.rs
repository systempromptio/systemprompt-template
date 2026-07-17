use sqlx::PgPool;
use systemprompt::identifiers::{Email, PluginId, SessionId, UserId};

use crate::types::{EventRow, EventTypeCount, EventsQuery, EventsResponse, UsageEvent};

pub async fn get_user_usage(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<UsageEvent>, sqlx::Error> {
    sqlx::query_as!(
        UsageEvent,
        r#"
        SELECT id, event_type, tool_name, created_at, metadata
        FROM plugin_usage_events
        WHERE user_id = $1
        ORDER BY created_at DESC
        LIMIT 100
        "#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_events(
    pool: &PgPool,
    user_id: &UserId,
    query: &EventsQuery,
) -> Result<EventsResponse, sqlx::Error> {
    let search_pattern = query
        .search
        .as_deref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{s}%"));
    let event_type = query.event_type.as_deref();

    let total = sqlx::query_scalar!(
        r#"SELECT COALESCE(COUNT(*), 0)::BIGINT AS "count!"
           FROM plugin_usage_events p
           JOIN users u ON u.id = p.user_id
           WHERE p.user_id = $1
             AND ($2::text IS NULL
                  OR p.id ILIKE $2
                  OR p.tool_name ILIKE $2
                  OR p.event_type ILIKE $2
                  OR p.session_id ILIKE $2)
             AND ($3::text IS NULL OR p.event_type = $3)"#,
        user_id.as_str(),
        search_pattern.as_deref(),
        event_type,
    )
    .fetch_one(pool)
    .await?;

    let events = sqlx::query_as!(
        EventRow,
        r#"SELECT
            p.id AS "id!",
            p.user_id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS "display_name!",
            u.email AS "email?: Email",
            p.session_id AS "session_id!: SessionId",
            p.event_type AS "event_type!",
            p.tool_name,
            p.plugin_id AS "plugin_id?: PluginId",
            p.metadata AS "metadata!",
            p.created_at AS "created_at!"
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE p.user_id = $1
          AND ($2::text IS NULL
               OR p.id ILIKE $2
               OR p.tool_name ILIKE $2
               OR p.event_type ILIKE $2
               OR p.session_id ILIKE $2)
          AND ($3::text IS NULL OR p.event_type = $3)
        ORDER BY p.created_at DESC
        LIMIT $4 OFFSET $5"#,
        user_id.as_str(),
        search_pattern.as_deref(),
        event_type,
        query.limit,
        query.offset,
    )
    .fetch_all(pool)
    .await?;

    Ok(EventsResponse {
        events,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}

pub async fn get_user_event_type_counts(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<EventTypeCount>, sqlx::Error> {
    sqlx::query_as!(
        EventTypeCount,
        r#"SELECT
            event_type as "event_type!",
            COUNT(*)::BIGINT as "count!"
        FROM plugin_usage_events
        WHERE user_id = $1
        GROUP BY event_type
        ORDER BY 2 DESC"#,
        user_id.as_str(),
    )
    .fetch_all(pool)
    .await
}
