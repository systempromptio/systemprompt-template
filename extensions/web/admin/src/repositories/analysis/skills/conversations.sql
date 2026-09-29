-- The conversations that invoked one skill in the window, newest first, with
-- the same fact columns the conversations page shows and the judge's row.
WITH sessions AS (
    SELECT e.session_id, COUNT(*)::bigint AS invocations, MIN(e.invoked_at) AS first_invoked_at
    FROM analysis_skill_version_events e
    WHERE e.skill = $3 AND e.invoked_at >= $1 AND e.invoked_at < $2
      AND ($4::text[] IS NULL OR e.user_id = ANY($4))
    GROUP BY e.session_id
)
SELECT f.context_id AS "context_id!: ContextId", f.user_id AS "user_id!: UserId", u.display_name,
       f.client_session_id, g.name AS "group_name?", p.name AS "project_name?",
       f.client_kind AS "client_kind!", f.model, f.models AS "models!",
       f.turn_count AS "turn_count!", f.tool_calls_intended AS "tool_calls!",
       f.tool_calls_failed AS "tool_calls_failed!", f.artifact_count AS "artifacts!",
       f.error_count AS "error_count!",
       f.gov_deny AS "gov_deny!",
       (f.input_tokens + f.output_tokens)::bigint AS "total_tokens!",
       (f.cache_read_tokens + f.cache_creation_tokens)::bigint AS "cache_tokens!",
       f.cost_microdollars AS "cost_microdollars!", f.p95_latency_ms,
       f.duration_seconds AS "duration_seconds!", f.skills AS "skills!",
       f.first_at AS "first_at!", f.last_at AS "last_at!",
       s.invocations AS "invocations!",
       a.title AS judge_title, a.category, a.outcome, a.completion, a.summary,
       conversation_title(f.context_id, f.client_session_id) AS "title!",
       COUNT(*) OVER ()::bigint AS "total!"
FROM sessions s
JOIN conversation_facts f ON f.client_session_id = s.session_id
LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
LEFT JOIN users u ON u.id = f.user_id
LEFT JOIN groups g ON g.id = f.group_id
LEFT JOIN projects p ON p.id = f.project_id
ORDER BY f.last_at DESC, f.context_id
LIMIT $5 OFFSET $6
