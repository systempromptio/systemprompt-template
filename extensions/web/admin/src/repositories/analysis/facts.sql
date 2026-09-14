WITH events AS MATERIALIZED (
 SELECT e.* FROM analysis_skill_events e JOIN users u ON u.id=e.user_id
 WHERE e.invoked_at >= $1 AND e.invoked_at < $2
 AND ($3::text IS NULL OR e.skill=$3)
 AND ($4::text IS NULL OR e.user_id=$4)
), links AS (
 SELECT DISTINCT e.user_id,e.session_id,r.id
 FROM events e JOIN ai_requests r ON r.user_id=e.user_id AND r.client_session_id=e.session_id
 UNION
 SELECT DISTINCT e.user_id,e.session_id,r.id
 FROM events e JOIN ai_request_tool_calls t ON t.ai_tool_call_id=e.tool_use_id
 JOIN ai_requests r ON r.id=t.request_id AND r.user_id=e.user_id
 WHERE (r.client_session_id IS NULL OR r.client_session_id=e.session_id)
 AND NOT EXISTS (
  SELECT 1 FROM plugin_usage_events other_event
  WHERE other_event.user_id=e.user_id AND other_event.metadata->>'tool_use_id'=e.tool_use_id
  AND other_event.session_id<>e.session_id
 )
), requests AS MATERIALIZED (
 SELECT DISTINCT l.session_id AS native_session,r.* FROM links l JOIN ai_requests r ON r.id=l.id
 WHERE NOT r.synthetic AND r.created_at >= $1 AND r.created_at < $2
 AND ($5::text IS NULL OR r.model=$5)
 AND NOT EXISTS(SELECT 1 FROM user_sessions s WHERE s.session_id=r.session_id AND s.user_id<>r.user_id)
 AND NOT EXISTS(SELECT 1 FROM eval_session_bindings b WHERE b.session_id=r.session_id)
), invocations AS (
 SELECT skill,user_id,session_id,count(*) AS uses,min(invoked_at) AS first_use,max(invoked_at) AS last_use
 FROM events GROUP BY skill,user_id,session_id
), facts AS (
 SELECT i.*,count(r.id) AS requests,
 count(r.id) FILTER(WHERE r.input_tokens IS NOT NULL AND r.output_tokens IS NOT NULL AND r.status='completed') AS measured,
 coalesce(sum(r.cost_microdollars),0)::bigint AS cost,
 coalesce(sum(coalesce(r.input_tokens,0)::bigint+coalesce(r.output_tokens,0)+coalesce(r.cache_read_tokens,0)+coalesce(r.cache_creation_tokens,0)),0)::bigint AS tokens,
 count(r.id) FILTER(WHERE r.status='failed') AS failures
 FROM invocations i LEFT JOIN requests r ON r.user_id=i.user_id AND r.native_session=i.session_id
 WHERE $5::text IS NULL OR r.id IS NOT NULL
 GROUP BY i.skill,i.user_id,i.session_id,i.uses,i.first_use,i.last_use
)
