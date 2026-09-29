//! Discovery: which conversations the judge queues, and the operator's
//! re-queue switch. A conversation qualifies once it has a turn, has been
//! quiet for the configured window, and either was never classified or grew
//! past the fingerprint its last verdict recorded. A session the harness
//! reported closed (`SessionEnd`) is queued at once instead of after the quiet
//! window.

use systemprompt::identifiers::ContextId;

use super::repository::ConversationJudgeRepository;
use crate::JobError;

// Why: the sentinel context core pools context-less requests under; never a
// conversation.
const LEGACY_CONTEXT: &str = "00000000-0000-0000-0000-4c4547414359";

#[derive(Debug, Clone, Copy)]
pub(super) struct DiscoveryWindow {
    pub(super) quiet_minutes: i32,
    pub(super) lookback_days: i32,
}

impl ConversationJudgeRepository {
    // Why: queues every quiet conversation with at least one turn that has
    // never been classified, and re-queues classified ones that grew since.
    // Returns how many rows were inserted or flipped back to pending.
    pub(super) async fn discover_and_enqueue(
        &self,
        window: DiscoveryWindow,
    ) -> Result<u64, JobError> {
        let result = sqlx::query!(
            r#"
            WITH latest AS (
                SELECT ar.context_id,
                       (ARRAY_AGG(ar.user_id ORDER BY ar.created_at DESC, ar.id DESC))[1] AS user_id,
                       COUNT(*)::bigint AS request_count,
                       MAX(ar.created_at) AS last_at,
                       COUNT(*) FILTER (WHERE ar.request_kind = 'turn')::bigint AS turns
                FROM ai_requests ar
                WHERE ar.context_id <> $1
                  AND ar.actor_kind <> 'job'
                  AND NOT ar.synthetic
                  AND ar.created_at >= now() - make_interval(days => $2::int)
                GROUP BY ar.context_id
            ), quiet AS (
                SELECT * FROM latest l
                WHERE l.turns >= 1
                  AND (l.last_at < now() - make_interval(mins => $3::int)
                       OR EXISTS (SELECT 1 FROM ai_requests s
                                  JOIN plugin_usage_events e ON e.session_id = s.client_session_id
                                  WHERE s.context_id = l.context_id AND e.event_type = 'SessionEnd'
                                    AND e.created_at >= l.last_at))
            )
            INSERT INTO conversation_analyses (context_id, user_id, status)
            SELECT q.context_id, q.user_id, 'pending' FROM quiet q
            ON CONFLICT (context_id) DO UPDATE
                SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                    user_id = EXCLUDED.user_id, updated_at = clock_timestamp()
                WHERE conversation_analyses.status = 'classified'
                  AND conversation_analyses.lease_until IS NULL
                  AND (SELECT q.request_count FROM quiet q WHERE q.context_id = conversation_analyses.context_id)
                      > conversation_analyses.source_request_count
            "#,
            LEGACY_CONTEXT,
            window.lookback_days,
            window.quiet_minutes,
        )
        .execute(&self.pool)
        .await?;
        Ok(result.rows_affected())
    }

    // Why: forces one conversation back onto the queue regardless of state.
    pub(super) async fn mark_for_rejudging(
        &self,
        context_id: &ContextId,
    ) -> Result<bool, JobError> {
        let user_id = sqlx::query_scalar!(
            r#"SELECT (ARRAY_AGG(ar.user_id ORDER BY ar.created_at DESC, ar.id DESC))[1] AS "user_id"
               FROM ai_requests ar WHERE ar.context_id = $1 AND ar.actor_kind <> 'job'"#,
            context_id.as_str()
        )
        .fetch_one(&self.pool)
        .await?;
        let Some(user_id) = user_id else {
            return Ok(false);
        };
        sqlx::query!(
            r#"INSERT INTO conversation_analyses (context_id, user_id, status)
               VALUES ($1, $2, 'pending')
               ON CONFLICT (context_id) DO UPDATE
                   SET status = 'pending', attempts = 0, next_attempt = clock_timestamp(),
                       lease_token = NULL, lease_until = NULL, updated_at = clock_timestamp()"#,
            context_id.as_str(),
            user_id
        )
        .execute(&self.pool)
        .await?;
        Ok(true)
    }
}
