-- Every skill invoked in the window, with what the deterministic record says
-- about the conversations that invoked it. Invocations come from the hook
-- plane (analysis_skill_version_events, dashed `plugin:skill`, whose
-- plugin_id is the key's prefix, not the hook owner); spend, tokens,
-- tools, errors, governance and the judge's verdict come from the
-- conversation_facts rows whose harness session invoked the skill. Installs
-- are distinct consumers holding a verified receipt for the skill resource
-- (snake id), on any host. Sorting is a bound CASE arm.
WITH events AS MATERIALIZED (
    SELECT e.skill, e.plugin_id, e.user_id, e.session_id, e.source, e.attribution_status, e.invoked_at
    FROM analysis_skill_version_events e
    WHERE e.invoked_at >= $1 AND e.invoked_at < $2
      AND e.skill IS NOT NULL
      AND ($3::text[] IS NULL OR e.user_id = ANY($3))
      AND ($4::text IS NULL OR EXISTS (SELECT 1 FROM service_owned_ids o
                                       WHERE o.kind = 'plugin' AND o.id = e.plugin_id AND o.marketplace_id = $4))
      AND ($5::text IS NULL OR e.skill ILIKE $5 OR e.plugin_id ILIKE $5)
      AND ($10::text[] IS NULL OR e.skill = ANY($10))
), sessions AS (
    SELECT DISTINCT skill, plugin_id, user_id, session_id FROM events
), facts AS (
    SELECT s.skill, s.plugin_id,
           COUNT(DISTINCT f.context_id)::bigint AS conversations,
           COALESCE(SUM(f.request_count), 0)::bigint AS requests,
           COALESCE(SUM(f.turn_count), 0)::bigint AS turns,
           COALESCE(SUM(f.input_tokens), 0)::bigint AS input_tokens,
           COALESCE(SUM(f.output_tokens), 0)::bigint AS output_tokens,
           COALESCE(SUM(f.cache_read_tokens + f.cache_creation_tokens), 0)::bigint AS cache_tokens,
           COALESCE(SUM(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(SUM(f.error_count), 0)::bigint AS errors,
           COALESCE(SUM(f.gov_deny), 0)::bigint AS denied,
           COALESCE(SUM(f.tool_calls_intended), 0)::bigint AS tool_calls,
           COALESCE(SUM(f.tool_calls_failed), 0)::bigint AS tool_calls_failed,
           COALESCE(SUM(f.artifact_count), 0)::bigint AS artifacts,
           percentile_cont(0.95) WITHIN GROUP (ORDER BY f.p95_latency_ms)::float8 AS p95_latency_ms,
           COUNT(DISTINCT f.context_id) FILTER (WHERE a.completion IS NOT NULL)::bigint AS judged,
           AVG(a.completion)::float8 AS completion_avg,
           COUNT(DISTINCT f.context_id) FILTER (WHERE a.outcome = 'achieved')::bigint AS achieved,
           (ARRAY_AGG(DISTINCT f.model) FILTER (WHERE f.model IS NOT NULL))[1:6] AS models,
           (ARRAY_AGG(DISTINCT f.client_kind))[1:6] AS clients
    FROM sessions s
    JOIN conversation_facts f ON f.client_session_id = s.session_id
        AND ($6::text IS NULL OR f.client_kind = $6)
    LEFT JOIN conversation_analyses a ON a.context_id = f.context_id
    GROUP BY s.skill, s.plugin_id
), installs AS (
    SELECT sk.skill, sk.plugin_id,
           COUNT(DISTINCT r.consumer_id)::bigint AS installs,
           COUNT(DISTINCT r.host)::bigint AS install_hosts
    FROM (SELECT DISTINCT skill, plugin_id FROM events) sk
    JOIN managed_resources res ON res.kind = 'skill'
        AND res.resource_key = replace(split_part(sk.skill, ':', 2), '-', '_')
    JOIN managed_installation_receipts r ON r.resource_id = res.id AND r.fully_verified
        AND r.consumer_id IS NOT NULL
    GROUP BY sk.skill, sk.plugin_id
), spark AS (
    SELECT e.skill, e.plugin_id, date_trunc('day', e.invoked_at)::date AS day, COUNT(*)::bigint AS n
    FROM events e GROUP BY e.skill, e.plugin_id, 3
), sparks AS (
    SELECT skill, plugin_id,
           ARRAY_AGG(day ORDER BY day) AS spark_days,
           ARRAY_AGG(n ORDER BY day) AS spark_counts
    FROM spark GROUP BY skill, plugin_id
), rows AS (
    SELECT ev.skill, ev.plugin_id,
           (SELECT o.marketplace_id FROM service_owned_ids o WHERE o.kind = 'plugin' AND o.id = ev.plugin_id LIMIT 1) AS marketplace_id,
           COUNT(*)::bigint AS invocations,
           COUNT(*) FILTER (WHERE ev.source = 'slash')::bigint AS slash,
           COUNT(*) FILTER (WHERE ev.source = 'tool')::bigint AS tool,
           COUNT(*) FILTER (WHERE ev.attribution_status = 'verified')::bigint AS attributed,
           COUNT(DISTINCT ev.user_id)::bigint AS users,
           COUNT(DISTINCT ev.session_id)::bigint AS sessions,
           MIN(ev.invoked_at) AS first_used, MAX(ev.invoked_at) AS last_used,
           COALESCE(MAX(f.conversations), 0)::bigint AS conversations,
           COALESCE(MAX(f.requests), 0)::bigint AS requests,
           COALESCE(MAX(f.turns), 0)::bigint AS turns,
           COALESCE(MAX(f.input_tokens), 0)::bigint AS input_tokens,
           COALESCE(MAX(f.output_tokens), 0)::bigint AS output_tokens,
           COALESCE(MAX(f.cache_tokens), 0)::bigint AS cache_tokens,
           COALESCE(MAX(f.cost_microdollars), 0)::bigint AS cost_microdollars,
           COALESCE(MAX(f.errors), 0)::bigint AS errors,
           COALESCE(MAX(f.denied), 0)::bigint AS denied,
           COALESCE(MAX(f.tool_calls), 0)::bigint AS tool_calls,
           COALESCE(MAX(f.tool_calls_failed), 0)::bigint AS tool_calls_failed,
           COALESCE(MAX(f.artifacts), 0)::bigint AS artifacts,
           MAX(f.p95_latency_ms) AS p95_latency_ms,
           COALESCE(MAX(f.judged), 0)::bigint AS judged,
           MAX(f.completion_avg) AS completion_avg,
           COALESCE(MAX(f.achieved), 0)::bigint AS achieved,
           COALESCE(MAX(f.models), '{}'::text[]) AS models,
           COALESCE(MAX(f.clients), '{}'::text[]) AS clients,
           COALESCE(MAX(i.installs), 0)::bigint AS installs,
           COALESCE(MAX(i.install_hosts), 0)::bigint AS install_hosts,
           COALESCE(MAX(sp.spark_days), '{}'::date[]) AS spark_days,
           COALESCE(MAX(sp.spark_counts), '{}'::bigint[]) AS spark_counts
    FROM events ev
    LEFT JOIN facts f ON f.skill = ev.skill AND f.plugin_id = ev.plugin_id
    LEFT JOIN installs i ON i.skill = ev.skill AND i.plugin_id = ev.plugin_id
    LEFT JOIN sparks sp ON sp.skill = ev.skill AND sp.plugin_id = ev.plugin_id
    GROUP BY ev.skill, ev.plugin_id
)
SELECT r.skill AS "skill!", r.plugin_id AS "plugin_id?: PluginId", r.marketplace_id AS "marketplace_id?: MarketplaceId",
       r.invocations AS "invocations!", r.slash AS "slash!", r.tool AS "tool!",
       r.attributed AS "attributed!", r.users AS "users!", r.sessions AS "sessions!",
       r.first_used AS "first_used!", r.last_used AS "last_used!",
       r.conversations AS "conversations!", r.requests AS "requests!", r.turns AS "turns!",
       r.input_tokens AS "input_tokens!", r.output_tokens AS "output_tokens!",
       r.cache_tokens AS "cache_tokens!", r.cost_microdollars AS "cost_microdollars!",
       r.errors AS "errors!", r.denied AS "denied!", r.tool_calls AS "tool_calls!",
       r.tool_calls_failed AS "tool_calls_failed!", r.artifacts AS "artifacts!",
       r.p95_latency_ms, r.judged AS "judged!", r.completion_avg, r.achieved AS "achieved!",
       r.models AS "models!", r.clients AS "clients!",
       r.installs AS "installs!", r.install_hosts AS "install_hosts!",
       r.spark_days AS "spark_days!", r.spark_counts AS "spark_counts!",
       COUNT(*) OVER ()::bigint AS "total!"
FROM rows r
ORDER BY
    CASE WHEN $7 = 'invocations' THEN r.invocations END DESC,
    CASE WHEN $7 = 'users' THEN r.users END DESC,
    CASE WHEN $7 = 'cost' THEN r.cost_microdollars END DESC,
    CASE WHEN $7 = 'tokens' THEN r.input_tokens + r.output_tokens END DESC,
    CASE WHEN $7 = 'errors' THEN r.errors + r.denied END DESC,
    CASE WHEN $7 = 'completion' THEN r.completion_avg END DESC NULLS LAST,
    CASE WHEN $7 = 'recent' THEN r.last_used END DESC,
    r.invocations DESC, r.skill
LIMIT $8 OFFSET $9
