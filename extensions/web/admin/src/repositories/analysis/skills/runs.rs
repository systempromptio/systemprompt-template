//! Skill runs: one harness session that invoked a marketplace's skill,
//! timed from the first invocation, with its human wait split out and, for a
//! skill that logs a workflow, the state it reached and time per phase.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, MarketplaceId, SessionId};

use super::SkillWindow;

/// One skill run. Seconds are wall-clock; the waits are the parts of it the
/// agent spent blocked on a person.
#[derive(Debug, Clone)]
pub struct SkillRunRow {
    pub context_id: ContextId,
    pub session_id: SessionId,
    pub skill: String,
    pub marketplace_id: Option<MarketplaceId>,
    pub kit_hash: Option<String>,
    pub kit_version: Option<String>,
    pub started_at: DateTime<Utc>,
    pub ended_at: DateTime<Utc>,
    pub wall_s: f64,
    pub prompt_wait_s: f64,
    pub question_wait_s: f64,
    pub prompts: i64,
    pub questions: i64,
    pub turns: i64,
    pub tool_calls: i64,
    pub tool_failures: i64,
    pub errors: i64,
    pub rejected: i64,
    pub cost_microdollars: i64,
    pub contexts: i64,
    pub outcome: Option<String>,
    pub completion: Option<i16>,
    pub terminal_state: Option<String>,
    pub transitions: i64,
    pub requirements_s: Option<f64>,
    pub design_s: Option<f64>,
    pub build_s: Option<f64>,
    pub verify_s: Option<f64>,
    pub ship_s: Option<f64>,
}

/// Which runs to read: a marketplace's skills, or one `plugin:skill` key.
#[derive(Debug, Clone, Default)]
pub struct SkillRunFilter {
    pub marketplace: Option<String>,
    pub skill: Option<String>,
}

pub async fn list_skill_runs(
    pool: &PgPool,
    window: &SkillWindow,
    filter: &SkillRunFilter,
    limit: i64,
) -> Result<Vec<SkillRunRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/runs.sql",
        window.start,
        window.end,
        filter.marketplace.as_deref(),
        filter.skill.as_deref(),
        window.subject_ids.as_deref(),
        limit.clamp(1, 5_000),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SkillRunRow {
            context_id: r.context_id,
            session_id: SessionId::new(r.session_id),
            skill: r.skill,
            marketplace_id: r.marketplace_id.map(MarketplaceId::new),
            kit_hash: r.kit_hash,
            kit_version: r.kit_version,
            started_at: r.started_at,
            ended_at: r.ended_at,
            wall_s: r.wall_s,
            prompt_wait_s: r.prompt_wait_s,
            question_wait_s: r.question_wait_s,
            prompts: r.prompts,
            questions: r.questions,
            turns: r.turns,
            tool_calls: r.tool_calls,
            tool_failures: r.tool_failures,
            errors: r.errors,
            rejected: r.rejected,
            cost_microdollars: r.cost_microdollars,
            contexts: r.contexts,
            outcome: r.outcome,
            completion: r.completion,
            terminal_state: r.terminal_state,
            transitions: r.transitions,
            requirements_s: r.requirements_s,
            design_s: r.design_s,
            build_s: r.build_s,
            verify_s: r.verify_s,
            ship_s: r.ship_s,
        })
        .collect())
}
