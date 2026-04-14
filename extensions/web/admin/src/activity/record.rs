use sqlx::PgPool;

use super::types::NewActivity;

pub async fn record(pool: &PgPool, activity: NewActivity) {
    let category = activity.category.as_ref();
    let action = activity.action.as_ref();

    if should_deduplicate(pool, category, action, &activity).await {
        return;
    }

    let (entity_type, entity_id, entity_name) =
        activity.entity.as_ref().map_or((None, None, None), |ent| {
            (
                Some(ent.entity_type.as_ref().to_string()),
                ent.entity_id.clone(),
                ent.entity_name.clone(),
            )
        });

    if let Err(e) = sqlx::query!(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_id, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6, $7, $8)",
        activity.user_id,
        category,
        action,
        entity_type,
        entity_id,
        entity_name,
        activity.description,
        activity.metadata,
    )
    .execute(pool)
    .await
    {
        tracing::warn!(error = %e, "Failed to record user activity (non-fatal)");
    }
}

async fn should_deduplicate(
    pool: &PgPool,
    category: &str,
    action: &str,
    activity: &NewActivity,
) -> bool {
    match (category, action) {
        ("login", _) => {
            has_recent_activity(
                pool,
                "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'login' AND created_at > NOW() - INTERVAL '1 hour'",
                &[&activity.user_id],
            )
            .await
        }
        ("tool_usage", _) => {
            let tool_name = activity
                .entity
                .as_ref()
                .and_then(|e| e.entity_name.as_ref());
            match tool_name {
                Some(name) => {
                    has_recent_activity(
                        pool,
                        "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'tool_usage' AND entity_name = $2 AND created_at > NOW() - INTERVAL '30 seconds'",
                        &[&activity.user_id, name],
                    )
                    .await
                }
                None => false,
            }
        }
        ("mcp_access", "rejected") => {
            let server_name = activity
                .entity
                .as_ref()
                .and_then(|e| e.entity_name.as_ref());
            match server_name {
                Some(name) => {
                    has_recent_activity(
                        pool,
                        "SELECT COUNT(*) FROM user_activity WHERE category = 'mcp_access' AND action = 'rejected' AND entity_name = $1 AND created_at > NOW() - INTERVAL '60 seconds'",
                        &[name],
                    )
                    .await
                }
                None => false,
            }
        }
        ("session", "started") => {
            let entity_id = activity
                .entity
                .as_ref()
                .and_then(|e| e.entity_id.as_ref());
            match entity_id {
                Some(eid) => {
                    has_recent_activity(
                        pool,
                        "SELECT COUNT(*) FROM user_activity WHERE entity_id = $1 AND category = 'session' AND action = 'started'",
                        &[eid],
                    )
                    .await
                }
                None => false,
            }
        }
        _ => false,
    }
}

async fn has_recent_activity(pool: &PgPool, query: &str, binds: &[&str]) -> bool {
    let mut q = sqlx::query_scalar::<_, i64>(query);
    for bind in binds {
        q = q.bind(bind);
    }
    let count = q.fetch_one(pool).await.unwrap_or(0);
    count > 0
}
