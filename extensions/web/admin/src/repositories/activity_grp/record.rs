use sqlx::PgPool;

use crate::activity::NewActivity;

pub async fn record(pool: &PgPool, activity: NewActivity) {
    let category = activity.category.as_ref();
    let action = activity.action.as_ref();

    if should_deduplicate(pool, category, action, &activity).await {
        return;
    }

    let (entity_type, entity_id, entity_name) =
        activity.entity.as_ref().map_or((None, None, None), |ent| {
            (
                Some(ent.kind.as_ref().to_owned()),
                ent.id.clone(),
                ent.name.clone(),
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
        ("login", _) => has_recent_login(pool, &activity.user_id).await,
        ("tool_usage", _) => match activity.entity.as_ref().and_then(|e| e.name.as_ref()) {
            Some(name) => has_recent_tool_usage(pool, &activity.user_id, name).await,
            None => false,
        },
        ("mcp_access", "rejected") => {
            match activity.entity.as_ref().and_then(|e| e.name.as_ref()) {
                Some(name) => has_recent_mcp_rejected(pool, name).await,
                None => false,
            }
        },
        ("session", "started") => match activity.entity.as_ref().and_then(|e| e.id.as_ref()) {
            Some(eid) => has_session_started(pool, eid).await,
            None => false,
        },
        _ => false,
    }
}

async fn has_recent_login(pool: &PgPool, user_id: &str) -> bool {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'login' AND created_at > NOW() - INTERVAL '1 hour'",
        user_id
    )
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "activity_grp: dedupe count query failed"))
    .ok()
    .flatten()
    .unwrap_or(0)
        > 0
}

async fn has_recent_tool_usage(pool: &PgPool, user_id: &str, tool_name: &str) -> bool {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'tool_usage' AND entity_name = $2 AND created_at > NOW() - INTERVAL '30 seconds'",
        user_id,
        tool_name
    )
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "activity_grp: dedupe count query failed"))
    .ok()
    .flatten()
    .unwrap_or(0)
        > 0
}

async fn has_recent_mcp_rejected(pool: &PgPool, server_name: &str) -> bool {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_activity WHERE category = 'mcp_access' AND action = 'rejected' AND entity_name = $1 AND created_at > NOW() - INTERVAL '60 seconds'",
        server_name
    )
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "activity_grp: dedupe count query failed"))
    .ok()
    .flatten()
    .unwrap_or(0)
        > 0
}

async fn has_session_started(pool: &PgPool, entity_id: &str) -> bool {
    sqlx::query_scalar!(
        "SELECT COUNT(*) FROM user_activity WHERE entity_id = $1 AND category = 'session' AND action = 'started'",
        entity_id
    )
    .fetch_one(pool)
    .await
    .inspect_err(|e| tracing::warn!(error = %e, "activity_grp: dedupe count query failed"))
    .ok()
    .flatten()
    .unwrap_or(0)
        > 0
}
