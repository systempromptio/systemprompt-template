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

    /// Bound text key naming the column the `ORDER BY` `CASE` ladder switches
    /// on.
    const fn sql_key(self) -> &'static str {
        match self {
            Self::LastActive => "last_active",
            Self::Sessions => "sessions",
            Self::Contexts => "contexts",
            Self::Tokens => "tokens",
            Self::Denies => "denies",
            Self::Cost => "cost",
            Self::DisplayName => "display_name",
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

    /// Bound text key naming the direction the `ORDER BY` `CASE` ladder uses.
    const fn sql_key(self) -> &'static str {
        match self {
            Self::Asc => "asc",
            Self::Desc => "desc",
        }
    }
}

/// Aggregates `ai_requests` and `governance_decisions` per user for the
/// `/admin/overview/identity` table. Anonymous users are excluded the same
/// way `fetch_department_stats` excludes them.
///
/// Returns `(rows, total_count)`. `total_count` ignores limit/offset but
/// honours the search filter.
/// Sort/search/page inputs for [`fetch_user_identity_rows`] (was 5 trailing
/// positional args).
#[derive(Debug, Clone, Copy)]
pub struct IdentityQuery<'a> {
    pub sort: IdentitySort,
    pub dir: SortDir,
    pub search: Option<&'a str>,
    pub limit: i64,
    pub offset: i64,
}

async fn count_user_identity_rows(
    pool: &PgPool,
    search_pattern: Option<&str>,
) -> Result<i64, sqlx::Error> {
    sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::BIGINT AS "count!"
        FROM users u
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text IS NULL
               OR LOWER(COALESCE(u.display_name, u.full_name, u.name, '')) LIKE $1
               OR LOWER(COALESCE(u.email, '')) LIKE $1
               OR LOWER(COALESCE(upe.department, '')) LIKE $1)
        "#,
        search_pattern,
    )
    .fetch_one(pool)
    .await
}

