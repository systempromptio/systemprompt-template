//! Timeline and summary reads over recorded activity.

use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::activity::{
    ActivityAction, ActivityCategory, ActivityCategorySummary, ActivityTimelineEvent,
};
use crate::repositories::scope::SubjectScope;

pub async fn list_timeline(
    pool: &PgPool,
    scope: &SubjectScope,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityTimelineEvent,
        r#"SELECT a.id, a.user_id AS "user_id!: UserId",
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

// Why: the gateway member's requests are part of a person's feed in the builds
// that link it; they are read beside the console feed and merged by time.
pub async fn list_user_recent_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    let feed = list_console_activity(pool, user_id).await?;
    #[cfg(feature = "gateway")]
    let feed = {
        let mut feed = feed;
        feed.extend(list_gateway_activity(pool, user_id).await?);
        feed.sort_by_key(|e| std::cmp::Reverse(e.created_at));
        feed.truncate(FEED_LIMIT);
        feed
    };
    Ok(feed)
}

#[cfg(feature = "gateway")]
const FEED_LIMIT: usize = 50;

async fn list_console_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityTimelineEvent,
        r#"SELECT id AS "id!", user_id AS "user_id!: UserId",
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
        ) feed
        ORDER BY created_at DESC LIMIT 50"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

#[cfg(feature = "gateway")]
async fn list_gateway_activity(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityTimelineEvent>, sqlx::Error> {
    sqlx::query_as!(
        ActivityTimelineEvent,
        r#"SELECT r.id AS "id!", r.user_id AS "user_id!: UserId",
            COALESCE(u.display_name, u.full_name, u.name, u.email, r.user_id) AS "display_name!",
            'prompt' AS "category!: ActivityCategory",
            'submitted' AS "action!: ActivityAction",
            'gateway_route' AS entity_type, r.model AS entity_name,
            CASE WHEN r.status = 'completed'
                THEN 'Gateway request to ' || r.model || ' (' || r.provider || ')'
                ELSE 'Gateway request to ' || r.model || ' (' || r.provider || '): ' || r.status
            END AS "description!",
            r.created_at AS "created_at!"
        FROM ai_requests r
        JOIN users u ON u.id = r.user_id
        WHERE r.user_id = $1
        ORDER BY r.created_at DESC LIMIT 50"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}

pub async fn list_user_activity_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ActivityCategorySummary>, sqlx::Error> {
    let counts = list_console_activity_counts(pool, user_id).await?;
    #[cfg(feature = "gateway")]
    let counts = {
        let mut counts = counts;
        let requests = sqlx::query_scalar!(
            r#"SELECT COUNT(*)::BIGINT AS "count!" FROM ai_requests WHERE user_id = $1"#,
            user_id.as_str()
        )
        .fetch_one(pool)
        .await?;
        if requests > 0 {
            counts.push(ActivityCategorySummary {
                category: "Gateway Requests".to_owned(),
                count: requests,
            });
            counts.sort_by_key(|c| std::cmp::Reverse(c.count));
        }
        counts
    };
    Ok(counts)
}

async fn list_console_activity_counts(
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
        ) counted
        GROUP BY label
        ORDER BY COUNT(*) DESC"#,
        user_id.as_str()
    )
    .fetch_all(pool)
    .await
}
