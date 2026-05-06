use sqlx::PgPool;

#[derive(Debug, Clone, Copy)]
pub enum IdentitySort {
    LastActive,
    Sessions,
    Contexts,
    Tokens,
    Denies,
    Cost,
    DisplayName,
}

impl IdentitySort {
    pub fn parse(s: Option<&str>) -> Self {
        match s.unwrap_or("") {
            "sessions" => Self::Sessions,
            "contexts" => Self::Contexts,
            "tokens" => Self::Tokens,
            "denies" => Self::Denies,
            "cost" => Self::Cost,
            "name" => Self::DisplayName,
            _ => Self::LastActive,
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::LastActive => "last_active",
            Self::Sessions => "sessions",
            Self::Contexts => "contexts",
            Self::Tokens => "tokens",
            Self::Denies => "denies",
            Self::Cost => "cost",
            Self::DisplayName => "name",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum SortDir {
    Asc,
    Desc,
}

impl SortDir {
    pub fn parse(s: Option<&str>) -> Self {
        if matches!(s, Some("asc")) {
            Self::Asc
        } else {
            Self::Desc
        }
    }

    pub const fn slug(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

const fn order_clause(sort: IdentitySort, dir: SortDir) -> &'static str {
    match (sort, dir) {
        (IdentitySort::LastActive, SortDir::Asc) => "last_active ASC NULLS LAST",
        (IdentitySort::LastActive, SortDir::Desc) => "last_active DESC NULLS LAST",
        (IdentitySort::Sessions, SortDir::Asc) => "sessions ASC, last_active DESC NULLS LAST",
        (IdentitySort::Sessions, SortDir::Desc) => "sessions DESC, last_active DESC NULLS LAST",
        (IdentitySort::Contexts, SortDir::Asc) => "contexts ASC, last_active DESC NULLS LAST",
        (IdentitySort::Contexts, SortDir::Desc) => "contexts DESC, last_active DESC NULLS LAST",
        (IdentitySort::Tokens, SortDir::Asc) => "tokens ASC, last_active DESC NULLS LAST",
        (IdentitySort::Tokens, SortDir::Desc) => "tokens DESC, last_active DESC NULLS LAST",
        (IdentitySort::Denies, SortDir::Asc) => "denies ASC, last_active DESC NULLS LAST",
        (IdentitySort::Denies, SortDir::Desc) => "denies DESC, last_active DESC NULLS LAST",
        (IdentitySort::Cost, SortDir::Asc) => "cost_microdollars ASC, last_active DESC NULLS LAST",
        (IdentitySort::Cost, SortDir::Desc) => {
            "cost_microdollars DESC, last_active DESC NULLS LAST"
        }
        (IdentitySort::DisplayName, SortDir::Asc) => "display_name ASC NULLS LAST",
        (IdentitySort::DisplayName, SortDir::Desc) => "display_name DESC NULLS LAST",
    }
}

/// Aggregates `ai_requests` and `governance_decisions` per user for the
/// `/admin/overview/identity` table. Anonymous users are excluded the same
/// way `fetch_department_stats` excludes them.
///
/// Returns `(rows, total_count)`. `total_count` ignores limit/offset but
/// honours the search filter.
pub async fn fetch_user_identity_rows(
    pool: &PgPool,
    sort: IdentitySort,
    dir: SortDir,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<crate::types::UserIdentityRow>, i64), sqlx::Error> {
    let search_pattern = search
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.to_lowercase()));

    let order = order_clause(sort, dir);

    let count_sql = r"
        SELECT COUNT(*)::BIGINT
        FROM users u
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text IS NULL
               OR LOWER(COALESCE(u.display_name, u.full_name, u.name, '')) LIKE $1
               OR LOWER(COALESCE(u.email, '')) LIKE $1
               OR LOWER(COALESCE(u.department, '')) LIKE $1)
    ";

    let total: i64 = sqlx::query_scalar(count_sql)
        .bind(search_pattern.as_deref())
        .fetch_one(pool)
        .await?;

    let rows_sql = format!(
        r"
        SELECT
            u.id AS user_id,
            COALESCE(u.display_name, u.full_name, u.name) AS display_name,
            u.email AS email,
            COALESCE(NULLIF(u.department, ''), 'Unassigned') AS department,
            (u.status = 'active') AS is_active,
            ar.last_active AS last_active,
            COALESCE(ar.requests, 0)::BIGINT AS requests,
            COALESCE(ar.sessions, 0)::BIGINT AS sessions,
            COALESCE(ar.contexts, 0)::BIGINT AS contexts,
            COALESCE(ar.models, 0)::BIGINT AS models,
            COALESCE(ar.tokens, 0)::BIGINT AS tokens,
            COALESCE(ar.cost_microdollars, 0)::BIGINT AS cost_microdollars,
            COALESCE(g.denies, 0)::BIGINT AS denies,
            COALESCE(g.secret_breaches, 0)::BIGINT AS secret_breaches,
            COALESCE(g.scope_violations, 0)::BIGINT AS scope_violations
        FROM users u
        LEFT JOIN (
            SELECT
                user_id,
                COUNT(*)::BIGINT AS requests,
                COUNT(DISTINCT session_id)::BIGINT AS sessions,
                COUNT(DISTINCT context_id)::BIGINT AS contexts,
                COUNT(DISTINCT model)::BIGINT AS models,
                (COALESCE(SUM(input_tokens), 0) + COALESCE(SUM(output_tokens), 0))::BIGINT AS tokens,
                COALESCE(SUM(cost_microdollars), 0)::BIGINT AS cost_microdollars,
                MAX(created_at) AS last_active
            FROM ai_requests
            GROUP BY user_id
        ) ar ON ar.user_id = u.id
        LEFT JOIN (
            SELECT
                user_id,
                COUNT(*) FILTER (WHERE decision = 'deny')::BIGINT AS denies,
                COUNT(*) FILTER (WHERE decision = 'deny' AND policy ILIKE '%secret%')::BIGINT AS secret_breaches,
                COUNT(*) FILTER (WHERE decision = 'deny' AND policy ILIKE '%scope%')::BIGINT AS scope_violations
            FROM governance_decisions
            GROUP BY user_id
        ) g ON g.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text IS NULL
               OR LOWER(COALESCE(u.display_name, u.full_name, u.name, '')) LIKE $1
               OR LOWER(COALESCE(u.email, '')) LIKE $1
               OR LOWER(COALESCE(u.department, '')) LIKE $1)
        ORDER BY {order}
        LIMIT $2 OFFSET $3
        ",
    );

