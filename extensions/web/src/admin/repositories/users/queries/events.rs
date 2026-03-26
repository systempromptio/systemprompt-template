use std::fmt::Write;
use std::sync::Arc;

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::super::super::types::{
    EventRow, EventTypeCount, EventsQuery, EventsResponse, UsageEvent,
};

pub async fn get_user_usage(
    pool: &Arc<PgPool>,
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
        user_id as &UserId,
    )
    .fetch_all(pool.as_ref())
    .await
}

pub async fn list_user_events(
    pool: &Arc<PgPool>,
    user_id: &UserId,
    query: &EventsQuery,
) -> Result<EventsResponse, sqlx::Error> {
    let mut where_clause = String::from(" WHERE p.user_id = $1");
    let mut bind_idx = 1u32;
    let mut binds: Vec<String> = vec![user_id.as_str().to_string()];

    if let Some(ref search) = query.search {
        bind_idx += 1;
        let pattern = format!("%{search}%");
        let _ = write!(
            where_clause,
            " AND (p.id ILIKE ${bind_idx} OR p.tool_name ILIKE ${bind_idx} OR p.event_type ILIKE ${bind_idx} OR p.session_id ILIKE ${bind_idx})",
        );
        binds.push(pattern);
    }
    if let Some(ref et) = query.event_type {
        bind_idx += 1;
        let _ = write!(where_clause, " AND p.event_type = ${bind_idx}");
        binds.push(et.clone());
    }

    let count_sql = format!(
        "SELECT COALESCE(COUNT(*), 0)::BIGINT FROM plugin_usage_events p JOIN users u ON u.id = p.user_id{where_clause}"
    );
    let mut count_q = sqlx::query_scalar::<_, i64>(&count_sql);
    for b in &binds {
        count_q = count_q.bind(b);
    }
    let total = count_q.fetch_one(pool.as_ref()).await?;

    let limit_idx = bind_idx + 1;
    let offset_idx = bind_idx + 2;

    let data_sql = format!(
        r"SELECT
            p.id, p.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, p.user_id) AS display_name,
            u.email, p.session_id, p.event_type, p.tool_name, p.metadata, p.created_at
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        {where_clause}
        ORDER BY p.created_at DESC
        LIMIT ${limit_idx} OFFSET ${offset_idx}"
    );

    let mut data_q = sqlx::query_as::<_, EventRow>(&data_sql);
    for b in &binds {
        data_q = data_q.bind(b);
    }
    data_q = data_q.bind(query.limit).bind(query.offset);
    let events = data_q.fetch_all(pool.as_ref()).await?;

    Ok(EventsResponse {
        events,
        total,
        limit: query.limit,
        offset: query.offset,
    })
}

pub async fn get_user_event_type_counts(
    pool: &Arc<PgPool>,
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
    .fetch_all(pool.as_ref())
    .await
}
