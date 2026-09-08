//! Per-skill adoption for the Skills tab, measured only.
//!
//! Invocations come from the `skill_invocation_events` view, which counts both
//! signals a client can send: a typed `/plugin:skill` slash command, and a
//! dispatched `Skill` tool call. Cost and tokens come from somewhere else
//! entirely — `ai_request_tool_calls`, joined on the tool-call id the `tool`
//! arm carries — so a slash invocation has **no** cost that can honestly be
//! named. It is reported in an unattributed bucket rather than estimated: the
//! same-user time-window heuristic that used to fill the column produced a
//! number no one could check, and a blank cell beside a stated count is more
//! useful than a plausible wrong one.
//!
//! Ratings live in `skill_ratings` under the bare skill id, while the view
//! names a skill `plugin:skill` with underscores rendered as hyphens, so the
//! join normalises both sides to the id after the colon.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct SkillStatsRow {
    pub skill: String,
    pub plugin_id: Option<String>,
    pub invocations: i64,
    pub slash_invocations: i64,
    pub tool_invocations: i64,
    pub distinct_users: i64,
    pub distinct_sessions: i64,
    // Why: invocations whose cost is measured, not inferred — a `Skill` tool
    // call with a matching `ai_request_tool_calls` row.
    pub measured_invocations: i64,
    pub measured_cost_microdollars: i64,
    pub measured_tokens: i64,
    pub avg_rating: Option<f64>,
    pub ratings: i64,
}

impl SkillStatsRow {
    // Why: invocations no gateway request can be tied to. Never folded into
    // the cost column — it is the size of what that column does not cover.
    #[must_use]
    pub const fn unattributed_invocations(&self) -> i64 {
        self.invocations - self.measured_invocations
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "one page assembly per handler; splitting is tracked in docs/tech-debt.md"
)]
pub async fn list_skill_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SkillStatsRow>, i64), sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        WITH events AS (
            SELECT e.skill, e.plugin_id, e.user_id, e.session_id, e.source, e.tool_use_id
            FROM skill_invocation_events e
            WHERE e.invoked_at >= $1 AND e.invoked_at < $2
              AND e.skill IS NOT NULL
              AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
              AND ($4::TEXT IS NULL OR e.user_id = $4)
        ),
        measured AS (
            SELECT ev.skill,
                   COUNT(*)::BIGINT AS invocations,
                   COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS cost,
                   COALESCE(SUM(COALESCE(r.input_tokens, 0)
                              + COALESCE(r.output_tokens, 0)), 0)::BIGINT AS tokens
            FROM events ev
            JOIN ai_request_tool_calls tc ON tc.ai_tool_call_id = ev.tool_use_id
            JOIN ai_requests r ON r.id = tc.request_id AND NOT r.synthetic
            GROUP BY ev.skill
        ),
        rated AS (
            SELECT lower(replace(sr.skill_name, '_', '-')) AS skill_id,
                   AVG(sr.rating::DOUBLE PRECISION) AS avg_rating,
                   COUNT(*)::BIGINT AS ratings
            FROM skill_ratings sr
            GROUP BY 1
        )
        SELECT
            ev.skill AS "skill!",
            MIN(ev.plugin_id) AS plugin_id,
            COUNT(*)::BIGINT AS "invocations!",
            COUNT(*) FILTER (WHERE ev.source = 'slash')::BIGINT AS "slash!",
            COUNT(*) FILTER (WHERE ev.source = 'tool')::BIGINT AS "tool!",
            COUNT(DISTINCT ev.user_id)::BIGINT AS "distinct_users!",
            COUNT(DISTINCT ev.session_id)::BIGINT AS "distinct_sessions!",
            COALESCE(MAX(m.invocations), 0)::BIGINT AS "measured!",
            COALESCE(MAX(m.cost), 0)::BIGINT AS "measured_cost!",
            COALESCE(MAX(m.tokens), 0)::BIGINT AS "measured_tokens!",
            MAX(rt.avg_rating) AS avg_rating,
            COALESCE(MAX(rt.ratings), 0)::BIGINT AS "ratings!",
            COUNT(*) OVER ()::BIGINT AS "total!"
        FROM events ev
        LEFT JOIN measured m ON m.skill = ev.skill
        LEFT JOIN rated rt
               ON rt.skill_id = lower(replace(split_part(ev.skill, ':', 2), '_', '-'))
        GROUP BY ev.skill
        ORDER BY COUNT(*) DESC, ev.skill
        LIMIT $5 OFFSET $6
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total);
    Ok((
        rows.into_iter()
            .map(|r| SkillStatsRow {
                skill: r.skill,
                plugin_id: r.plugin_id,
                invocations: r.invocations,
                slash_invocations: r.slash,
                tool_invocations: r.tool,
                distinct_users: r.distinct_users,
                distinct_sessions: r.distinct_sessions,
                measured_invocations: r.measured,
                measured_cost_microdollars: r.measured_cost,
                measured_tokens: r.measured_tokens,
                avg_rating: r.avg_rating,
                ratings: r.ratings,
            })
            .collect(),
        total,
    ))
}

