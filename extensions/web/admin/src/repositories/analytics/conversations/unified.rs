//! One conversation list over both places a conversation can be recorded.
//!
//! A Claude Code session is captured by the Stop hook into
//! `session_transcripts`; a `/v1/messages` call through the gateway is
//! captured into `ai_requests`, grouped by `context_id`. A user who only ever
//! talks to the gateway has zero transcript rows, which is why
//! `/admin/history` used to show them nothing at all while their traffic was
//! plainly visible to an admin on the contexts list.
//!
//! Both halves are read in one statement so paging and the total count are
//! computed over the union rather than per source. Ranked search runs against
//! the transcript `search_tsv` index as before; the gateway half has no
//! full-text index, so it matches by `ILIKE` over the context name, the
//! opening prompt, the id, and the model, and always ranks below a scored
//! transcript hit.
//!
//! A session that has both a Stop-hook transcript and gateway rows appears
//! twice, once per source. That is deliberate: the two records are captured by
//! different mechanisms and hold different fields, and collapsing them would
//! mean silently choosing one of them to show.
//!
//! The gateway half reads `conversation_requests`, so a turn is a request
//! whose `effective_kind` is `turn`; probes and utility calls are counted as
//! side calls and a conversation made of side calls alone is listed only when
//! the caller asks for them.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use sqlx::types::Json;
use systemprompt::identifiers::{ContextId, SessionId, UserId};

/// Where a conversation was recorded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HistorySource {
    Transcript,
    Gateway,
}

impl HistorySource {
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Transcript => "Claude Code",
            Self::Gateway => "Gateway",
        }
    }
}

/// One conversation in the unified history list. Exactly one of `session_id`
/// and `context_id` is set, according to `source`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryItem<Id = ContextId> {
    pub source: HistorySource,
    pub session_id: Option<SessionId>,
    pub context_id: Option<Id>,
    pub user_id: UserId,
    pub title: Option<String>,
    pub preview: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub last_at: DateTime<Utc>,
    pub turns: i64,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub cost_microdollars: i64,
    pub side_call_count: i64,
    pub rank: Option<f32>,
    pub snippet: Option<String>,
}

#[expect(
    clippy::too_many_arguments,
    reason = "every argument is one bound parameter of a compile-time-checked query"
)]
pub async fn list_history_items(
    pool: &PgPool,
    scope_user_ids: Option<&[String]>,
    search: Option<&str>,
    include_side_calls: bool,
    limit: i64,
    offset: i64,
) -> Result<(Vec<HistoryItem>, i64), sqlx::Error> {
    let query = search.map(str::trim).filter(|q| !q.is_empty());
    let pattern = query.map(|q| format!("%{}%", q.replace('\\', "\\\\").replace('%', "\\%")));
    let legacy = ContextId::legacy();

    let mut transaction = crate::repositories::dashboard_read::begin(pool).await?;
    let row = sqlx::query_file!(
        "src/repositories/analytics/conversations/history_page.sql",
        query,
        scope_user_ids,
        limit,
        offset,
        legacy.as_str(),
        pattern,
        include_side_calls,
    )
    .fetch_one(&mut *transaction)
    .await?;

    transaction.commit().await?;
    let items = row
        .payload
        .0
        .items
        .into_iter()
        .map(|r| {
            Ok(HistoryItem {
                source: r.source,
                session_id: r.session_id,
                context_id: r
                    .context_id
                    .map(|id| {
                        crate::repositories::dashboard_read::context_id(&id, &row.context_ids)
                    })
                    .transpose()?,
                user_id: r.user_id,
                title: r.title,
                preview: r.preview,
                model: r.model,
                started_at: r.started_at,
                last_at: r.last_at,
                turns: r.turns,
                total_input_tokens: r.total_input_tokens,
                total_output_tokens: r.total_output_tokens,
                cost_microdollars: r.cost_microdollars,
                side_call_count: r.side_call_count,
                rank: r.rank,
                snippet: r.snippet,
            })
        })
        .collect::<Result<_, sqlx::Error>>()?;
    Ok((items, row.payload.0.total))
}

#[derive(Debug, Deserialize)]
struct HistoryPageResult {
    items: Vec<HistoryItem<String>>,
    total: i64,
}
