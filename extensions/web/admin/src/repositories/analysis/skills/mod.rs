//! Skills — the repository behind `/admin/analysis/skills` and the skill
//! detail page.
//!
//! An invocation is a hook event (`analysis_skill_version_events`, dashed
//! `plugin:skill`); everything else about a skill — the people, the
//! conversations, the tokens and cost, the tools, the errors and denials,
//! the judge's verdict — is read from the `conversation_facts` rows whose
//! harness session invoked it, joined on `client_session_id`. Installs are
//! verified `managed_installation_receipts` on the skill resource. Each
//! query is one bound statement in a `.sql` file beside this module. The
//! list and its row are here; the per-day and per-dimension splits are in
//! `daily`, the conversations and marketplace adoption in `usage`, the
//! timed runs and their workflow phases in `runs`.

mod daily;
mod runs;
mod usage;

use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{MarketplaceId, PluginId};

pub use daily::{
    SkillBreakdownBy, SkillBucketRow, SkillDayRow, list_skill_breakdown, list_skill_daily,
};
pub use runs::{SkillRunFilter, SkillRunRow, list_skill_runs};
pub use usage::{
    MarketplaceAdoptionRow, SkillConversationRow, list_marketplace_adoption,
    list_skill_conversations_paged,
};

/// The window and scope every skills query takes.
#[derive(Debug, Clone)]
pub struct SkillWindow {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    // Why: the caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default)]
pub struct SkillListFilter {
    pub marketplace_id: Option<MarketplaceId>,
    pub search: Option<String>,
    pub client_kind: Option<String>,
    pub sort: SkillSort,
    pub limit: i64,
    pub offset: i64,
    // Why: an explicit row selection (`plugin:skill` keys) from the bulk bar;
    // the export of ticked rows reads exactly these.
    pub skills: Option<Vec<String>>,
}

impl SkillListFilter {
    fn search_pattern(&self) -> Option<String> {
        self.search
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
    }
}

/// The column the skill rows sort by, always descending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkillSort {
    #[default]
    Invocations,
    Users,
    Cost,
    Tokens,
    Errors,
    Completion,
    Recent,
}

impl SkillSort {
    pub const ALL: [(Self, &'static str); 7] = [
        (Self::Invocations, "Most invoked"),
        (Self::Users, "Most people"),
        (Self::Cost, "Highest cost"),
        (Self::Tokens, "Most tokens"),
        (Self::Errors, "Most errors"),
        (Self::Completion, "Best completion"),
        (Self::Recent, "Most recent"),
    ];

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Invocations => "invocations",
            Self::Users => "users",
            Self::Cost => "cost",
            Self::Tokens => "tokens",
            Self::Errors => "errors",
            Self::Completion => "completion",
            Self::Recent => "recent",
        }
    }

    #[must_use]
    pub fn parse_skill_sort(value: Option<&str>) -> Self {
        match value {
            Some("users") => Self::Users,
            Some("cost") => Self::Cost,
            Some("tokens") => Self::Tokens,
            Some("errors") => Self::Errors,
            Some("completion") => Self::Completion,
            Some("recent") => Self::Recent,
            _ => Self::Invocations,
        }
    }
}

/// One skill's record over the window.
#[derive(Debug, Clone)]
pub struct SkillFactRow {
    pub skill: String,
    pub plugin_id: Option<PluginId>,
    pub marketplace_id: Option<MarketplaceId>,
    pub invocations: i64,
    pub slash: i64,
    pub tool: i64,
    pub attributed: i64,
    pub users: i64,
    pub sessions: i64,
    pub first_used: DateTime<Utc>,
    pub last_used: DateTime<Utc>,
    pub conversations: i64,
    pub requests: i64,
    pub turns: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub denied: i64,
    pub tool_calls: i64,
    pub tool_calls_failed: i64,
    pub artifacts: i64,
    pub p95_latency_ms: Option<f64>,
    pub judged: i64,
    pub completion_avg: Option<f64>,
    pub achieved: i64,
    pub models: Vec<String>,
    pub clients: Vec<String>,
    pub installs: i64,
    pub install_hosts: i64,
    pub spark_days: Vec<NaiveDate>,
    pub spark_counts: Vec<i64>,
    pub total: i64,
}

pub async fn list_skill_facts(
    pool: &PgPool,
    window: &SkillWindow,
    filter: &SkillListFilter,
) -> Result<Vec<SkillFactRow>, sqlx::Error> {
    let mut connection = crate::repositories::dashboard_read::begin(pool).await?;
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/page.sql",
        window.start,
        window.end,
        window.subject_ids.as_deref(),
        filter.marketplace_id.as_ref().map(MarketplaceId::as_str),
        filter.search_pattern(),
        filter.client_kind,
        filter.sort.as_str(),
        filter.limit.clamp(1, 1000),
        filter.offset.max(0),
        filter.skills.as_deref(),
    )
    .fetch_all(&mut *connection)
    .await?;
    connection.commit().await?;
    Ok(rows
        .into_iter()
        .map(|r| SkillFactRow {
            skill: r.skill,
            plugin_id: r.plugin_id,
            marketplace_id: r.marketplace_id,
            invocations: r.invocations,
            slash: r.slash,
            tool: r.tool,
            attributed: r.attributed,
            users: r.users,
            sessions: r.sessions,
            first_used: r.first_used,
            last_used: r.last_used,
            conversations: r.conversations,
            requests: r.requests,
            turns: r.turns,
            input_tokens: r.input_tokens,
            output_tokens: r.output_tokens,
            cache_tokens: r.cache_tokens,
            cost_microdollars: r.cost_microdollars,
            errors: r.errors,
            denied: r.denied,
            tool_calls: r.tool_calls,
            tool_calls_failed: r.tool_calls_failed,
            artifacts: r.artifacts,
            p95_latency_ms: r.p95_latency_ms,
            judged: r.judged,
            completion_avg: r.completion_avg,
            achieved: r.achieved,
            models: r.models,
            clients: r.clients,
            installs: r.installs,
            install_hosts: r.install_hosts,
            spark_days: r.spark_days,
            spark_counts: r.spark_counts,
            total: r.total,
        })
        .collect())
}

// Why: one skill's row, or `None` when it was not invoked in the window.
pub async fn find_skill_facts(
    pool: &PgPool,
    window: &SkillWindow,
    skill: &str,
) -> Result<Option<SkillFactRow>, sqlx::Error> {
    let filter = SkillListFilter {
        search: Some(skill.to_owned()),
        limit: 20,
        ..SkillListFilter::default()
    };
    Ok(list_skill_facts(pool, window, &filter)
        .await?
        .into_iter()
        .find(|r| r.skill == skill))
}
