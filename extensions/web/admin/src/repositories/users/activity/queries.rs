//! Timeline and summary reads over recorded activity.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::activity::{
    ActivityAction, ActivityCategory, ActivityCategorySummary, ActivityTimelineEvent,
};

pub async fn list_timeline(
    pool: &PgPool,
    department: Option<&str>,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    if let Some(dept) = department {
        sqlx::query_as!(
            ActivityTimelineEvent,
            r#"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                a.category AS "category: ActivityCategory",
                a.action AS "action: ActivityAction",
                a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            JOIN user_profile_ext upe ON upe.user_id = u.id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
              AND upe.department = $1
            ORDER BY a.created_at DESC LIMIT 50"#,
            dept
        )
        .fetch_all(pool)
        .await
    } else {
        sqlx::query_as!(
            ActivityTimelineEvent,
            r#"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                a.category AS "category: ActivityCategory",
                a.action AS "action: ActivityAction",
                a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            ORDER BY a.created_at DESC LIMIT 50"#
        )
        .fetch_all(pool)
        .await
    }
}

pub async fn list_user_recent_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityTimelineEvent,
        r#"SELECT a.id, a.user_id,
            COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
            a.category AS "category: ActivityCategory",
            a.action AS "action: ActivityAction",
            a.entity_type, a.entity_name, a.description, a.created_at
        FROM user_activity a
        JOIN users u ON u.id = a.user_id
        WHERE a.user_id = $1
        ORDER BY a.created_at DESC LIMIT 50"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_activity_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityCategorySummary>, sqlx::Error> {
    sqlx::query_as!(
        ActivityCategorySummary,
        r#"SELECT CASE category
            WHEN 'login' THEN 'Logins'
            WHEN 'skill_usage' THEN 'Skill Uses'
            WHEN 'marketplace_edit' THEN 'Edits'
            WHEN 'marketplace_connect' THEN 'Uploads'
            WHEN 'session' THEN 'Sessions'
            ELSE REPLACE(category, '_', ' ')
        END AS "category!",
        COUNT(*)::BIGINT AS "count!"
        FROM user_activity
        WHERE user_id = $1
        GROUP BY category
        ORDER BY COUNT(*) DESC"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
