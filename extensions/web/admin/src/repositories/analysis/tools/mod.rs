//! Tool calls and artifacts — the repository behind `/admin/tools` and
//! `/admin/artifacts`.
//!
//! Both pages read `tool_activity` (schema 46): every ledger row with the
//! one artifact rule applied, so a tool call is counted the same way on the
//! Tools page, the Artifacts page and the conversation record. One statement
//! (`page.sql`) narrows the view, then reads its totals, its time series
//! padded to every bucket of the window, one breakdown dimension, the facet
//! lists and one page of rows from the same filtered set. The Artifacts page
//! is the same statement with `artifacts_only` set.

mod filter;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::UserId;

pub use filter::{ArtifactKind, ToolBreakdownBy, ToolSort, ToolState};

/// Narrowing applied to the rows, totals, series and breakdown alike.
#[derive(Debug, Clone, Default)]
pub struct ToolActivityFilter {
    pub since: Option<DateTime<Utc>>,
    pub until: Option<DateTime<Utc>>,
    // Why: the caller's resolved `SubjectScope::as_sql()`; `None` = every user.
    pub subject_ids: Option<Vec<String>>,
    pub user_id: Option<UserId>,
    pub tool: Option<String>,
    pub server: Option<String>,
    // Why: `Some(true)` = builtin harness tools only, `Some(false)` = MCP only.
    pub builtin: Option<bool>,
    pub state: Option<ToolState>,
    pub decision: Option<String>,
    pub context: Option<String>,
    pub session: Option<String>,
    pub skill: Option<String>,
    pub client_kind: Option<String>,
    pub artifact_kind: Option<ArtifactKind>,
    pub artifacts_only: bool,
    pub free_text: Option<String>,
    // Why: row keys ticked on the page; an export of a selection.
    pub ids: Option<Vec<String>>,
}

impl ToolActivityFilter {
    fn free_text_pattern(&self) -> Option<String> {
        self.free_text
            .as_ref()
            .filter(|s| !s.is_empty())
            .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")))
    }

    // Why: a window of two days or less buckets by hour; longer by day.
    fn bucket(&self) -> &'static str {
        match (self.since, self.until) {
            (Some(since), Some(until)) if (until - since).num_hours() <= 48 => "hour",
            _ => "day",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ToolActivityPage {
    pub sort: ToolSort,
    pub descending: bool,
    pub limit: i64,
    pub offset: i64,
    pub breakdown: ToolBreakdownBy,
}

/// One tool call as `tool_activity` shows it, with the owner's name, the
/// decision keyed to the call and the skill last invoked before it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolActivityRow {
    pub row_key: String,
    pub ai_tool_call_id: Option<String>,
    pub intent_id: Option<String>,
    pub request_id: Option<String>,
    pub mcp_execution_id: Option<String>,
    // Why: the view's id columns arrive as opaque link keys — the pages only
    // ever put them in a URL and never construct a typed id from them.
    #[serde(rename = "artifact_id")]
    pub artifact_key: Option<String>,
    pub user_id: Option<UserId>,
    pub display_name: Option<String>,
    #[serde(rename = "session_id")]
    pub session_key: Option<String>,
    #[serde(rename = "context_id")]
    pub context_key: Option<String>,
    #[serde(rename = "execution_context_id")]
    pub execution_context_key: Option<String>,
    pub execution_trace_id: Option<String>,
    pub client_kind: Option<String>,
    pub tool_name: Option<String>,
    pub server_name: Option<String>,
    pub intended_at: Option<DateTime<Utc>>,
    pub executed_at: Option<DateTime<Utc>>,
    pub execution_time_ms: Option<i32>,
    pub execution_status: Option<String>,
    pub error_message: Option<String>,
    pub source: Option<String>,
    pub correlation: Option<String>,
    pub artifact_type: Option<String>,
    pub artifact_title: Option<String>,
    #[serde(default)]
    pub is_structured: bool,
    #[serde(default)]
    pub has_ui_resource: bool,
    #[serde(default)]
    pub is_error: bool,
    pub payload_bytes: Option<i32>,
    pub secret_redactions: Option<i32>,
    pub state: String,
    pub occurred_at: Option<DateTime<Utc>>,
    pub artifact_kind: Option<String>,
    pub input_summary: Option<String>,
    #[serde(default)]
    pub is_builtin: bool,
    pub decision: Option<String>,
    pub decision_id: Option<String>,
    pub skill: Option<String>,
    #[serde(default)]
    pub failed: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ToolActivityTotals {
    pub calls: i64,
    pub executed: i64,
    pub failed: i64,
    pub denied: i64,
    pub warned: i64,
    pub intended: i64,
    pub unattested: i64,
    pub builtin: i64,
    pub tools: i64,
    pub servers: i64,
    pub users: i64,
    pub conversations: i64,
    pub p95_duration_ms: Option<f64>,
    pub artifacts: i64,
    pub files: i64,
    pub cards: i64,
    pub ui: i64,
    pub bodies: i64,
    pub artifact_errors: i64,
    pub bytes: i64,
    pub redactions: i64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ToolSeriesPoint {
    pub bucket: DateTime<Utc>,
    pub calls: i64,
    pub executed: i64,
    pub failed: i64,
    pub denied: i64,
    pub files: i64,
    pub cards: i64,
    pub ui: i64,
    pub bodies: i64,
    pub artifacts: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolBucketRow {
    pub label: String,
    pub value: Option<String>,
    pub calls: i64,
    pub executed: i64,
    pub failed: i64,
    pub denied: i64,
    pub users: i64,
    pub artifacts: i64,
    pub p95_duration_ms: Option<f64>,
    pub bytes: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFacet {
    pub value: String,
    #[serde(default)]
    pub label: Option<String>,
    pub calls: i64,
}

#[derive(Debug, Default, Deserialize)]
pub struct ToolActivityResult {
    pub rows: Vec<ToolActivityRow>,
    pub totals: ToolActivityTotals,
    pub series: Vec<ToolSeriesPoint>,
    pub breakdown: Vec<ToolBucketRow>,
    pub tools: Vec<ToolFacet>,
    pub servers: Vec<ToolFacet>,
    pub users: Vec<ToolFacet>,
    pub clients: Vec<ToolFacet>,
    pub skills: Vec<ToolFacet>,
    pub kinds: Vec<ToolFacet>,
    pub decisions: Vec<ToolFacet>,
}

pub async fn load_tool_activity_page(
    pool: &PgPool,
    filter: &ToolActivityFilter,
    page: ToolActivityPage,
) -> Result<ToolActivityResult, sqlx::Error> {
    let mut connection = crate::repositories::dashboard_read::begin(pool).await?;
    let pattern = filter.free_text_pattern();
    let result = sqlx::query_file!(
        "src/repositories/analysis/tools/page.sql",
        filter.since,
        filter.until,
        filter.subject_ids.as_deref(),
        filter.user_id.as_ref().map(UserId::as_str),
        filter.tool,
        filter.server,
        filter.builtin,
        filter.state.map(ToolState::as_str),
        filter.decision,
        filter.context,
        filter.session,
        filter.skill,
        filter.artifact_kind.map(ArtifactKind::as_str),
        filter.artifacts_only,
        pattern,
        filter.ids.as_deref(),
        page.sort.as_str(),
        page.descending,
        page.limit,
        page.offset,
        page.breakdown.as_str(),
        filter.bucket(),
        filter.client_kind,
    )
    .fetch_one(&mut *connection)
    .await?;
    connection.commit().await?;
    Ok(result.payload.0)
}
