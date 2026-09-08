//! The profile pane's conversation rollup: how many conversations a user had
//! in the trailing window, split by model, plus the most recent few.
//!
//! Split out of `usage` because it is the one part of that module answering a
//! different question — not "what did this account spend" but "what did it
//! talk about". Every count here reads `conversation_requests`, so probes and
//! utility calls are counted as side calls rather than as conversations, and
//! the legacy sentinel context is already excluded by the view.

use sqlx::PgPool;
use systemprompt::identifiers::{ContextId, UserId};

use super::{CONVERSATION_WINDOW_DAYS, ConversationGroup, ConversationSummary, RecentConversation};

const RECENT_LIMIT: i64 = 10;

// Why: `ai_requests` has no agent column today; the existing analytics surface
// reads agent ids from `plugin_usage_events`, which is keyed differently.
pub async fn get_conversation_summary(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<ConversationSummary, sqlx::Error> {
    let totals = get_conversation_totals(pool, user_id).await?;
    let by_model = list_conversation_by_model(pool, user_id).await?;
    let recent = list_recent_conversations(pool, user_id).await?;

    Ok(ConversationSummary {
        window_days: CONVERSATION_WINDOW_DAYS,
        total_conversations: totals.conversations,
        total_ai_requests: totals.ai_requests,
        side_call_count: totals.side_calls,
        by_model,
        by_agent: Vec::new(),
        latest: recent.first().cloned(),
        recent,
    })
}

struct Totals {
    conversations: i64,
    ai_requests: i64,
    side_calls: i64,
}

async fn get_conversation_totals(pool: &PgPool, user_id: &UserId) -> Result<Totals, sqlx::Error> {
    // Why: `ai_requests` counts every request, sentinel context included; the
    // view already drops the sentinel, which is right for conversations and
    // side calls but would under-report what the account actually sent.
    let row = sqlx::query!(
        r#"SELECT
            (SELECT COUNT(DISTINCT context_id) FILTER (WHERE effective_kind = 'turn')::bigint
               FROM conversation_requests
              WHERE user_id = $1
                AND created_at >= NOW() - make_interval(days => $2)) AS "conversations!",
            (SELECT COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint
               FROM conversation_requests
              WHERE user_id = $1
                AND created_at >= NOW() - make_interval(days => $2)) AS "side_calls!",
            COUNT(*)::bigint AS "ai_requests!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - make_interval(days => $2)"#,
        user_id.as_str(),
        CONVERSATION_WINDOW_DAYS,
    )
    .fetch_one(pool)
    .await?;
    Ok(Totals {
        conversations: row.conversations,
        ai_requests: row.ai_requests,
        side_calls: row.side_calls,
    })
}

async fn list_conversation_by_model(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<ConversationGroup>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT
            COALESCE(model, 'unrouted') AS "model!",
            COUNT(DISTINCT context_id)::bigint AS "conversations!",
            COUNT(*)::bigint AS "ai_requests!"
          FROM conversation_requests
          WHERE user_id = $1
            AND effective_kind = 'turn'
            AND created_at >= NOW() - make_interval(days => $2)
          GROUP BY COALESCE(model, 'unrouted')
          ORDER BY COUNT(*) DESC
          LIMIT 5"#,
        user_id.as_str(),
        CONVERSATION_WINDOW_DAYS,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| ConversationGroup {
        name: r.model,
        conversations: r.conversations,
        ai_requests: r.ai_requests,
    })
    .collect())
}

async fn list_recent_conversations(
    pool: &PgPool,
    user_id: &UserId,
) -> Result<Vec<RecentConversation>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT
            r.context_id            AS "context_id!: ContextId",
            uc.name                 AS "context_name?",
            conversation_title(r.context_id, r.client_session_id) AS "title!",
            r.last_at               AS "last_at!",
            (r.turn_count + r.side_call_count)::bigint AS "ai_requests!",
            r.turn_count            AS "turn_count!",
            r.side_call_count       AS "side_call_count!",
            r.total_cost_microdollars AS "cost_microdollars!",
            r.model                 AS "model?"
          FROM conversation_metrics_for(ARRAY(
              SELECT DISTINCT context_id::text FROM ai_requests WHERE user_id = $1
          )) r
          LEFT JOIN user_contexts uc ON uc.context_id = r.context_id
          WHERE r.user_id = $1
            AND r.turn_count > 0
            AND r.last_at >= NOW() - make_interval(days => $2)
          ORDER BY r.last_at DESC
          LIMIT $3"#,
        user_id.as_str(),
        CONVERSATION_WINDOW_DAYS,
        RECENT_LIMIT,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| RecentConversation {
        context_id: r.context_id,
        context_name: r.context_name,
        title: r.title,
        last_activity: r.last_at,
        ai_requests: r.ai_requests,
        turn_count: r.turn_count,
        side_call_count: r.side_call_count,
        cost_microdollars: r.cost_microdollars,
        model: r.model,
        agent_name: None,
    })
    .collect())
}