#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn fetch_user_identity_rows(
    pool: &PgPool,
    query: IdentityQuery<'_>,
) -> Result<(Vec<crate::types::UserIdentityRow>, i64), sqlx::Error> {
    let IdentityQuery {
        sort,
        dir,
        search,
        limit,
        offset,
    } = query;
    let search_pattern = search
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.to_lowercase()));

    let sort_col = sort.sql_key();
    let sort_dir = dir.sql_key();

    let total = count_user_identity_rows(pool, search_pattern.as_deref()).await?;

    // Why: sort column/dir are bound as text ($4/$5) and switched via a
    // per-key `CASE` in the `ORDER BY` so the statement stays a single
    // compile-time `query_as!` rather than interpolated (raw) SQL.
    let rows = sqlx::query_as!(
        crate::types::UserIdentityRow,
        r#"
        SELECT
            u.id AS "user_id!",
            COALESCE(u.display_name, u.full_name, u.name) AS "display_name?",
            u.email AS "email?",
            COALESCE(NULLIF(upe.department, ''), 'Unassigned') AS "department!",
            (u.status = 'active') AS "is_active!",
            ar.last_active AS "last_active?",
            COALESCE(ar.requests, 0)::BIGINT AS "requests!",
            COALESCE(ar.sessions, 0)::BIGINT AS "sessions!",
            COALESCE(ar.contexts, 0)::BIGINT AS "contexts!",
            COALESCE(ar.models, 0)::BIGINT AS "models!",
            COALESCE(ar.tokens, 0)::BIGINT AS "tokens!",
            COALESCE(ar.cost_microdollars, 0)::BIGINT AS "cost_microdollars!",
            COALESCE(g.denies, 0)::BIGINT AS "denies!",
            COALESCE(g.secret_breaches, 0)::BIGINT AS "secret_breaches!",
            COALESCE(g.scope_violations, 0)::BIGINT AS "scope_violations!"
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
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
          AND ($1::text IS NULL
               OR LOWER(COALESCE(u.display_name, u.full_name, u.name, '')) LIKE $1
               OR LOWER(COALESCE(u.email, '')) LIKE $1
               OR LOWER(COALESCE(upe.department, '')) LIKE $1)
        ORDER BY
            (CASE WHEN $4 = 'last_active' AND $5 = 'asc'  THEN ar.last_active END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'last_active' AND $5 = 'desc' THEN ar.last_active END) DESC NULLS LAST,
            (CASE WHEN $4 = 'sessions' AND $5 = 'asc'  THEN COALESCE(ar.sessions, 0) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'sessions' AND $5 = 'desc' THEN COALESCE(ar.sessions, 0) END) DESC NULLS LAST,
            (CASE WHEN $4 = 'contexts' AND $5 = 'asc'  THEN COALESCE(ar.contexts, 0) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'contexts' AND $5 = 'desc' THEN COALESCE(ar.contexts, 0) END) DESC NULLS LAST,
            (CASE WHEN $4 = 'tokens' AND $5 = 'asc'  THEN COALESCE(ar.tokens, 0) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'tokens' AND $5 = 'desc' THEN COALESCE(ar.tokens, 0) END) DESC NULLS LAST,
            (CASE WHEN $4 = 'denies' AND $5 = 'asc'  THEN COALESCE(g.denies, 0) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'denies' AND $5 = 'desc' THEN COALESCE(g.denies, 0) END) DESC NULLS LAST,
            (CASE WHEN $4 = 'cost' AND $5 = 'asc'  THEN COALESCE(ar.cost_microdollars, 0) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'cost' AND $5 = 'desc' THEN COALESCE(ar.cost_microdollars, 0) END) DESC NULLS LAST,
            (CASE WHEN $4 = 'display_name' AND $5 = 'asc'  THEN COALESCE(u.display_name, u.full_name, u.name) END) ASC  NULLS LAST,
            (CASE WHEN $4 = 'display_name' AND $5 = 'desc' THEN COALESCE(u.display_name, u.full_name, u.name) END) DESC NULLS LAST,
            (CASE WHEN $4 IN ('sessions','contexts','tokens','denies','cost') THEN ar.last_active END) DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#,
        search_pattern.as_deref(),
        limit,
        offset,
        sort_col,
        sort_dir,
    )
    .fetch_all(pool)
    .await?;

    Ok((rows, total))
}

pub async fn fetch_department_stats(
    pool: &PgPool,
) -> Result<Vec<crate::types::DepartmentStats>, sqlx::Error> {
    sqlx::query_as!(
        crate::types::DepartmentStats,
        r#"
        SELECT
            COALESCE(NULLIF(upe.department, ''), 'Unassigned') AS "department!",
            COUNT(DISTINCT u.id)::BIGINT AS "user_count!",
            COUNT(DISTINCT u.id) FILTER (WHERE u.status = 'active')::BIGINT AS "active_count!",
            COALESCE(SUM(ev.event_count), 0)::BIGINT AS "total_events!",
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '24 hours')::BIGINT AS "active_24h!",
            COUNT(DISTINCT u.id) FILTER (WHERE ev.last_event >= NOW() - INTERVAL '7 days')::BIGINT AS "active_7d!",
            COALESCE(SUM(tok.total_tokens), 0)::BIGINT AS "total_tokens!",
            COALESCE(SUM(ev.prompt_count), 0)::BIGINT AS "total_prompts!",
            COALESCE(SUM(ev.session_count), 0)::BIGINT AS "total_sessions!",
            COALESCE(SUM(ev.sessions_this_week), 0)::BIGINT AS "sessions_this_week!",
            COALESCE(SUM(ev.sessions_prev_week), 0)::BIGINT AS "sessions_prev_week!"
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
        LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'
        GROUP BY COALESCE(NULLIF(upe.department, ''), 'Unassigned')
        ORDER BY 2 DESC
        "#,
    )
    .fetch_all(pool)
    .await
}
