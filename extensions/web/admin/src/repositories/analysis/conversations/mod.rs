//! Conversations — the repository behind `/admin/analysis/conversations`.
//!
//! A conversation is a gateway context; `conversation_facts` (schema 45) is
//! its deterministic record, rolled up by the `conversation_rollup` job from
//! the request log, the hook plane, the tool ledger and the governance spine,
//! and `conversation_analyses` (schema 37) is the judge's one label on top.
//! One statement (`page.sql`) narrows the fact table, then reads its totals,
//! its time series, one breakdown dimension, the facet lists and one page of
//! rows from the same filtered set, so every number above the table
//! describes the rows inside it. Filtering, paging and sorting are bound
//! parameters chosen by `CASE` arms, never interpolated text.

pub mod detail;
mod filter;
pub mod hook_events;
pub mod planes;
mod row;

use row::decode_row;
pub use row::{ContinuationLink, ConversationFactRow};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContextId, UserId};
use systemprompt_web_shared::{GroupId, ProjectId};

pub use filter::{BreakdownBy, FactSort, FlagFilter, JudgedFilter};

/// Narrowing applied to the rows, totals, series and breakdown alike.
#[derive(Debug, Clone, Default)]
pub struct ConversationAnalysisFilter {
    // Why: the caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
    pub user_id: Option<UserId>,
    pub category: Option<String>,
    pub outcome: Option<String>,
    pub skill: Option<String>,
    pub free_text: Option<String>,
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    pub judged: Option<JudgedFilter>,
    pub model: Option<String>,
    pub client_kind: Option<String>,
    pub group_id: Option<GroupId>,
    pub project_id: Option<ProjectId>,
    pub flag: Option<FlagFilter>,
    // Why: an explicit row selection — the export of ticked rows and the
    // judge-selected action read exactly these contexts, window aside.
    pub context_ids: Option<Vec<String>>,
    // Why: a conversation with no turn is a title or side call the client
    // made on its own; the page hides those unless asked for every row.
    pub include_without_turns: bool,
}

impl ConversationAnalysisFilter {
    fn free_text_pattern(&self) -> Option<String> {
        self.free_text
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
    }

