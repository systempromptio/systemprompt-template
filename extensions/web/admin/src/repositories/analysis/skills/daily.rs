//! One skill's record split by day and by dimension, for the skill page's
//! charts and facets. Both read invocations from the hook plane and
//! everything else from `conversation_facts` through the harness session.

use chrono::NaiveDate;
use sqlx::PgPool;
use systemprompt::identifiers::UserId;

use super::SkillWindow;

/// One day of a skill's record.
#[derive(Debug, Clone, Copy)]
pub struct SkillDayRow {
    pub day: NaiveDate,
    pub invocations: i64,
    pub users: i64,
    pub sessions: i64,
    pub conversations: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub p95_latency_ms: Option<f64>,
    pub completion_avg: Option<f64>,
}

pub async fn list_skill_daily(
    pool: &PgPool,
    window: &SkillWindow,
    skill: &str,
) -> Result<Vec<SkillDayRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/daily.sql",
        window.start,
        window.end,
        skill,
        window.subject_ids.as_deref(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SkillDayRow {
            day: r.day,
            invocations: r.invocations,
            users: r.users,
            sessions: r.sessions,
            conversations: r.conversations,
            tokens: r.tokens,
            cost_microdollars: r.cost_microdollars,
            errors: r.errors,
            p95_latency_ms: r.p95_latency_ms,
            completion_avg: r.completion_avg,
        })
        .collect())
}

/// The dimension a skill's invocations are split by on its detail page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkillBreakdownBy {
    Model,
    Client,
    Group,
    Project,
    User,
    Version,
    Outcome,
}

impl SkillBreakdownBy {
    pub const ALL: [Self; 7] = [
        Self::Model,
        Self::Client,
        Self::Group,
        Self::Project,
        Self::User,
        Self::Version,
        Self::Outcome,
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Model => "model",
            Self::Client => "client",
            Self::Group => "group",
            Self::Project => "project",
            Self::User => "user",
            Self::Version => "version",
            Self::Outcome => "outcome",
        }
    }

    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Model => "Model",
            Self::Client => "Client",
            Self::Group => "Group",
            Self::Project => "Project",
            Self::User => "Person",
            Self::Version => "Marketplace version",
            Self::Outcome => "Outcome",
        }
    }
}

/// One bucket of a skill breakdown.
#[derive(Debug, Clone)]
pub struct SkillBucketRow {
    pub label: String,
    pub user_id: Option<UserId>,
    pub invocations: i64,
    pub users: i64,
    pub conversations: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub p95_latency_ms: Option<f64>,
    pub judged: i64,
    pub completion_avg: Option<f64>,
}

pub async fn list_skill_breakdown(
    pool: &PgPool,
    window: &SkillWindow,
    skill: &str,
    by: SkillBreakdownBy,
) -> Result<Vec<SkillBucketRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/breakdown.sql",
        window.start,
        window.end,
        skill,
        window.subject_ids.as_deref(),
        by.as_str(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| SkillBucketRow {
            label: r.label,
            user_id: r.user_id,
            invocations: r.invocations,
            users: r.users,
            conversations: r.conversations,
            tokens: r.tokens,
            cost_microdollars: r.cost_microdollars,
            errors: r.errors,
            p95_latency_ms: r.p95_latency_ms,
            judged: r.judged,
            completion_avg: r.completion_avg,
        })
        .collect())
}
