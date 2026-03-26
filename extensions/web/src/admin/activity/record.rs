use sqlx::PgPool;

use super::types::NewActivity;

pub async fn record(pool: &PgPool, activity: NewActivity) {
    let category = activity.category.as_ref();
    let action = activity.action.as_ref();

    if category == "login" {
        let recent = sqlx::query_scalar::<_, i64>(
            "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'login' AND created_at > NOW() - INTERVAL '1 hour'"
        )
        .bind(&activity.user_id)
        .fetch_one(pool)
        .await
        .unwrap_or(0);

        if recent > 0 {
            return;
        }
    }

    if category == "tool_usage" {
        if let Some(ref ent) = activity.entity {
            if let Some(ref tool_name) = ent.entity_name {
                let recent = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM user_activity WHERE user_id = $1 AND category = 'tool_usage' AND entity_name = $2 AND created_at > NOW() - INTERVAL '30 seconds'"
                )
                .bind(&activity.user_id)
                .bind(tool_name)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                if recent > 0 {
                    return;
                }
            }
        }
    }

    if category == "mcp_access" && action == "rejected" {
        if let Some(ref ent) = activity.entity {
            if let Some(ref server_name) = ent.entity_name {
                let recent = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM user_activity WHERE category = 'mcp_access' AND action = 'rejected' AND entity_name = $1 AND created_at > NOW() - INTERVAL '60 seconds'"
                )
                .bind(server_name)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                if recent > 0 {
                    return;
                }
            }
        }
    }

    if category == "session" && action == "started" {
        if let Some(ref ent) = activity.entity {
            if let Some(ref eid) = ent.entity_id {
                let existing = sqlx::query_scalar::<_, i64>(
                    "SELECT COUNT(*) FROM user_activity WHERE entity_id = $1 AND category = 'session' AND action = 'started'",
                )
                .bind(eid)
                .fetch_one(pool)
                .await
                .unwrap_or(0);

                if existing > 0 {
                    return;
                }
            }
        }
    }

    let (entity_type, entity_id, entity_name) = match &activity.entity {
        Some(ent) => (
            Some(ent.entity_type.as_ref().to_string()),
            ent.entity_id.clone(),
            ent.entity_name.clone(),
        ),
        None => (None, None, None),
    };

    if let Err(e) = sqlx::query(
        r"INSERT INTO user_activity (id, user_id, category, action, entity_type, entity_id, entity_name, description, metadata)
          VALUES (gen_random_uuid()::TEXT, $1, $2, $3, $4, $5, $6, $7, $8)",
    )
    .bind(&activity.user_id)
    .bind(category)
    .bind(action)
    .bind(&entity_type)
    .bind(&entity_id)
    .bind(&entity_name)
    .bind(&activity.description)
    .bind(&activity.metadata)
    .execute(pool)
    .await
    {
        tracing::warn!(error = %e, "Failed to record user activity (non-fatal)");
    }
}
