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
)
SELECT
  r.marketplace_hash AS "marketplace_hash!",
  r.plugin_id AS "plugin_id!: PluginId",
  r.skill AS "skill!",
  c.tool AS "tool!",
  count(*)::bigint AS "calls!",
  count(DISTINCT c.context_id)::bigint AS "conversations!",
  count(*) FILTER (WHERE c.schema_error OR c.access_error OR c.upstream_error OR c.bad_arguments OR c.timeout)::bigint AS "failed_calls!",
  count(*) FILTER (WHERE c.schema_error)::bigint AS "schema_errors!",
  count(*) FILTER (WHERE c.access_error)::bigint AS "access_errors!",
  count(*) FILTER (WHERE c.upstream_error)::bigint AS "upstream_errors!",
  count(*) FILTER (WHERE c.bad_arguments)::bigint AS "bad_arguments!",
  count(*) FILTER (WHERE c.timeout)::bigint AS "timeouts!",
  count(*) FILTER (WHERE c.repeated)::bigint AS "repeated_calls!"
FROM classified c
JOIN runs r ON r.context_id = c.context_id
WHERE c.tool IS NOT NULL
GROUP BY r.marketplace_hash, r.plugin_id, r.skill, c.tool
ORDER BY 7 DESC, 5 DESC, 3, 4
LIMIT 5000
