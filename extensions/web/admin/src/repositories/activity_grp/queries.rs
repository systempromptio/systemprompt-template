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

pub async fn list_new_events(
    pool: &PgPool,
    since_id: Option<&str>,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    if let Some(sid) = since_id {
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
              AND a.created_at > (SELECT created_at FROM user_activity WHERE id = $1)
            ORDER BY a.created_at DESC LIMIT 20"#,
            sid
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
            ORDER BY a.created_at DESC LIMIT 5"#
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

pub async fn search_user_entity_activity(
    pool: &PgPool,
    user_id: &UserId,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<ActivityTimelineEvent>, i64), sqlx::Error> {
    if let Some(q) = search {
        let pattern = format!("%{q}%");
        let total: i64 = sqlx::query_scalar!(
            r"SELECT COUNT(*)::BIGINT FROM user_activity a
              JOIN users u ON u.id = a.user_id
              WHERE a.user_id = $1
                AND (a.description ILIKE $2 OR a.entity_name ILIKE $2 OR a.category ILIKE $2)",
            user_id.as_str(),
            pattern,
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            ActivityTimelineEvent,
            r#"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                a.category AS "category: ActivityCategory",
                a.action AS "action: ActivityAction",
                a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE a.user_id = $1
              AND (a.description ILIKE $2 OR a.entity_name ILIKE $2 OR a.category ILIKE $2)
            ORDER BY a.created_at DESC LIMIT $3 OFFSET $4"#,
            user_id.as_str(),
            pattern,
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok((rows, total))
    } else {
        let total: i64 = sqlx::query_scalar!(
            "SELECT COUNT(*)::BIGINT FROM user_activity WHERE user_id = $1",
            user_id.as_str()
        )
        .fetch_one(pool)
        .await?
        .unwrap_or(0);

        let rows = sqlx::query_as!(
            ActivityTimelineEvent,
            r#"SELECT a.id, a.user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, a.user_id) AS "display_name!",
                a.category AS "category: ActivityCategory",
                a.action AS "action: ActivityAction",
                a.entity_type, a.entity_name, a.description, a.created_at
            FROM user_activity a
            JOIN users u ON u.id = a.user_id
            WHERE a.user_id = $1
            ORDER BY a.created_at DESC LIMIT $2 OFFSET $3"#,
            user_id.as_str(),
            limit,
            offset
        )
        .fetch_all(pool)
        .await?;

        Ok((rows, total))
    }
}

pub async fn list_user_entity_activity(
    pool: &PgPool,
    user_id: &UserId,
    limit: i64,
    offset: i64,
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
        ORDER BY a.created_at DESC LIMIT $2 OFFSET $3"#,
        user_id.as_str(),
        limit,
        offset
    )
    .fetch_all(pool)
    .await
}

pub async fn count_user_entity_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<i64, sqlx::Error> {
    Ok(sqlx::query_scalar!(
        "SELECT COUNT(*)::BIGINT FROM user_activity WHERE user_id = $1",
        user_id.as_str()
    )
    .fetch_one(pool)
    .await?
    .unwrap_or(0))
}
