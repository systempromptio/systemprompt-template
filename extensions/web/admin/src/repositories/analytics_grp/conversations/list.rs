//! Left-pane session list query — sessions filtered by time-range, identity,
//! and free-text against title / summary / transcript body.

use sqlx::PgPool;

use systemprompt::identifiers::{SessionId, UserId};

use super::{ConversationListFilter, ConversationListItem};

pub async fn fetch_conversation_list(
    pool: &PgPool,
    filter: &ConversationListFilter,
) -> Result<Vec<ConversationListItem>, sqlx::Error> {
    let limit = if filter.limit > 0 && filter.limit <= 500 {
        filter.limit
    } else {
        100
    };
    let free_text_pattern = filter
        .free_text
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| format!("%{}%", s.replace('\\', "\\\\").replace('%', "\\%")));

    let rows = sqlx::query!(
        r#"
        SELECT s.session_id    AS "session_id!: SessionId",
               s.user_id       AS "user_id!: UserId",
               s.plugin_id,
               s.model,
               s.status,
               s.ai_title,
               s.started_at,
               COALESCE(s.total_input_tokens, 0)::bigint  AS "total_input_tokens!",
               COALESCE(s.total_output_tokens, 0)::bigint AS "total_output_tokens!",
               COALESCE(g.intervention_count, 0)::bigint  AS "governance_intervention_count!",
               COALESCE(g.deny_count, 0)::bigint          AS "deny_count!"
        FROM plugin_session_summaries s
        LEFT JOIN (
            SELECT session_id,
                   COUNT(*)                                            AS intervention_count,
                   COUNT(*) FILTER (WHERE decision = 'deny')           AS deny_count
            FROM governance_decisions
            GROUP BY session_id
        ) g ON g.session_id = s.session_id
        LEFT JOIN session_transcripts t ON t.session_id = s.session_id
        WHERE ($1::text IS NULL OR s.user_id   = $1)
          AND ($2::text IS NULL OR s.plugin_id = $2)
          AND ($3::timestamptz IS NULL OR s.started_at >= $3)
          AND ($4::timestamptz IS NULL OR s.started_at <  $4)
          AND ($5::text IS NULL
               OR s.ai_title ILIKE $5
               OR s.ai_summary ILIKE $5
               OR t.transcript::text ILIKE $5)
        ORDER BY s.started_at DESC NULLS LAST
        LIMIT $6
        "#,
        filter.user_id.as_ref().map(UserId::as_str),
        filter.plugin_id,
        filter.since,
        filter.until,
        free_text_pattern,
        limit,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| ConversationListItem {
            session_id: r.session_id,
            user_id: r.user_id,
            plugin_id: r.plugin_id,
            model: r.model,
            status: r.status,
            ai_title: r.ai_title,
            started_at: r.started_at,
            total_input_tokens: r.total_input_tokens,
            total_output_tokens: r.total_output_tokens,
            governance_intervention_count: r.governance_intervention_count,
            deny_count: r.deny_count,
        })
        .collect())
}
