-- One skill's invocations split by one dimension of the conversations that
-- invoked it: model, client, group, project, person, marketplace version or
-- host of the invocation. Each bucket carries the same deterministic figures
-- as the skill row so a facet reads like a smaller version of the page.
WITH events AS MATERIALIZED (
    SELECT e.invocation_id, e.user_id, e.session_id, e.invoked_at, e.plugin_id
    FROM analysis_skill_version_events e
    WHERE e.skill = $3 AND e.invoked_at >= $1 AND e.invoked_at < $2
      AND ($4::text[] IS NULL OR e.user_id = ANY($4))
), joined AS (
    SELECT e.invocation_id, e.user_id, e.session_id, f.context_id, f.model, f.client_kind,
           f.input_tokens + f.output_tokens AS tokens, f.cost_microdollars,
           f.error_count + f.gov_deny AS errors, f.p95_latency_ms, a.completion, a.outcome,
           g.name AS group_name, p.name AS project_name, u.display_name,
           (SELECT o.marketplace_id FROM service_owned_ids o WHERE o.kind = 'plugin' AND o.id = e.plugin_id LIMIT 1) AS marketplace_id,
           CASE $5::text WHEN 'version' THEN marketplace_version_at(
               (SELECT o.marketplace_id FROM service_owned_ids o WHERE o.kind = 'plugin' AND o.id = e.plugin_id LIMIT 1),
               e.invoked_at) END AS version_hash
    FROM events e
    LEFT JOIN conversation_facts f ON f.client_session_id = e.session_id
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    LEFT JOIN groups g ON g.id = f.group_id
    LEFT JOIN projects p ON p.id = f.project_id
    LEFT JOIN users u ON u.id = e.user_id
), keyed AS (
    SELECT j.*,
           CASE $5::text
               WHEN 'model' THEN COALESCE(j.model, 'unknown')
               WHEN 'client' THEN COALESCE(j.client_kind, 'unknown')
               WHEN 'group' THEN COALESCE(j.group_name, 'Unattributed')
               WHEN 'project' THEN COALESCE(j.project_name, 'Unattributed')
               WHEN 'user' THEN COALESCE(j.display_name, j.user_id)
               WHEN 'version' THEN COALESCE(LEFT(j.version_hash, 12), 'unknown')
               WHEN 'outcome' THEN COALESCE(j.outcome, 'unjudged')
               ELSE COALESCE(j.model, 'unknown') END AS bucket,
           CASE $5::text WHEN 'user' THEN j.user_id ELSE NULL END AS bucket_user
    FROM joined j
)
SELECT k.bucket AS "label!", k.bucket_user AS "user_id?: UserId",
       COUNT(*)::bigint AS "invocations!",
       COUNT(DISTINCT k.user_id)::bigint AS "users!",
       COUNT(DISTINCT k.context_id)::bigint AS "conversations!",
       COALESCE(SUM(k.tokens), 0)::bigint AS "tokens!",
       COALESCE(SUM(k.cost_microdollars), 0)::bigint AS "cost_microdollars!",
       COALESCE(SUM(k.errors), 0)::bigint AS "errors!",
       percentile_cont(0.95) WITHIN GROUP (ORDER BY k.p95_latency_ms)::float8 AS p95_latency_ms,
       COUNT(DISTINCT k.context_id) FILTER (WHERE k.completion IS NOT NULL)::bigint AS "judged!",
       AVG(k.completion)::float8 AS completion_avg
FROM keyed k
GROUP BY k.bucket, k.bucket_user
ORDER BY COUNT(*) DESC, k.bucket
LIMIT 50
