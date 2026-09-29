//! The conversations that invoked one skill, and each marketplace's adoption
//! record — installs by host, active people, spend — for the Skills page.

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, MarketplaceId, UserId};

use super::SkillWindow;

/// One conversation that invoked the skill.
#[derive(Debug, Clone)]
pub struct SkillConversationRow {
    pub context_id: ContextId,
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub client_session_id: Option<String>,
    pub group_name: Option<String>,
    pub project_name: Option<String>,
    pub client_kind: String,
    pub model: Option<String>,
    pub models: Vec<String>,
    pub turn_count: i64,
    pub tool_calls: i64,
    pub tool_calls_failed: i64,
    pub artifacts: i64,
    pub error_count: i64,
    pub gov_deny: i64,
    pub total_tokens: i64,
    pub cache_tokens: i64,
    pub cost_microdollars: i64,
    pub p95_latency_ms: Option<i32>,
    pub duration_seconds: i64,
    pub skills: Vec<String>,
    pub first_at: DateTime<Utc>,
    pub last_at: DateTime<Utc>,
    pub invocations: i64,
    pub judge_title: Option<String>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub completion: Option<i16>,
    pub summary: Option<String>,
    pub title: String,
    pub total: i64,
}

pub async fn list_skill_conversations_paged(
    pool: &PgPool,
    window: &SkillWindow,
    skill: &str,
    limit: i64,
    offset: i64,
) -> Result<(Vec<SkillConversationRow>, i64), sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/conversations.sql",
        window.start,
        window.end,
        skill,
        window.subject_ids.as_deref(),
        limit.clamp(1, 500),
        offset.max(0),
    )
    .fetch_all(pool)
    .await?;
    let total = rows.first().map_or(0, |r| r.total);
    let rows = rows
        .into_iter()
        .map(|r| SkillConversationRow {
            context_id: r.context_id,
            user_id: r.user_id,
            display_name: r.display_name,
            client_session_id: r.client_session_id,
            group_name: r.group_name,
            project_name: r.project_name,
            client_kind: r.client_kind,
            model: r.model,
            models: r.models,
            turn_count: r.turn_count,
            tool_calls: r.tool_calls,
            tool_calls_failed: r.tool_calls_failed,
            artifacts: r.artifacts,
            error_count: r.error_count,
            gov_deny: r.gov_deny,
            total_tokens: r.total_tokens,
            cache_tokens: r.cache_tokens,
            cost_microdollars: r.cost_microdollars,
            p95_latency_ms: r.p95_latency_ms,
            duration_seconds: r.duration_seconds,
            skills: r.skills,
            first_at: r.first_at,
            last_at: r.last_at,
            invocations: r.invocations,
            judge_title: r.judge_title,
            category: r.category,
            outcome: r.outcome,
            completion: r.completion,
            summary: r.summary,
            title: r.title,
            total: r.total,
        })
        .collect();
    Ok((rows, total))
}

/// One marketplace's adoption record: installs by host, activity, spend.
#[derive(Debug, Clone)]
pub struct MarketplaceAdoptionRow {
    pub marketplace_id: MarketplaceId,
    pub skills: i64,
    pub plugins: i64,
    pub installed: i64,
    // Why: entitlement is resolved in Rust; the ids let the page count the
    // installed who are entitled, so a rate never exceeds its denominator.
    pub installed_consumers: Vec<String>,
    pub installed_claude_code: i64,
    pub installed_opencode: i64,
    pub installed_other: i64,
    pub last_install_at: Option<DateTime<Utc>>,
    pub invocations: i64,
    pub active_users: i64,
    pub sessions: i64,
    pub skills_used: i64,
    pub conversations: i64,
    pub cost_microdollars: i64,
    pub tokens: i64,
    pub completion_avg: Option<f64>,
    pub current_hash: Option<String>,
    pub versions: i64,
}

pub async fn list_marketplace_adoption(
    pool: &PgPool,
    window: &SkillWindow,
) -> Result<Vec<MarketplaceAdoptionRow>, sqlx::Error> {
    let rows = sqlx::query_file!(
        "src/repositories/analysis/skills/adoption.sql",
        window.start,
        window.end,
        window.subject_ids.as_deref(),
    )
    .fetch_all(pool)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| MarketplaceAdoptionRow {
            marketplace_id: r.marketplace_id,
            skills: r.skills,
            plugins: r.plugins,
            installed: r.installed,
            installed_consumers: r.installed_consumers,
            installed_claude_code: r.installed_claude_code,
            installed_opencode: r.installed_opencode,
            installed_other: r.installed_other,
            last_install_at: r.last_install_at,
            invocations: r.invocations,
            active_users: r.active_users,
            sessions: r.sessions,
            skills_used: r.skills_used,
            conversations: r.conversations,
            cost_microdollars: r.cost_microdollars,
            tokens: r.tokens,
            completion_avg: r.completion_avg,
            current_hash: r.current_hash,
            versions: r.versions,
        })
        .collect())
}
