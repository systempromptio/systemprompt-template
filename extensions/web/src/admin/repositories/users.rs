use std::sync::Arc;

use sqlx::PgPool;

use super::super::types::{CreateUserRequest, UpdateUserRequest, UserSummary};

pub async fn list_users(
    pool: &Arc<PgPool>,
    department: Option<&str>,
) -> Result<Vec<UserSummary>, sqlx::Error> {
    let base_query = r"
            SELECT
                u.id AS user_id,
                COALESCE(u.display_name, u.full_name, u.name) AS display_name,
                u.email,
                NULLIF(u.department, '') AS department,
                u.roles,
                (u.status = 'active') AS is_active,
                COALESCE(MAX(p.created_at), u.created_at) AS last_active,
                COALESCE(COUNT(DISTINCT p.id), 0)::BIGINT AS total_events,
                (SELECT tool_name FROM plugin_usage_events p2
                 WHERE p2.user_id = u.id
                 ORDER BY created_at DESC LIMIT 1) AS last_tool,
                COALESCE(s.skill_count, 0)::BIGINT AS custom_skills_count,
                (SELECT plugin_id FROM plugin_usage_events p3
                 WHERE p3.user_id = u.id AND p3.plugin_id IS NOT NULL
                 GROUP BY plugin_id ORDER BY COUNT(*) DESC LIMIT 1) AS preferred_client,
                COALESCE(COUNT(DISTINCT p.id) FILTER (WHERE p.event_type LIKE '%UserPromptSubmit%'), 0)::BIGINT AS prompts,
                COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS sessions,
                COALESCE(tok.total_tokens, 0)::BIGINT AS tokens
            FROM users u
            LEFT JOIN plugin_usage_events p ON p.user_id = u.id
            LEFT JOIN (
                SELECT user_id, COUNT(*)::BIGINT AS skill_count
                FROM user_skills GROUP BY user_id
            ) s ON s.user_id = u.id
            LEFT JOIN (
                SELECT user_id,
                       (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0))::BIGINT AS total_tokens
                FROM plugin_usage_daily GROUP BY user_id
            ) tok ON tok.user_id = u.id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
    ";

    if let Some(dept) = department {
        let q = format!(
            "{base_query} AND u.department = $1
            GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                     u.roles, u.status, u.department, s.skill_count, tok.total_tokens
            ORDER BY last_active DESC"
        );
        sqlx::query_as::<_, UserSummary>(&q)
            .bind(dept)
            .fetch_all(pool.as_ref())
            .await
    } else {
        let q = format!(
            "{base_query}
            GROUP BY u.id, u.created_at, u.name, u.display_name, u.full_name, u.email,
                     u.roles, u.status, u.department, s.skill_count, tok.total_tokens
            ORDER BY last_active DESC"
        );
        sqlx::query_as::<_, UserSummary>(&q)
            .fetch_all(pool.as_ref())
            .await
    }
}

pub async fn create_user(
    pool: &Arc<PgPool>,
    req: &CreateUserRequest,
) -> Result<UserSummary, sqlx::Error> {
    let id = uuid::Uuid::new_v4().to_string();
    let name = req.user_id.clone();
    let status = "active".to_string();
    sqlx::query_as::<_, UserSummary>(
        r"
        INSERT INTO users (id, name, email, display_name, department, roles, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING
            id AS user_id,
            COALESCE(display_name, name) AS display_name,
            email,
            NULLIF(department, '') AS department,
            roles,
            (status = 'active') AS is_active,
            created_at AS last_active,
            0::BIGINT AS total_events,
            NULL::TEXT AS last_tool,
            0::BIGINT AS custom_skills_count,
            NULL::TEXT AS preferred_client,
            0::BIGINT AS prompts,
            0::BIGINT AS sessions,
            0::BIGINT AS tokens
        ",
    )
    .bind(&id)
    .bind(&name)
    .bind(&req.email)
    .bind(&req.display_name)
    .bind(&req.department)
    .bind(&req.roles)
    .bind(&status)
    .fetch_one(pool.as_ref())
    .await
}

pub async fn update_user(
    pool: &Arc<PgPool>,
    user_id: &str,
    req: &UpdateUserRequest,
) -> Result<Option<UserSummary>, sqlx::Error> {
    let status = req.is_active.map(|active| {
        if active {
            "active".to_string()
        } else {
            "inactive".to_string()
        }
    });
    sqlx::query_as::<_, UserSummary>(
        r"
        UPDATE users
        SET
            display_name = COALESCE($2, display_name),
            email = COALESCE($3, email),
            department = COALESCE($4, department),
            roles = COALESCE($5, roles),
            status = COALESCE($6, status),
            updated_at = NOW()
        WHERE id = $1
        RETURNING
            id AS user_id,
            COALESCE(display_name, name) AS display_name,
            email,
            NULLIF(department, '') AS department,
            roles,
            (status = 'active') AS is_active,
            updated_at AS last_active,
            0::BIGINT AS total_events,
            NULL::TEXT AS last_tool,
            0::BIGINT AS custom_skills_count,
            NULL::TEXT AS preferred_client,
            0::BIGINT AS prompts,
            0::BIGINT AS sessions,
            0::BIGINT AS tokens
        ",
    )
    .bind(user_id)
    .bind(&req.display_name)
    .bind(&req.email)
    .bind(&req.department)
    .bind(&req.roles)
    .bind(&status)
    .fetch_optional(pool.as_ref())
    .await
}

pub async fn delete_user(pool: &Arc<PgPool>, user_id: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(pool.as_ref())
        .await?;
    Ok(result.rows_affected() > 0)
}