    // Why: a 24-hour window buckets by hour so the chart has a shape; every
    // longer window buckets by day.
    fn bucket(&self) -> &'static str {
        match self.since {
            Some(since) if (Utc::now() - since).num_hours() <= 48 => "hour",
            _ => "day",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ConversationAnalysisPage {
    pub sort: FactSort,
    pub descending: bool,
    pub limit: i64,
    pub offset: i64,
    pub breakdown: BreakdownBy,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ConversationAnalysisTotals {
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub requests: i64,
    pub tool_calls: i64,
    pub tool_calls_executed: i64,
    pub tool_calls_failed: i64,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_tokens: i64,
    pub reasoning_tokens: i64,
    pub total_cost_microdollars: i64,
    pub errors: i64,
    pub rejected: i64,
    pub denied: i64,
    pub warned: i64,
    pub safety_findings: i64,
    pub safety_blocked: i64,
    pub artifacts: i64,
    #[serde(default)]
    pub artifact_files: i64,
    #[serde(default)]
    pub artifact_cards: i64,
    pub skill_invocations: i64,
    pub with_skills: i64,
    pub achieved: i64,
    pub judged: i64,
    pub completion_avg: Option<f64>,
    pub p95_latency_ms: Option<f64>,
    // Why: queued judge rows in scope — what the label has not yet covered.
    pub pending_judgement: i64,
    // Why: rows in scope left out because they have no turn.
    #[serde(default)]
    pub without_turns: i64,
    #[serde(default)]
    pub active_ms: i64,
}

/// One bucket of the time series: conversations started in the bucket and
/// what they added up to.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ConversationSeriesPoint {
    pub bucket: DateTime<Utc>,
    pub conversations: i64,
    pub turns: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub errors: i64,
    pub tool_calls: i64,
    #[serde(default)]
    pub artifacts: i64,
    #[serde(default)]
    pub denied: i64,
    pub users: i64,
}

/// One bucket of the chosen breakdown dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationBucketRow {
    pub label: String,
    pub user_id: Option<UserId>,
    pub conversations: i64,
    pub users: i64,
    pub turns: i64,
    pub total_tokens: i64,
    pub total_cost_microdollars: i64,
    pub errors: i64,
    pub denied: i64,
    pub tool_calls: i64,
    #[serde(default)]
    pub artifacts: i64,
    pub achieved: i64,
    pub judged: i64,
    pub completion_avg: Option<f64>,
    pub top_category: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillFacet {
    pub skill: String,
    pub conversations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFacet {
    pub user_id: UserId,
    pub display_name: Option<String>,
    pub conversations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelFacet {
    pub model: String,
    pub conversations: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientFacet {
    pub client_kind: String,
    pub conversations: i64,
}

#[derive(Debug, Default)]
pub struct ConversationAnalysisResult {
    pub rows: Vec<ConversationFactRow>,
    pub totals: ConversationAnalysisTotals,
    pub series: Vec<ConversationSeriesPoint>,
    pub breakdown: Vec<ConversationBucketRow>,
    pub skills: Vec<SkillFacet>,
    pub users: Vec<UserFacet>,
    pub models: Vec<ModelFacet>,
    pub clients: Vec<ClientFacet>,
}

// Why: IDs are decoded by SQLx from a companion array; the JSON payload keeps
// them as text so no unchecked constructor is needed on the way out.
#[derive(Debug, Deserialize)]
struct ConversationAnalysisWire {
    rows: Vec<ConversationFactRow<String>>,
    totals: ConversationAnalysisTotals,
    series: Vec<ConversationSeriesPoint>,
    breakdown: Vec<ConversationBucketRow>,
    skills: Vec<SkillFacet>,
    users: Vec<UserFacet>,
    models: Vec<ModelFacet>,
    clients: Vec<ClientFacet>,
}

pub async fn load_conversation_analysis_page(
    pool: &PgPool,
    filter: &ConversationAnalysisFilter,
    page: ConversationAnalysisPage,
) -> Result<ConversationAnalysisResult, sqlx::Error> {
    let started = std::time::Instant::now();
    let mut connection = crate::repositories::dashboard_read::begin(pool).await?;
    let pattern = filter.free_text_pattern();
    let result = sqlx::query_file!(
        "src/repositories/analysis/conversations/page.sql",
        filter.subject_ids.as_deref(),
        filter.user_id.as_ref().map(UserId::as_str),
        filter.category,
        filter.outcome,
        filter.skill,
        pattern,
        filter.since,
        filter.until,
        page.sort.as_str(),
        page.descending,
        page.limit,
        page.offset,
        page.breakdown.as_str(),
        filter.judged.map(JudgedFilter::as_str),
        filter.model,
        filter.client_kind,
        filter.group_id.as_ref().map(GroupId::as_str),
        filter.project_id.as_ref().map(ProjectId::as_str),
        filter.flag.map(FlagFilter::as_str),
        filter.bucket(),
        filter.context_ids.as_deref(),
        filter.include_without_turns,
    )
    .fetch_one(&mut *connection)
    .await?;
    connection.commit().await?;
    tracing::debug!(
        query_ms = started.elapsed().as_secs_f64() * 1000.0,
        "conversation analysis page loaded"
    );
    let ids = result.context_ids;
    let wire = result.payload.0;
    Ok(ConversationAnalysisResult {
        rows: wire
            .rows
            .into_iter()
            .map(|r| decode_row(r, &ids))
            .collect::<Result<_, _>>()?,
        totals: wire.totals,
        series: wire.series,
        breakdown: wire.breakdown,
        skills: wire.skills,
        users: wire.users,
        models: wire.models,
        clients: wire.clients,
    })
}
