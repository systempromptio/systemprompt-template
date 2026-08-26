//! Identity-scoped conversation-history listing with ranked full-text search.
//!
//! Backs the `/admin/history` page and its JSON search endpoint. Search goes
//! through the generated `session_transcripts.search_tsv` column and its GIN
//! index (`websearch_to_tsquery` + `ts_rank`), not the unindexed ILIKE path
//! the admin conversations page uses. Scope is a caller-supplied user-id
//! allowlist; `None` means unrestricted (admin/auditor viewers).

use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::PgPool;
use systemprompt::identifiers::{SessionId, UserId};

/// One conversation in the history list. `snippet` is a `ts_headline`
/// fragment (plain `[`/`]` markers, no HTML) present only when a search
/// query was given; callers redact it before it reaches a browser.
#[derive(Debug, Clone, Serialize)]
pub struct HistoryListItem {
    pub session_id: SessionId,
    pub user_id: UserId,
    pub ai_title: Option<String>,
    pub model: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub captured_at: DateTime<Utc>,
    pub entries_counted: i32,
    pub total_input_tokens: i64,
    pub total_output_tokens: i64,
    pub rank: Option<f32>,
    pub snippet: Option<String>,
}

pub async fn list_transcripts_matching(
    pool: &PgPool,
    scope_user_ids: Option<&[String]>,
    search: Option<&str>,
    limit: i64,
    offset: i64,
) -> Result<(Vec<HistoryListItem>, i64), sqlx::Error> {
    let query = search.map(str::trim).filter(|q| !q.is_empty());

    let rows = sqlx::query!(
        r#"WITH latest AS (
            SELECT DISTINCT ON (st.session_id)
                st.session_id, st.user_id, st.model, st.transcript,
                st.captured_at,
                COALESCE(st.entries_counted, 0) AS entries_counted,
                COALESCE(st.total_input_tokens, 0) AS total_input_tokens,
                COALESCE(st.total_output_tokens, 0) AS total_output_tokens,
                CASE WHEN $1::text IS NULL THEN NULL
                     ELSE ts_rank(st.search_tsv, websearch_to_tsquery('english', $1))
                END AS rank
            FROM session_transcripts st
            WHERE ($2::text[] IS NULL OR st.user_id = ANY($2))
              AND ($1::text IS NULL
                   OR st.search_tsv @@ websearch_to_tsquery('english', $1))
            ORDER BY st.session_id, st.captured_at DESC
        )
        SELECT
            l.session_id AS "session_id!: SessionId",
            l.user_id AS "user_id!: UserId",
            pss.ai_title,
            l.model,
            pss.started_at,
            l.captured_at AS "captured_at!",
            l.entries_counted AS "entries_counted!",
            l.total_input_tokens AS "total_input_tokens!",
            l.total_output_tokens AS "total_output_tokens!",
            l.rank,
            CASE WHEN $1::text IS NULL THEN NULL
                 ELSE ts_headline('english', left(l.transcript::text, 262144),
                                  websearch_to_tsquery('english', $1),
                                  'StartSel=[, StopSel=], MaxWords=24, MinWords=8, MaxFragments=2')
            END AS snippet,
            (SELECT COUNT(*) FROM latest)::bigint AS "total_count!"
        FROM latest l
        LEFT JOIN plugin_session_summaries pss ON pss.session_id = l.session_id
        ORDER BY l.rank DESC NULLS LAST, l.captured_at DESC
        LIMIT $3 OFFSET $4"#,
        query,
        scope_user_ids,
        limit,
        offset,
    )
    .fetch_all(pool)
    .await?;

    let total = rows.first().map_or(0, |r| r.total_count);
    let items = rows
        .into_iter()
        .map(|r| HistoryListItem {
            session_id: r.session_id,
            user_id: r.user_id,
            ai_title: r.ai_title,
            model: r.model,
            started_at: r.started_at,
            captured_at: r.captured_at,
            entries_counted: r.entries_counted,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            rank: r.rank,
            snippet: r.snippet,
        })
        .collect();
    Ok((items, total))
}
