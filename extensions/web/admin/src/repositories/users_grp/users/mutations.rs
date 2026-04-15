use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use crate::types::{CreateUserRequest, UpdateUserRequest, UserSummary};

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
        &req.roles,
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
    let roles_update: Option<&[String]> = req.roles.as_deref();
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
        roles_update,
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

#[allow(clippy::cognitive_complexity)]
pub async fn delete_user_complete(pool: &PgPool, user_id: &UserId) -> Result<bool, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let uid = user_id.as_str();

    sqlx::query!("DELETE FROM skill_secrets WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_plugins WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_skills WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_agents WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_mcp_servers WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_hooks WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM plugin_usage_events WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM plugin_usage_daily WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM plugin_session_summaries WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM session_analyses WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM session_ratings WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM skill_ratings WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM daily_summaries WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_profile_reports WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_settings WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_encryption_keys WHERE user_id = $1", uid).execute(&mut *tx).await?;
    sqlx::query!("DELETE FROM user_selected_org_plugins WHERE user_id = $1", uid).execute(&mut *tx).await?;

    // marketplace schema tables — runtime query (schema not in compile-time search_path)
    for table in ["marketplace.subscriptions", "marketplace.paddle_customers"] {
        sqlx::query(&format!("DELETE FROM {table} WHERE user_id = $1"))
            .bind(uid)
            .execute(&mut *tx)
            .await?;
    }

    let result = sqlx::query!("DELETE FROM users WHERE id = $1", uid)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;
    Ok(result.rows_affected() > 0)
}
