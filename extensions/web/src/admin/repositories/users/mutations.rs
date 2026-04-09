
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::super::super::types::{CreateUserRequest, UpdateUserRequest, UserSummary};

pub async fn create_user(
    pool: &PgPool,
    req: &CreateUserRequest,
) -> Result<UserSummary, sqlx::Error> {
    let user_id_str = req.user_id.as_str().to_string();
    let status = req.status.clone().unwrap_or_else(|| "active".to_string());
    let username = req.email.as_str();
    sqlx::query_as!(
        UserSummary,
        r#"
        INSERT INTO users (id, name, email, display_name, roles, status)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (email) DO UPDATE SET
            display_name = COALESCE(EXCLUDED.display_name, users.display_name),
            roles = EXCLUDED.roles,
            status = EXCLUDED.status,
            updated_at = NOW()
        RETURNING
            id AS "user_id!",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            created_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        &user_id_str,
        username,
        req.email.as_str(),
        &req.display_name,
        &req.roles as &[String],
        &status,
    )
    .fetch_one(pool)
    .await
}

pub async fn update_user(
    pool: &PgPool,
    user_id: &UserId,
    req: &UpdateUserRequest,
) -> Result<Option<UserSummary>, sqlx::Error> {
    let status = req.is_active.map(|active| {
        if active {
            "active".to_string()
        } else {
            "inactive".to_string()
        }
    });
    let set_email_verified = req.is_active == Some(true);
    sqlx::query_as!(
        UserSummary,
        r#"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            email = COALESCE($3, email),
            roles = COALESCE($4, roles),
            status = COALESCE($5, status),
            email_verified = CASE WHEN $6 THEN true ELSE email_verified END,
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id AS "user_id!",
            COALESCE(display_name, name) AS display_name,
            email AS "email: _",
            roles AS "roles!: Vec<String>",
            (status = 'active') AS "is_active!",
            updated_at AS "last_active!",
            0::BIGINT AS "total_events!",
            NULL::TEXT AS last_tool,
            0::BIGINT AS "custom_skills_count!",
            NULL::TEXT AS preferred_client,
            0::BIGINT AS "prompts!",
            0::BIGINT AS "sessions!",
            0::BIGINT AS "bytes!",
            0::BIGINT AS "logins!"
        "#,
        user_id.as_str(),
        req.display_name.as_deref(),
        req.email.as_deref(),
        &req.roles as &Option<Vec<String>>,
        status.as_deref(),
        set_email_verified,
    )
    .fetch_optional(pool)
    .await
}

pub async fn delete_user(pool: &PgPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", user_id.as_str())
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_user_complete(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let uid = user_id.as_str();

    let tables_with_user_id = [
        "skill_secrets",
        "user_plugins",
        "user_skills",
        "user_agents",
        "user_mcp_servers",
        "user_hooks",
        "plugin_usage_events",
        "plugin_usage_daily",
        "plugin_session_summaries",
        "session_analyses",
        "session_ratings",
        "skill_ratings",
        "daily_summaries",
        "user_profile_reports",
        "user_settings",
        "user_encryption_keys",
        "user_selected_org_plugins",
    ];

    for table in tables_with_user_id {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    for table in ["marketplace.subscriptions", "marketplace.paddle_customers"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}
