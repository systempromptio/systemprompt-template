//! Timeline and summary reads over recorded activity.

use crate::repositories::scope::SubjectScope;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::activity::{
    ActivityAction, ActivityCategory, ActivityCategorySummary, ActivityTimelineEvent,
};

pub async fn list_user_recent_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityTimelineEvent,
        r#"SELECT id AS "id!", user_id AS "user_id!",
            display_name AS "display_name!",
            category AS "category!: ActivityCategory",
            action AS "action!: ActivityAction",
            entity_type, entity_name, description AS "description!", created_at AS "created_at!"
        FROM (
            SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS display_name,
                a.category, a.action,
                a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE a.user_id = $1
            UNION ALL
            SELECT r.id, r.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, r.user_id),
                'prompt', 'submitted',
                'gateway_route', r.model,
                CASE WHEN r.status = 'completed'
                    THEN 'Gateway request to ' || r.model || ' (' || r.provider || ')'
                    ELSE 'Gateway request to ' || r.model || ' (' || r.provider || '): ' || r.status
                END,
                r.created_at
            FROM ai_requests r
            JOIN users u ON u.id = r.user_id
            WHERE r.user_id = $1
        ) feed
        ORDER BY created_at DESC LIMIT 50"#,
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
        r#"SELECT label AS "category!", COUNT(*)::BIGINT AS "count!"
        FROM (
            SELECT CASE category
                WHEN 'login' THEN 'Logins'
                WHEN 'skill_usage' THEN 'Skill Uses'
                WHEN 'marketplace_edit' THEN 'Edits'
                WHEN 'marketplace_connect' THEN 'Uploads'
                WHEN 'session' THEN 'Sessions'
                ELSE REPLACE(category, '_', ' ')
            END AS label
            FROM user_activity
            WHERE user_id = $1
            UNION ALL
            SELECT 'Gateway Requests' FROM ai_requests WHERE user_id = $1
        ) counted
        GROUP BY label
        ORDER BY COUNT(*) DESC"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_timeline(
    pool: &PgPool,
    scope: &SubjectScope,
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
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::TEXT[] IS NULL OR u.id = ANY($1))
        ORDER BY a.created_at DESC LIMIT 50"#,
        scope.as_sql()
    )
    .fetch_all(pool)
    .await
}
