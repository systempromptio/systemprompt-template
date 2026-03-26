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
