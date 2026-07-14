//! Per-user usage aggregations against `ai_requests` for the bridge profile
//! pane.
//!
//! Mirrors the shape of `BridgeProfileUsage` so the SSR profile page and the
//! `/v1/bridge/profile/usage` API endpoint render the same data.

use chrono::{DateTime, Utc};
use sqlx::PgPool;

#[derive(Debug, Clone, Copy, Default)]
pub struct UsageWindow {
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub previous_cost_microdollars: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct ModelShare {
    pub model: String,
    pub requests: i64,
    pub tokens: i64,
    pub cost_microdollars: i64,
    pub token_share: f64,
}

#[derive(Debug, Clone)]
pub struct ConversationGroup {
    pub name: String,
    pub conversations: i64,
    pub ai_requests: i64,
}

#[derive(Debug, Clone)]
pub struct RecentConversation {
    pub context_id: String,
    pub context_name: Option<String>,
    pub last_activity: DateTime<Utc>,
    pub ai_requests: i64,
    pub model: Option<String>,
    pub agent_name: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ConversationSummary {
    pub total_conversations: i64,
    pub total_ai_requests: i64,
    pub by_model: Vec<ConversationGroup>,
    pub by_agent: Vec<ConversationGroup>,
    pub recent: Vec<RecentConversation>,
}

/// Sum requests, tokens, and cost over a recent window for one user.
///
/// `window_days` is the trailing window; `previous` covers the equivalent prior
/// window so the caller can compute a delta.
pub async fn fetch_usage_window(
    pool: &PgPool,
    user_id: &str,
    window_days: i32,
) -> Result<UsageWindow, sqlx::Error> {
    let curr = sqlx::query!(
        r#"SELECT
            COUNT(*)::bigint AS "requests!",
            COALESCE(SUM(tokens_used), 0)::bigint AS "tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - make_interval(days => $2)"#,
        user_id,
        window_days,
    )
    .fetch_one(pool)
    .await?;

    let prev = sqlx::query!(
        r#"SELECT COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
           FROM ai_requests
           WHERE user_id = $1
             AND created_at >= NOW() - make_interval(days => $2 * 2)
             AND created_at <  NOW() - make_interval(days => $2)"#,
        user_id,
        window_days,
    )
    .fetch_one(pool)
    .await?;

    Ok(UsageWindow {
        requests: curr.requests,
        tokens: curr.tokens,
        cost_microdollars: curr.cost,
        previous_cost_microdollars: Some(prev.cost),
    })
}

/// Top-N models by tokens for one user over the last 30 days.
///
/// `token_share` is computed against the 30-day total and may be 0.0 when the
/// user has no activity.
pub async fn fetch_top_models(
    pool: &PgPool,
    user_id: &str,
    limit: i64,
) -> Result<Vec<ModelShare>, sqlx::Error> {
    let total = sqlx::query!(
        r#"SELECT COALESCE(SUM(tokens_used), 0)::bigint AS "tokens!"
           FROM ai_requests
           WHERE user_id = $1
             AND created_at >= NOW() - INTERVAL '30 days'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?
    .tokens;

    let rows = sqlx::query!(
        r#"SELECT
            model AS "model!",
            COUNT(*)::bigint AS "requests!",
            COALESCE(SUM(tokens_used), 0)::bigint AS "tokens!",
            COALESCE(SUM(cost_microdollars), 0)::bigint AS "cost!"
          FROM ai_requests
          WHERE user_id = $1
            AND created_at >= NOW() - INTERVAL '30 days'
          GROUP BY model
          ORDER BY SUM(tokens_used) DESC NULLS LAST
          LIMIT $2"#,
        user_id,
        limit,
    )
    .fetch_all(pool)
    .await?;

    let total_f = total as f64;
    Ok(rows
        .into_iter()
        .map(|r| ModelShare {
            model: r.model,
            requests: r.requests,
            tokens: r.tokens,
            cost_microdollars: r.cost,
            token_share: if total_f > 0.0 {
                r.tokens as f64 / total_f
            } else {
                0.0
            },
        })
        .collect())
}

/// Conversation summary for the last 30 days: totals, by-model breakdown,
/// recent conversations.
///
/// `by_agent` is left empty when no agent label is recorded against requests —
/// `ai_requests` has no agent column today and the existing analytics surface
/// reads agent ids from `plugin_usage_events`, which is keyed differently.
pub async fn fetch_conversation_summary(
    pool: &PgPool,
    user_id: &str,
) -> Result<ConversationSummary, sqlx::Error> {
    let (total_conversations, total_ai_requests) = fetch_conversation_totals(pool, user_id).await?;
    let by_model = fetch_conversation_by_model(pool, user_id).await?;
    let recent = fetch_recent_conversations(pool, user_id).await?;

    Ok(ConversationSummary {
        total_conversations,
        total_ai_requests,
        by_model,
        by_agent: Vec::new(),
        recent,
    })
}

async fn fetch_conversation_totals(
    pool: &PgPool,
    user_id: &str,
) -> Result<(i64, i64), sqlx::Error> {
    let totals = sqlx::query!(
        r#"SELECT
            COUNT(DISTINCT context_id)::bigint AS "total_conversations!",
            COUNT(*)::bigint AS "total_ai_requests!"
          FROM ai_requests
          WHERE user_id = $1
            AND context_id IS NOT NULL
            AND created_at >= NOW() - INTERVAL '30 days'"#,
        user_id,
    )
    .fetch_one(pool)
    .await?;
    Ok((totals.total_conversations, totals.total_ai_requests))
}

async fn fetch_conversation_by_model(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<ConversationGroup>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"SELECT
            model AS "model!",
            COUNT(DISTINCT context_id)::bigint AS "conversations!",
            COUNT(*)::bigint AS "ai_requests!"
          FROM ai_requests
          WHERE user_id = $1
            AND context_id IS NOT NULL
            AND created_at >= NOW() - INTERVAL '30 days'
          GROUP BY model
          ORDER BY COUNT(*) DESC
          LIMIT 5"#,
        user_id,
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

async fn fetch_recent_conversations(
    pool: &PgPool,
    user_id: &str,
) -> Result<Vec<RecentConversation>, sqlx::Error> {
    Ok(sqlx::query!(
        r#"WITH ranked AS (
            SELECT
              context_id,
              MAX(created_at) AS last_activity,
              COUNT(*)::bigint AS ai_requests,
              MAX(model) AS model
            FROM ai_requests
            WHERE user_id = $1
              AND context_id IS NOT NULL
              AND created_at >= NOW() - INTERVAL '30 days'
            GROUP BY context_id
          )
          SELECT
            ranked.context_id AS "context_id!",
            uc.name           AS "context_name?",
            ranked.last_activity AS "last_activity!",
            ranked.ai_requests AS "ai_requests!",
            ranked.model
          FROM ranked
          LEFT JOIN user_contexts uc ON uc.context_id = ranked.context_id
          ORDER BY ranked.last_activity DESC
          LIMIT 5"#,
        user_id,
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .map(|r| RecentConversation {
        context_id: r.context_id,
        context_name: r.context_name,
        last_activity: r.last_activity,
        ai_requests: r.ai_requests,
        model: r.model,
        agent_name: None,
    })
    .collect())
}
