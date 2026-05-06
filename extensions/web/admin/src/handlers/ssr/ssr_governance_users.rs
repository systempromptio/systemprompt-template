use std::sync::Arc;

use axum::{
    extract::{Extension, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
};
use serde_json::json;
use sqlx::PgPool;

use crate::repositories;
use crate::templates::AdminTemplateEngine;
use crate::types::{MarketplaceContext, UserContext};

use super::ACCESS_DENIED_HTML;

#[derive(Debug, sqlx::FromRow)]
struct UserWithDeptRow {
    user_id: String,
    display_name: String,
    department: String,
    total_tokens: i64,
    total_prompts: i64,
    total_sessions: i64,
}

pub async fn governance_users_page(
    Extension(user_ctx): Extension<UserContext>,
    Extension(mkt_ctx): Extension<MarketplaceContext>,
    Extension(engine): Extension<AdminTemplateEngine>,
    State(pool): State<Arc<PgPool>>,
) -> Response {
    if !user_ctx.is_admin {
        return (StatusCode::FORBIDDEN, Html(ACCESS_DENIED_HTML)).into_response();
    }

    let (dept_res, users_res) = tokio::join!(
        repositories::fetch_department_stats(&pool),
        sqlx::query_as::<_, UserWithDeptRow>(
            r"SELECT
                u.id AS user_id,
                COALESCE(u.display_name, u.full_name, u.name, u.email, u.id) AS display_name,
                COALESCE(NULLIF(u.department, ''), 'Unassigned') AS department,
                COALESCE(tok.total_tokens, 0)::BIGINT AS total_tokens,
                COALESCE(ev.prompt_count, 0)::BIGINT AS total_prompts,
                COALESCE(ev.session_count, 0)::BIGINT AS total_sessions
              FROM users u
              LEFT JOIN (
                  SELECT user_id,
                         (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0))::BIGINT AS total_tokens
                  FROM plugin_usage_daily
                  GROUP BY user_id
              ) tok ON tok.user_id = u.id
              LEFT JOIN (
                  SELECT user_id,
                         COUNT(*) FILTER (WHERE event_type LIKE '%UserPromptSubmit%')::BIGINT AS prompt_count,
                         COUNT(DISTINCT session_id)::BIGINT AS session_count
                  FROM plugin_usage_events
                  GROUP BY user_id
              ) ev ON ev.user_id = u.id
              WHERE NOT ('anonymous' = ANY(u.roles))
                AND u.email NOT LIKE '%@anonymous.local'
              ORDER BY COALESCE(tok.total_tokens, 0) DESC
              LIMIT 200"
        )
        .fetch_all(&*pool),
    );

    let dept_stats = dept_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch department stats for governance/users");
        vec![]
    });

    let user_rows = users_res.unwrap_or_else(|e| {
        tracing::warn!(error = %e, "Failed to fetch users with dept for governance/users");
        vec![]
    });

    let total_users: i64 = dept_stats.iter().map(|d| d.user_count).sum();
    let active_24h: i64 = dept_stats.iter().map(|d| d.active_24h).sum();
    let total_tokens: i64 = dept_stats.iter().map(|d| d.total_tokens).sum();
    let total_prompts: i64 = dept_stats.iter().map(|d| d.total_prompts).sum();

    let departments: Vec<serde_json::Value> = dept_stats
        .iter()
        .map(|d| {
            json!({
                "department": d.department,
                "user_count": d.user_count,
                "active_count": d.active_count,
                "active_24h": d.active_24h,
                "total_tokens": d.total_tokens,
                "total_prompts": d.total_prompts,
                "sessions_this_week": d.sessions_this_week,
            })
        })
        .collect();

    let users: Vec<serde_json::Value> = user_rows
        .iter()
        .map(|u| {
            json!({
                "user_id": u.user_id,
                "display_name": u.display_name,
                "department": u.department,
                "total_tokens": u.total_tokens,
                "total_prompts": u.total_prompts,
                "total_sessions": u.total_sessions,
            })
        })
        .collect();

    let data = json!({
        "page": "governance-users",
        "title": "Users & Departments",
        "total_users": total_users,
        "active_24h": active_24h,
        "total_tokens": total_tokens,
        "total_prompts": total_prompts,
        "departments": departments,
        "has_departments": !departments.is_empty(),
        "users": users,
        "has_users": !users.is_empty(),
    });

    super::render_page(&engine, "governance-users", &data, &user_ctx, &mkt_ctx)
}
