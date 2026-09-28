//! Skill adoption and the conversations it reached, for the analytics Skills
//! tab.
//!
//! A skill is not an execution. Invoking one pastes its instructions into the
//! conversation and costs nothing of its own; every figure here is therefore
//! either a count of invocations or a fact about the **conversations** that
//! invoked the skill. The rows read the same facts the Analysis section reads
//! — `analysis_skill_version_events`, joined to gateway requests on the owner
//! and the client's native session id, with synthetic requests excluded —
//! so a number here agrees with the number there.
//!
//! Per-skill conversation spend overlaps: a conversation that invoked two
//! skills is counted under both, so the rows must never be summed. The totals
//! query deduplicates conversations and requests for the honest window figure.

use sqlx::PgPool;

use crate::util::time_range::TimeRange;

use super::SiteScope;

#[derive(Debug, Clone)]
pub struct SkillStatsRow {
    pub skill: String,
    // Why: the managed resource this skill resolved to through a verified
    // installation receipt, when it did — the key the Analysis pages use.
    pub resource_id: Option<String>,
    pub invocations: i64,
    pub slash_invocations: i64,
    pub tool_invocations: i64,
    pub attributed_invocations: i64,
    pub distinct_users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub priced_requests: i64,
    pub conversation_cost_microdollars: i64,
    pub conversation_tokens: i64,
}

pub async fn list_skill_stats(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SkillStatsRow>, i64), sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analytics/site/skills.sql",
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
                resource_id: r.resource_id,
                invocations: r.invocations,
                slash_invocations: r.slash,
                tool_invocations: r.tool,
                attributed_invocations: r.attributed,
                distinct_users: r.distinct_users,
                conversations: r.conversations,
                requests: r.requests,
                priced_requests: r.priced_requests,
                conversation_cost_microdollars: r.cost,
                conversation_tokens: r.tokens,
            })
            .collect(),
        total,
    ))
}

/// The window's totals across every skill, with conversations and requests
/// counted once each however many skills they touched.
#[derive(Debug, Default, Clone, Copy)]
pub struct SkillTotals {
    pub invocations: i64,
    pub attributed_invocations: i64,
    pub distinct_skills: i64,
    pub distinct_users: i64,
    pub conversations: i64,
    pub requests: i64,
    pub conversation_cost_microdollars: i64,
}

pub async fn get_skill_totals(
    pool: &PgPool,
    range: TimeRange,
    scope: &SiteScope,
) -> Result<SkillTotals, sqlx::Error> {
    let row = sqlx::query_file!(
        "src/repositories/analytics/site/skills_totals.sql",
        range.from,
        range.to,
        scope.scope.as_sql(),
        scope.user_id_str(),
    )
    .fetch_one(pool)
    .await?;

    Ok(SkillTotals {
        invocations: row.invocations,
        attributed_invocations: row.attributed,
        distinct_skills: row.skills,
        distinct_users: row.users,
        conversations: row.conversations,
        requests: row.requests,
        conversation_cost_microdollars: row.cost,
    })
}
