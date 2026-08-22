//! Paged session list — one row per session id, with its gateway and hook
//! rollups side by side.

use sqlx::PgPool;
use systemprompt::identifiers::{PluginId, SessionId, UserId};

use super::{SessionListFilter, SessionListItem, SessionPage};
use crate::util::time_range::TimeRange;

#[derive(Debug)]
struct SessionRow {
    session_id: SessionId,
    user_id: Option<UserId>,
    display_name: Option<String>,
    department: Option<String>,
    model: Option<String>,
    ai_title: Option<String>,
    plugin_id: Option<PluginId>,
    client_source: Option<String>,
    permission_mode: Option<String>,
    status: Option<String>,
    started_at: Option<chrono::DateTime<chrono::Utc>>,
    last_activity_at: Option<chrono::DateTime<chrono::Utc>>,
    request_count: i64,
    context_count: i64,
    trace_count: i64,
    tool_uses: i64,
    prompts: i64,
    error_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cost_microdollars: i64,
    has_gateway: bool,
    has_hooks: bool,
    total_count: i64,
}

impl From<SessionRow> for SessionListItem {
    fn from(r: SessionRow) -> Self {
        Self {
            session_id: r.session_id,
            user_id: r.user_id,
            display_name: r.display_name,
            department: r.department,
            model: r.model,
            ai_title: r.ai_title,
            plugin_id: r.plugin_id,
            client_source: r.client_source,
            permission_mode: r.permission_mode,
            status: r.status,
            started_at: r.started_at,
            last_activity_at: r.last_activity_at,
            request_count: r.request_count,
            context_count: r.context_count,
            trace_count: r.trace_count,
            tool_uses: r.tool_uses,
            prompts: r.prompts,
            error_count: r.error_count,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            total_cost_microdollars: r.total_cost_microdollars,
            has_gateway: r.has_gateway,
            has_hooks: r.has_hooks,
        }
    }
}

// Why: The count ignores `page` and covers every row the filter matches, so a
// caller can render "page N of M" without a second round trip.
#[expect(
    clippy::too_many_lines,
    reason = "body is one irreducible compile-time-checked query_as! SQL literal"
)]
pub async fn list_sessions_paged(
    pool: &PgPool,
    filter: &SessionListFilter,
    range: TimeRange,
    page: SessionPage,
) -> Result<(Vec<SessionListItem>, i64), sqlx::Error> {
    let rows = sqlx::query_as!(
        SessionRow,
        r#"
        WITH req AS (
            SELECT
                session_id,
                MAX(user_id)                                       AS user_id,
                COUNT(*)::bigint                                   AS request_count,
                COUNT(DISTINCT context_id)::bigint                 AS context_count,
                COUNT(DISTINCT trace_id)::bigint                   AS trace_count,
                COUNT(*) FILTER (WHERE status = 'failed')::bigint  AS error_count,
                COALESCE(SUM(input_tokens), 0)::bigint             AS total_input_tokens,
                COALESCE(SUM(output_tokens), 0)::bigint            AS total_output_tokens,
                COALESCE(SUM(cost_microdollars), 0)::bigint        AS total_cost_microdollars,
                MIN(created_at)                                    AS first_seen,
                MAX(created_at)                                    AS last_seen,
                (ARRAY_AGG(model ORDER BY created_at DESC))[1]     AS model
            FROM ai_requests
            WHERE session_id IS NOT NULL
            GROUP BY session_id
        ),
        joined AS (
            SELECT
                COALESCE(s.session_id, req.session_id)              AS session_id,
                COALESCE(s.user_id, req.user_id)                    AS user_id,
                u.display_name                                      AS display_name,
                upe.department                                      AS department,
                COALESCE(s.model, req.model)                        AS model,
                s.ai_title                                          AS ai_title,
                s.plugin_id                                         AS plugin_id,
                s.client_source                                     AS client_source,
                s.permission_mode                                   AS permission_mode,
                s.status                                            AS status,
                COALESCE(s.started_at, req.first_seen)              AS started_at,
                GREATEST(
                    COALESCE(req.last_seen, s.ended_at, s.started_at),
                    COALESCE(s.ended_at, s.started_at, req.last_seen)
                )                                                   AS last_activity_at,
                COALESCE(req.request_count, 0)                      AS request_count,
                COALESCE(req.context_count, 0)                      AS context_count,
                COALESCE(req.trace_count, 0)                        AS trace_count,
                COALESCE(s.tool_uses, 0)                            AS tool_uses,
                COALESCE(s.prompts, 0)                              AS prompts,
                COALESCE(req.error_count, 0) + COALESCE(s.errors, 0) AS error_count,
                COALESCE(req.total_input_tokens, 0)
                    + COALESCE(s.total_input_tokens, 0)             AS total_input_tokens,
                COALESCE(req.total_output_tokens, 0)
                    + COALESCE(s.total_output_tokens, 0)            AS total_output_tokens,
                COALESCE(req.total_cost_microdollars, 0)            AS total_cost_microdollars,
                (req.session_id IS NOT NULL)                        AS has_gateway,
                (s.session_id IS NOT NULL)                          AS has_hooks
            FROM plugin_session_summaries s
            FULL OUTER JOIN req ON req.session_id = s.session_id
            LEFT JOIN users u ON u.id = COALESCE(s.user_id, req.user_id)
            LEFT JOIN user_profile_ext upe ON upe.user_id = u.id
        ),
        filtered AS (
            SELECT j.* FROM joined j
            WHERE j.last_activity_at >= $1
              AND j.last_activity_at < $2
              AND ($3::text IS NULL OR j.user_id = $3)
              AND (NOT $4 OR j.error_count > 0)
        ),
        counted AS (
            SELECT f.*, COUNT(*) OVER ()::bigint AS total_count FROM filtered f
        )
        SELECT
            session_id              AS "session_id!: SessionId",
            user_id                 AS "user_id?: UserId",
            display_name            AS "display_name?",
            department              AS "department?",
            model                   AS "model?",
            ai_title                AS "ai_title?",
            plugin_id               AS "plugin_id?: PluginId",
            client_source           AS "client_source?",
            permission_mode         AS "permission_mode?",
            status                  AS "status?",
            started_at              AS "started_at?",
            last_activity_at        AS "last_activity_at?",
            request_count           AS "request_count!",
            context_count           AS "context_count!",
            trace_count             AS "trace_count!",
            tool_uses               AS "tool_uses!",
            prompts                 AS "prompts!",
            error_count             AS "error_count!",
            total_input_tokens      AS "total_input_tokens!",
            total_output_tokens     AS "total_output_tokens!",
            total_cost_microdollars AS "total_cost_microdollars!",
            has_gateway             AS "has_gateway!",
            has_hooks               AS "has_hooks!",
            total_count             AS "total_count!"
        FROM counted
        ORDER BY started_at DESC NULLS LAST, session_id
        LIMIT $5 OFFSET $6
        "#,
        range.from,
        range.to,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.error_only,
        page.limit,
        page.offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    Ok((rows.into_iter().map(SessionListItem::from).collect(), total))
}