/// Which model served a skill's measured work.
///
/// Only the tool-dispatched arm can answer this at all, so the table is empty
/// on an instance whose clients only ever type slash commands — which is itself
/// the finding.
#[derive(Debug, Clone)]
pub struct SkillModelRow {
    pub skill: String,
    pub model: String,
    pub requests: i64,
    pub cost_microdollars: i64,
}

pub async fn list_skill_by_model(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<Vec<SkillModelRow>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            e.skill AS "skill!",
            COALESCE(r.model, 'unrouted') AS "model!",
            COUNT(*)::BIGINT AS "requests!",
            COALESCE(SUM(r.cost_microdollars), 0)::BIGINT AS "cost!"
        FROM skill_invocation_events e
        JOIN ai_request_tool_calls tc ON tc.ai_tool_call_id = e.tool_use_id
        JOIN ai_requests r ON r.id = tc.request_id AND NOT r.synthetic
        WHERE e.invoked_at >= $1 AND e.invoked_at < $2
          AND e.skill IS NOT NULL
          AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR e.user_id = $4)
        GROUP BY e.skill, COALESCE(r.model, 'unrouted')
        ORDER BY COUNT(*) DESC
        LIMIT 100
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| SkillModelRow {
            skill: r.skill,
            model: r.model,
            requests: r.requests,
            cost_microdollars: r.cost,
        })
        .collect())
}

/// The window's totals across every skill, so the tab can state how much of
/// the invocation count carries a measured cost before showing the table.
#[derive(Debug, Default, Clone, Copy)]
pub struct SkillTotals {
    pub invocations: i64,
    pub measured_invocations: i64,
    pub distinct_skills: i64,
    pub distinct_users: i64,
}

pub async fn get_skill_totals(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SkillTotals, sqlx::Error> {
    let row = sqlx::query!(
        r#"
        SELECT
            COUNT(*)::BIGINT AS "invocations!",
            COUNT(*) FILTER (WHERE EXISTS (
                SELECT 1 FROM ai_request_tool_calls tc
                WHERE tc.ai_tool_call_id = e.tool_use_id
            ))::BIGINT AS "measured!",
            COUNT(DISTINCT e.skill)::BIGINT AS "skills!",
            COUNT(DISTINCT e.user_id)::BIGINT AS "users!"
        FROM skill_invocation_events e
        WHERE e.invoked_at >= $1 AND e.invoked_at < $2
          AND e.skill IS NOT NULL
          AND ($3::TEXT[] IS NULL OR e.user_id = ANY($3))
          AND ($4::TEXT IS NULL OR e.user_id = $4)
        "#,
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(SkillTotals {
        invocations: row.invocations,
        measured_invocations: row.measured,
        distinct_skills: row.skills,
        distinct_users: row.users,
    })
}
