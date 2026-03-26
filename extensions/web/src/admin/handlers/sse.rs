use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;

use axum::{
    extract::{Extension, State},
    response::sse::{Event, KeepAlive, Sse},
};
use sqlx::PgPool;
use tokio_stream::Stream;

use crate::admin::activity;
use crate::admin::types::{ActivityStats, UserContext};

pub(crate) async fn dashboard_sse(
    Extension(_user_ctx): Extension<UserContext>,
    State(pool): State<Arc<PgPool>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = async_stream::stream! {
        let mut last_id: Option<String> = None;
        let mut interval = tokio::time::interval(Duration::from_secs(15));

        loop {
            interval.tick().await;

            if let Ok(events) = activity::queries::fetch_new_events(&pool, last_id.as_deref()).await {
                if !events.is_empty() {
                    last_id = Some(events[0].id.clone());
                    if let Ok(json) = serde_json::to_string(&events) {
                        yield Ok(Event::default().event("activity").data(json));
                    }
                }
            }

            if let Ok(stats) = fetch_stats_snapshot(&pool).await {
                if let Ok(json) = serde_json::to_string(&stats) {
                    yield Ok(Event::default().event("stats").data(json));
                }
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

async fn fetch_stats_snapshot(pool: &PgPool) -> Result<ActivityStats, sqlx::Error> {
    sqlx::query_as::<_, ActivityStats>(
        r"SELECT
            COALESCE(COUNT(*) FILTER (WHERE p.created_at >= CURRENT_DATE), 0)::BIGINT AS events_today,
            COALESCE(COUNT(*) FILTER (WHERE p.created_at >= DATE_TRUNC('week', CURRENT_DATE)), 0)::BIGINT AS events_this_week,
            COALESCE(COUNT(DISTINCT p.session_id), 0)::BIGINT AS total_sessions,
            COALESCE(COUNT(*) FILTER (WHERE p.event_type ILIKE '%error%' OR p.event_type ILIKE '%fail%'), 0)::BIGINT AS error_count,
            COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUse'), 0)::BIGINT AS tool_uses,
            COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_UserPromptSubmit'), 0)::BIGINT AS prompts,
            COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_SubagentStart'), 0)::BIGINT AS subagents_spawned,
            COALESCE((SELECT SUM(total_input_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_input_tokens,
            COALESCE((SELECT SUM(total_output_tokens) FROM plugin_session_summaries), 0)::BIGINT AS total_output_tokens,
            COALESCE((SELECT SUM((p2.metadata->>'total_cost_usd')::NUMERIC) FROM plugin_usage_events p2 WHERE p2.event_type = 'claude_code_StatusLine'), 0.0)::FLOAT8 AS total_cost_usd,
            COALESCE(COUNT(*) FILTER (WHERE p.event_type = 'claude_code_PostToolUseFailure'), 0)::BIGINT AS failure_count
        FROM plugin_usage_events p
        JOIN users u ON u.id = p.user_id
        WHERE NOT ('anonymous' = ANY(u.roles))
          AND u.email NOT LIKE '%@anonymous.local'",
    )
    .fetch_one(pool)
    .await
}
