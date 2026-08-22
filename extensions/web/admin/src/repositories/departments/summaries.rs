//! Read-only rollups for the department dashboard: per-department usage
//! summaries, member lists, and top-tool breakdowns over the last 30 days.

use sqlx::PgPool;

use crate::types::departments::{DepartmentMember, DepartmentSummary, DepartmentTopTool};

pub async fn list_department_names(pool: &PgPool) -> Result<Vec<String>, sqlx::Error> {
    sqlx::query_scalar!("SELECT name FROM departments ORDER BY name")
        .fetch_all(pool)
        .await
}

pub async fn list_departments(pool: &PgPool) -> Result<Vec<DepartmentSummary>, sqlx::Error> {
    sqlx::query_as!(
        DepartmentSummary,
        r#"
        SELECT
            d.id,
            d.name,
            d.description as "description!",
            COALESCE(mc.member_count, 0)::BIGINT  AS "member_count!",
            COALESCE(ac.assignment_count, 0)::BIGINT AS "assignment_count!",
            COALESCE(usg.input_tokens, 0)::BIGINT  AS "input_tokens!",
            COALESCE(usg.output_tokens, 0)::BIGINT AS "output_tokens!",
            COALESCE(usg.requests, 0)::BIGINT      AS "requests!",
            COALESCE(usg.cost_microdollars, 0)::BIGINT AS "cost_microdollars!",
            d.created_at,
            d.updated_at
        FROM departments d
        -- Driven from `users`, not from `user_profile_ext`: a user whose side
        -- row has not been created yet still belongs to Default, and joining
        -- through the side table dropped them from the count entirely.
        LEFT JOIN (
            SELECT COALESCE(NULLIF(upe.department, ''), 'Default') AS department,
                   COUNT(*)::BIGINT AS member_count
            FROM users u
            LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
            WHERE NOT ('anonymous' = ANY(u.roles))
              AND u.email NOT LIKE '%@anonymous.local'
            GROUP BY 1
        ) mc ON mc.department = d.name
        LEFT JOIN (
            SELECT rule_value, COUNT(*)::BIGINT AS assignment_count
            FROM access_control_rules
            WHERE rule_type = 'department'
            GROUP BY rule_value
        ) ac ON ac.rule_value = d.name
        LEFT JOIN (
            SELECT
                COALESCE(NULLIF(upe.department, ''), 'Default') AS dept,
                COALESCE(SUM(ar.input_tokens), 0)::BIGINT  AS input_tokens,
                COALESCE(SUM(ar.output_tokens), 0)::BIGINT AS output_tokens,
                COUNT(ar.id)::BIGINT                       AS requests,
                COALESCE(SUM(ar.cost_microdollars), 0)::BIGINT AS cost_microdollars
            FROM ai_requests ar
            JOIN user_profile_ext upe ON upe.user_id = ar.user_id
            WHERE ar.created_at >= NOW() - INTERVAL '30 days'
            GROUP BY 1
        ) usg ON usg.dept = d.name
        ORDER BY d.name
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_department_members(
    pool: &PgPool,
    department_name: &str,
) -> Result<Vec<DepartmentMember>, sqlx::Error> {
    sqlx::query_as!(
        DepartmentMember,
        r#"
        SELECT
            u.id,
            u.email,
            u.display_name,
            u.status as "status!",
            u.roles as "roles!: Vec<String>",
            COALESCE(ar.input_tokens, 0)::BIGINT     AS "input_tokens!",
            COALESCE(ar.output_tokens, 0)::BIGINT    AS "output_tokens!",
            COALESCE(ar.requests, 0)::BIGINT         AS "requests!",
            COALESCE(ar.cost_microdollars, 0)::BIGINT AS "cost_microdollars!",
            ar.last_active                           AS last_active
        FROM users u
        LEFT JOIN (
            SELECT
                user_id,
                COALESCE(SUM(input_tokens), 0)::BIGINT  AS input_tokens,
                COALESCE(SUM(output_tokens), 0)::BIGINT AS output_tokens,
                COUNT(*)::BIGINT                        AS requests,
                COALESCE(SUM(cost_microdollars), 0)::BIGINT AS cost_microdollars,
                MAX(created_at)                         AS last_active
            FROM ai_requests
            WHERE created_at >= NOW() - INTERVAL '30 days'
            GROUP BY user_id
        ) ar ON ar.user_id = u.id
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE COALESCE(NULLIF(upe.department, ''), 'Default') = $1
          AND NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        ORDER BY (COALESCE(ar.input_tokens, 0) + COALESCE(ar.output_tokens, 0)) DESC, u.email
        "#,
        department_name,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_department_top_tools(
    pool: &PgPool,
    department_name: &str,
    limit: i64,
) -> Result<Vec<DepartmentTopTool>, sqlx::Error> {
    sqlx::query_as!(
        DepartmentTopTool,
        r#"
        SELECT
            COALESCE(p.tool_name, 'unknown') AS "tool_name!",
            COALESCE(SUM(p.event_count), 0)::BIGINT AS "invocations!"
        FROM plugin_usage_daily p
        JOIN user_profile_ext upe ON upe.user_id = p.user_id
        WHERE COALESCE(NULLIF(upe.department, ''), 'Default') = $1
          AND p.tool_name IS NOT NULL
          AND p.date >= CURRENT_DATE - INTERVAL '30 days'
        GROUP BY p.tool_name
        ORDER BY 2 DESC
        LIMIT $2
        "#,
        department_name,
        limit,
    )
    .fetch_all(pool)
    .await
}
