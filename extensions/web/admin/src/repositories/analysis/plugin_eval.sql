WITH runs AS (
  SELECT s.context_id, s.plugin_id, s.skill, s.marketplace_hash, s.first_invoked_at
  FROM conversation_skill_facts s
  WHERE s.marketplace_id = $3 AND s.first_invoked_at >= $1 AND s.first_invoked_at < $2
),
turns AS (
  SELECT r.context_id, r.id, r.created_at
  FROM ai_requests r
  WHERE r.context_id IN (SELECT context_id FROM runs) AND r.request_kind = 'turn'
),
msg AS (
  SELECT t.context_id, t.created_at, m.role, m.content, m.sequence_number,
    lead(m.content) OVER (PARTITION BY m.request_id ORDER BY m.sequence_number) AS result
  FROM turns t
  JOIN ai_request_messages m ON m.request_id = t.id
  WHERE m.role <> 'system'
),
calls AS (
  SELECT DISTINCT ON (context_id, sequence_number, md5(content))
    context_id, sequence_number, substring(content FROM '^\[tool_use:([^ \]]+)') AS tool,
    content AS input, coalesce(result, '') AS result
  FROM msg
  WHERE role = 'assistant' AND content LIKE '[tool_use:%'
  ORDER BY context_id, sequence_number, md5(content), length(coalesce(result, '')) DESC
),
classified AS (
  SELECT context_id, tool, input,
    result ~ 'INVALID_FIELD|No such column|No such relation|INVALID_TYPE|is not supported|MALFORMED_QUERY|ValidationError' AS schema_error,
    result ~ 'INSUFFICIENT_ACCESS|Permission ''[^'']*'' denied|PERMISSION_DENIED' AS access_error,
    result ~ '"status_code"\W*5\d\d|UNKNOWN_EXCEPTION' AS upstream_error,
    result ~ 'Invalid JSON payload|Invalid value at ''|does not match \[a-z' AS bad_arguments,
    result ~* 'timed out' AS timeout,
    tool LIKE '%\_\_display\_widget' AND result !~ '"status_code"\W*[45]\d\d|isError' AS widget,
    tool ~* '(create|update|delete|upsert|send|insert)'
      OR input ~ '"method"\s*:\s*"(POST|PATCH|PUT|DELETE)"' AS write_call,
    count(*) OVER (PARTITION BY context_id, md5(input)) > 1 AS repeated
  FROM calls
),
per AS (
  SELECT c.context_id, r.skill,
    count(*) FILTER (WHERE c.tool LIKE 'mcp\_\_%') AS mcp_calls,
    count(*) FILTER (WHERE c.tool LIKE 'mcp\_\_plugin\_' || replace(r.plugin_id, '_', '\_') || '\_%') AS connector_calls,
    count(*) FILTER (WHERE c.tool NOT LIKE 'mcp\_\_%') AS builtin_calls,
    count(*) FILTER (WHERE c.schema_error OR c.access_error OR c.upstream_error OR c.bad_arguments OR c.timeout) AS failed_calls,
    count(*) FILTER (WHERE c.schema_error) AS schema_errors,
    count(*) FILTER (WHERE c.access_error) AS access_errors,
    count(*) FILTER (WHERE c.upstream_error) AS upstream_errors,
    count(*) FILTER (WHERE c.bad_arguments) AS bad_arguments,
    count(*) FILTER (WHERE c.timeout) AS timeouts,
    count(*) FILTER (WHERE c.widget) AS widgets,
    count(*) FILTER (WHERE c.write_call) AS writes,
    count(*) FILTER (WHERE c.repeated) - count(DISTINCT md5(c.input)) FILTER (WHERE c.repeated) AS repeated_calls
  FROM classified c
  JOIN runs r ON r.context_id = c.context_id
  GROUP BY c.context_id, r.skill
),
answer AS (
  SELECT DISTINCT ON (context_id) context_id, content
  FROM msg
  WHERE role = 'assistant' AND content NOT LIKE '[tool_use:%'
  ORDER BY context_id, created_at DESC, sequence_number DESC
),
judged AS (
  SELECT r.context_id, r.skill, coalesce(p.widgets, 0) AS widgets,
    coalesce(length(a.content), 0) AS answer_chars,
    coalesce(a.content ~* '(don''t|do not|cannot|can''t) (have|find|access|see)[^.]{0,80}(tool|connector)|not (available|connected) in this session|no [a-z ]{0,30}tool (is )?available', false) AS tools_unavailable,
    coalesce((SELECT count(*) FROM regexp_matches(a.content, '\mMOCK\M|placeholder|sample data', 'gi')), 0) AS placeholder_mentions
  FROM runs r
  LEFT JOIN per p ON p.context_id = r.context_id AND p.skill = r.skill
  LEFT JOIN answer a ON a.context_id = r.context_id
)
SELECT
  r.context_id AS "context_id!: ContextId",
  f.client_session_id AS "client_session_id?",
  r.plugin_id AS "plugin_id!: PluginId",
  r.skill AS "skill!",
  r.marketplace_hash AS "marketplace_hash!",
  r.first_invoked_at AS "first_invoked_at!",
  f.turn_count AS "turns!",
  f.request_count AS "requests!",
  f.input_tokens AS "input_tokens!",
  f.output_tokens AS "output_tokens!",
  f.cache_read_tokens AS "cache_read_tokens!",
  f.cache_creation_tokens AS "cache_creation_tokens!",
  (f.cost_microdollars - f.side_call_cost_microdollars) AS "cost!",
  f.duration_seconds AS "duration_seconds?",
  f.p50_latency_ms AS "p50_ms?",
  f.p95_latency_ms AS "p95_ms?",
  coalesce(p.mcp_calls, 0)::bigint AS "mcp_calls!",
  coalesce(p.connector_calls, 0)::bigint AS "connector_calls!",
  coalesce(p.builtin_calls, 0)::bigint AS "builtin_calls!",
  coalesce(p.failed_calls, 0)::bigint AS "failed_calls!",
  coalesce(p.schema_errors, 0)::bigint AS "schema_errors!",
  coalesce(p.access_errors, 0)::bigint AS "access_errors!",
  coalesce(p.upstream_errors, 0)::bigint AS "upstream_errors!",
  coalesce(p.bad_arguments, 0)::bigint AS "bad_arguments!",
  coalesce(p.timeouts, 0)::bigint AS "timeouts!",
  coalesce(p.repeated_calls, 0)::bigint AS "repeated_calls!",
  j.widgets::bigint AS "widgets!",
  coalesce(p.writes, 0)::bigint AS "writes!",
  j.answer_chars::bigint AS "answer_chars!",
  j.placeholder_mentions::bigint AS "placeholder_mentions!",
  j.tools_unavailable AS "tools_unavailable!",
  (j.widgets > 0 OR j.answer_chars >= 200) AS "completed!",
  ((j.widgets > 0 OR j.answer_chars >= 200)
    AND coalesce(p.connector_calls, 0) > 0
    AND coalesce(p.schema_errors, 0) = 0
    AND coalesce(p.writes, 0) = 0
    AND NOT j.tools_unavailable) AS "success!"
FROM runs r
JOIN conversation_facts f ON f.context_id = r.context_id
JOIN judged j ON j.context_id = r.context_id AND j.skill = r.skill
LEFT JOIN per p ON p.context_id = r.context_id AND p.skill = r.skill
ORDER BY r.first_invoked_at DESC, r.skill
LIMIT 5000