    let rows = sqlx::query_as::<_, crate::types::UserIdentityRow>(&rows_sql)
        .bind(search_pattern.as_deref())
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?;

    Ok((rows, total))
}

pub async fn fetch_department_stats(
    pool: &PgPool,
) -> Result<Vec<crate::types::DepartmentStats>, sqlx::Error> {
    sqlx::query_as::<_, crate::types::DepartmentStats>(
        r"
        SELECT
            COALESCE(NULLIF(u.department, ''), 'Unassigned') AS department,
            COUNT(DISTINCT u.id)::BIGINT AS user_count,
            COUNT(DISTINCT u.id) FILTER (WHERE u.status = 'active')::BIGINT AS active_count,
            COALESCE(SUM(ev.event_count), 0)::BIGINT AS total_events,
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '24 hours')::BIGINT AS active_24h,
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '7 days')::BIGINT AS active_7d,
            COALESCE(SUM(tok.total_tokens), 0)::BIGINT AS total_tokens,
            COALESCE(SUM(ev.prompt_count), 0)::BIGINT AS total_prompts,
            COALESCE(SUM(ev.session_count), 0)::BIGINT AS total_sessions,
            COALESCE(SUM(ev.sessions_this_week), 0)::BIGINT AS sessions_this_week,
            COALESCE(SUM(ev.sessions_prev_week), 0)::BIGINT AS sessions_prev_week
        FROM users u
        LEFT JOIN (
            SELECT
                user_id,
                COUNT(*)::BIGINT AS event_count,
                COUNT(*) FILTER (WHERE event_type LIKE '%UserPromptSubmit%')::BIGINT AS prompt_count,
                COUNT(DISTINCT session_id)::BIGINT AS session_count,
                COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - INTERVAL '7 days')::BIGINT AS sessions_this_week,
                COUNT(DISTINCT session_id) FILTER (WHERE created_at >= NOW() - INTERVAL '14 days' AND created_at < NOW() - INTERVAL '7 days')::BIGINT AS sessions_prev_week,
                MAX(created_at) AS last_event
            FROM plugin_usage_events
            GROUP BY user_id
        ) ev ON ev.user_id = u.id
        LEFT JOIN (
            SELECT
                user_id,
                (COALESCE(SUM(total_input_tokens), 0) + COALESCE(SUM(total_output_tokens), 0))::BIGINT AS total_tokens
            FROM plugin_usage_daily
            GROUP BY user_id
        ) tok ON tok.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY COALESCE(NULLIF(u.department, ''), 'Unassigned')
        ORDER BY user_count DESC
        ",
    )
    .fetch_all(pool)
    .await
}
