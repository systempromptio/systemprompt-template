use std::sync::Arc;

use sqlx::PgPool;

use super::types::{ActivityCategorySummary, ActivityTimelineEvent};

pub async fn fetch_timeline(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND u.department = $1
            ORDER BY a.created_at DESC LIMIT 50",
        )
        .bind(dept)
        .fetch_all(pool.as_ref())
        .await
    } else {
        sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            ORDER BY a.created_at DESC LIMIT 50",
        )
        .fetch_all(pool.as_ref())
        .await
    }
}

pub async fn fetch_new_events(
    pool: &PgPool,
    since_id: Option<&str>,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    if let Some(sid) = since_id {
        sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND a.created_at > (SELECT created_at FROM user_activity WHERE id = $1)
            ORDER BY a.created_at DESC LIMIT 20",
        )
        .bind(sid)
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            ORDER BY a.created_at DESC LIMIT 5",
        )
        .fetch_all(pool)
        .await
    }
}

pub async fn get_user_recent_activity(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as::<_, ActivityTimelineEvent>(
        r"SELECT a.id, a.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
            a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
        FROM user_activity a
        JOIN users u ON u.id = a.user_id
        WHERE a.user_id = $1
        ORDER BY a.created_at DESC LIMIT 50",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn get_user_activity_summary(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<Vec<ActivityCategorySummary>, sqlx::Error> {
    sqlx::query_as::<_, ActivityCategorySummary>(
        r"SELECT CASE category
            WHEN 'login' THEN 'Logins'
            WHEN 'skill_usage' THEN 'Skill Uses'
            WHEN 'marketplace_edit' THEN 'Edits'
            WHEN 'marketplace_connect' THEN 'Uploads'
            WHEN 'session' THEN 'Sessions'
            ELSE REPLACE(category, '_', ' ')
        END AS category,
        COUNT(*)::BIGINT AS count
        FROM user_activity
        WHERE user_id = $1
        GROUP BY category
        ORDER BY count DESC",
    )
    .bind(user_id)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn search_user_entity_activity(
    pool: &Arc<PgPool>,
    user_id: &str,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ActivityTimelineEvent>, i64), sqlx::Error> {
    if let Some(q) = search {
        let pattern = format!("%{q}%");
        let total: i64 = sqlx::query_scalar(
            r"SELECT COUNT(*)::BIGINT FROM user_activity a
              JOIN users u ON u.id = a.user_id
              WHERE a.user_id = $1
                AND (a.description ILIKE $2 OR a.entity_name ILIKE $2 OR a.category ILIKE $2)",
        )
        .bind(user_id)
        .bind(&pattern)
        .fetch_one(pool.as_ref())
        .await?;

        let rows = sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE a.user_id = $1
              AND (a.description ILIKE $2 OR a.entity_name ILIKE $2 OR a.category ILIKE $2)
            ORDER BY a.created_at DESC LIMIT $3 OFFSET $4",
        )
        .bind(user_id)
        .bind(&pattern)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await?;

        Ok((rows, total))
    } else {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*)::BIGINT FROM user_activity WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_one(pool.as_ref())
        .await?;

        let rows = sqlx::query_as::<_, ActivityTimelineEvent>(
            r"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE a.user_id = $1
            ORDER BY a.created_at DESC LIMIT $2 OFFSET $3",
        )
        .bind(user_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(pool.as_ref())
        .await?;

        Ok((rows, total))
    }
}

pub async fn get_user_entity_activity(
    pool: &Arc<PgPool>,
    user_id: &str,
    limit: i64,
    offset: i64,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as::<_, ActivityTimelineEvent>(
        r"SELECT a.id, a.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
            a.category, a.action, a.entity_type, a.entity_name, a.description, a.created_at
        FROM user_activity a
        JOIN users u ON u.id = a.user_id
        WHERE a.user_id = $1
        ORDER BY a.created_at DESC LIMIT $2 OFFSET $3",
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool.as_ref())
    .await
}

pub async fn count_user_entity_activity(
    pool: &Arc<PgPool>,
    user_id: &str,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar("SELECT COUNT(*)::BIGINT FROM user_activity WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(pool.as_ref())
        .await
}
