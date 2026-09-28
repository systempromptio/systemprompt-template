-- The deterministic record of one conversation, and the one judge row on top.
--
-- `conversation_facts` is what the Analysis pages read: one row per gateway
-- context with everything the gateway, the hook plane, the tool ledger and
-- the governance spine recorded about it — who, which client, which models,
-- tokens by kind, cost, latency percentiles, tool calls by state, artifacts,
-- safety findings, governance decisions, prompts and skills. Nothing here is
-- an opinion; every column is a count, a sum or a percentile over rows core
-- and the hooks wrote. The AI's contribution to a conversation is the single
-- `conversation_analyses` row (title, summary, intent category, outcome and
-- one 0–100 `completion` score) and the pages join it, never the other way
-- round.
--
-- The table is a rollup, refreshed by `refresh_conversation_facts` — the
-- `conversation_rollup` job calls it for every context touched since its last
-- tick and a detail page calls it for the one context it is about to show —
-- so list pages sort, filter and break down an indexed table instead of
-- aggregating the request log on every load.
--
-- Join keys, in order of trust: the gateway context (`ai_requests.context_id`)
-- is the row; the harness session (`ai_requests.client_session_id`, the
-- Claude Code uuid) attaches hook events, governance decisions and hook-sourced
-- tool executions, whose `session_id`/`trace_id` carry that same uuid.
--
-- The judge's columns on conversation_analyses are declared in 37_.
-- Declarative twin of migration 091. `artifact_count`, `artifact_files` and
-- `artifact_cards` count by the one artifact definition in
-- 46_tool_artifacts.sql (`tool_activity.artifact_kind`).
--
-- `conversation_skill_facts` is the per-skill half of the same record and is
-- written by the same refresh; `conversation_rollup_state` is the rollup job's
-- watermark. Both are kept as long as `conversation_facts` (forever) and are
-- declared by the same migration.

CREATE INDEX IF NOT EXISTS idx_conversation_analyses_completion
    ON conversation_analyses(completion) WHERE completion IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_ai_requests_updated_at ON ai_requests(updated_at);

CREATE TABLE IF NOT EXISTS conversation_facts (
    context_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    session_id TEXT,
    client_session_id TEXT,
    group_id TEXT,
    project_id TEXT,
    client_kind TEXT NOT NULL DEFAULT 'unknown',
    client_attestation TEXT NOT NULL DEFAULT 'unknown',
    wire_protocol TEXT NOT NULL DEFAULT 'unknown',
    model TEXT,
    provider TEXT,
    models TEXT[] NOT NULL DEFAULT '{}',
    providers TEXT[] NOT NULL DEFAULT '{}',
    request_count BIGINT NOT NULL DEFAULT 0,
    turn_count BIGINT NOT NULL DEFAULT 0,
    side_call_count BIGINT NOT NULL DEFAULT 0,
    side_call_cost_microdollars BIGINT NOT NULL DEFAULT 0,
    error_count BIGINT NOT NULL DEFAULT 0,
    rejected_count BIGINT NOT NULL DEFAULT 0,
    streaming_count BIGINT NOT NULL DEFAULT 0,
    input_tokens BIGINT NOT NULL DEFAULT 0,
    output_tokens BIGINT NOT NULL DEFAULT 0,
    cache_read_tokens BIGINT NOT NULL DEFAULT 0,
    cache_creation_tokens BIGINT NOT NULL DEFAULT 0,
    reasoning_tokens BIGINT NOT NULL DEFAULT 0,
    cost_microdollars BIGINT NOT NULL DEFAULT 0,
    p50_latency_ms INTEGER,
    p95_latency_ms INTEGER,
    max_latency_ms INTEGER,
    active_ms BIGINT NOT NULL DEFAULT 0,
    tool_calls_intended BIGINT NOT NULL DEFAULT 0,
    tool_calls_executed BIGINT NOT NULL DEFAULT 0,
    tool_calls_failed BIGINT NOT NULL DEFAULT 0,
    artifact_count BIGINT NOT NULL DEFAULT 0,
    artifact_files BIGINT NOT NULL DEFAULT 0,
    artifact_cards BIGINT NOT NULL DEFAULT 0,
    safety_findings BIGINT NOT NULL DEFAULT 0,
    safety_blocked BIGINT NOT NULL DEFAULT 0,
    gov_allow BIGINT NOT NULL DEFAULT 0,
    gov_warn BIGINT NOT NULL DEFAULT 0,
    gov_deny BIGINT NOT NULL DEFAULT 0,
    prompt_count BIGINT NOT NULL DEFAULT 0,
    hook_event_count BIGINT NOT NULL DEFAULT 0,
    hook_status TEXT,
    skill_invocations BIGINT NOT NULL DEFAULT 0,
    skills TEXT[] NOT NULL DEFAULT '{}',
    first_at TIMESTAMPTZ NOT NULL,
    last_at TIMESTAMPTZ NOT NULL,
    duration_seconds BIGINT NOT NULL DEFAULT 0,
    refreshed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_last_at ON conversation_facts(last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_user ON conversation_facts(user_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_group ON conversation_facts(group_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_project ON conversation_facts(project_id, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_model ON conversation_facts(model, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_client ON conversation_facts(client_kind, last_at DESC);
CREATE INDEX IF NOT EXISTS idx_conversation_facts_session ON conversation_facts(client_session_id)
    WHERE client_session_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_conversation_facts_skills ON conversation_facts USING gin(skills);

-- One row per (conversation, skill): how often it was invoked and failed, and
-- which marketplace version was being served when the conversation first
-- invoked it. The version is fixed at first invocation — `invoked_at` is
-- history, so a deploy after the fact never credits the new hash with
-- behaviour the old one produced. The Versions pages read this table joined
-- to `conversation_facts`, so their figures outlive raw-event retention.
CREATE TABLE IF NOT EXISTS conversation_skill_facts (
    context_id TEXT NOT NULL REFERENCES conversation_facts(context_id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    skill TEXT NOT NULL,
    user_id TEXT NOT NULL,
    marketplace_id TEXT,
    marketplace_hash TEXT,
    invocations BIGINT NOT NULL DEFAULT 0,
    failures BIGINT NOT NULL DEFAULT 0,
    first_invoked_at TIMESTAMPTZ NOT NULL,
    last_invoked_at TIMESTAMPTZ NOT NULL,
    refreshed_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    PRIMARY KEY (context_id, plugin_id, skill)
);
CREATE INDEX IF NOT EXISTS idx_conversation_skill_facts_version
    ON conversation_skill_facts(marketplace_id, marketplace_hash, first_invoked_at);
CREATE INDEX IF NOT EXISTS idx_conversation_skill_facts_first
    ON conversation_skill_facts(first_invoked_at);

-- The rollup job's high-water mark: every source row stamped before it has
-- been folded into the tables above. One row.
CREATE TABLE IF NOT EXISTS conversation_rollup_state (
    id BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (id),
    watermark TIMESTAMPTZ NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp()
);

-- Rebuilds the fact row for each context given (NULL = every context that
-- has a conversation request), and its skill rows. Returns the number of
-- fact rows written. The request classification is `conversation_requests`
-- (27_), so a turn here is a turn everywhere; the group and project are the
-- ones stamped on the latest request, so a conversation stays filed where it
-- was made.
CREATE OR REPLACE FUNCTION refresh_conversation_facts(context_ids TEXT[])
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE written BIGINT; stamp TIMESTAMPTZ := clock_timestamp();
BEGIN
    WITH requests AS MATERIALIZED (
        -- Why: this repo's conversation_requests (27_) does not carry the
        -- client columns, so they are read from ai_requests, joined anyway.
        -- When astound's 27_ (which exposes them) is ported, drop
        -- `ar.client_kind, ar.client_attestation` here or the CTE carries
        -- duplicate column names. Only this file changes; 091 is history.
        SELECT r.*, ar.client_kind, ar.client_attestation,
               ar.wire_protocol, ar.cache_read_tokens, ar.cache_creation_tokens,
               ar.reasoning_tokens, ar.is_streaming
        FROM conversation_requests r
        JOIN ai_requests ar ON ar.id = r.id
        WHERE context_ids IS NULL OR r.context_id = ANY(context_ids)
    ), agg AS (
        SELECT r.context_id,
            (ARRAY_AGG(r.user_id ORDER BY r.created_at DESC, r.id DESC))[1] AS user_id,
            (ARRAY_AGG(r.id ORDER BY r.created_at DESC, r.id DESC))[1] AS last_request_id,
            (ARRAY_AGG(r.session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.session_id IS NOT NULL))[1] AS session_id,
            (ARRAY_AGG(r.client_session_id ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.client_session_id IS NOT NULL))[1] AS client_session_id,
            (ARRAY_AGG(r.client_kind ORDER BY r.created_at DESC, r.id DESC))[1] AS client_kind,
            (ARRAY_AGG(r.client_attestation ORDER BY r.created_at DESC, r.id DESC))[1] AS client_attestation,
            (ARRAY_AGG(r.wire_protocol ORDER BY r.created_at DESC, r.id DESC))[1] AS wire_protocol,
            (ARRAY_AGG(r.model ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.model IS NOT NULL))[1] AS model,
            (ARRAY_AGG(r.provider ORDER BY r.created_at DESC, r.id DESC)
                FILTER (WHERE r.effective_kind = 'turn' AND r.provider IS NOT NULL))[1] AS provider,
            COUNT(*)::bigint AS request_count,
            COUNT(*) FILTER (WHERE r.effective_kind = 'turn')::bigint AS turn_count,
            COUNT(*) FILTER (WHERE r.effective_kind <> 'turn')::bigint AS side_call_count,
            COALESCE(SUM(r.cost_microdollars) FILTER (WHERE r.effective_kind <> 'turn'), 0)::bigint AS side_call_cost,
            COUNT(*) FILTER (WHERE r.status = 'failed')::bigint AS error_count,
            COUNT(*) FILTER (WHERE r.status = 'rejected')::bigint AS rejected_count,
            COUNT(*) FILTER (WHERE r.is_streaming)::bigint AS streaming_count,
            COALESCE(SUM(r.input_tokens), 0)::bigint AS input_tokens,
            COALESCE(SUM(r.output_tokens), 0)::bigint AS output_tokens,
            COALESCE(SUM(r.cache_read_tokens), 0)::bigint AS cache_read_tokens,
            COALESCE(SUM(r.cache_creation_tokens), 0)::bigint AS cache_creation_tokens,
            COALESCE(SUM(r.reasoning_tokens), 0)::bigint AS reasoning_tokens,
            COALESCE(SUM(r.cost_microdollars), 0)::bigint AS cost_microdollars,
            percentile_cont(0.5) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p50,
            percentile_cont(0.95) WITHIN GROUP (ORDER BY r.latency_ms)
                FILTER (WHERE r.latency_ms IS NOT NULL AND r.effective_kind = 'turn') AS p95,
            MAX(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn') AS max_latency_ms,
            COALESCE(SUM(r.latency_ms) FILTER (WHERE r.effective_kind = 'turn'), 0)::bigint AS active_ms,
            MIN(r.created_at) AS first_at,
            MAX(COALESCE(r.completed_at, r.created_at)) AS last_at
        FROM requests r
        GROUP BY r.context_id
    ), used AS (
        SELECT r.context_id,
            ARRAY(SELECT m.model FROM (
                SELECT model, MIN(created_at) AS first_use FROM requests q
                WHERE q.context_id = r.context_id AND q.model IS NOT NULL GROUP BY model) m
                ORDER BY m.first_use, m.model) AS models,
            ARRAY(SELECT DISTINCT q.provider FROM requests q
                WHERE q.context_id = r.context_id AND q.provider IS NOT NULL ORDER BY q.provider) AS providers
        FROM (SELECT DISTINCT context_id FROM requests) r
    ), intents AS (
        SELECT r.context_id, COUNT(*)::bigint AS tool_calls_intended
        FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
        GROUP BY r.context_id
    ), safety AS (
        SELECT r.context_id, COUNT(*)::bigint AS findings,
               COUNT(*) FILTER (WHERE f.blocked)::bigint AS blocked
        FROM requests r JOIN ai_safety_findings f ON f.ai_request_id = r.id
        GROUP BY r.context_id
    ), sessions AS (
        SELECT DISTINCT r.client_session_id
        FROM requests r WHERE r.client_session_id IS NOT NULL
    ), windows AS (
        SELECT r.context_id, r.client_session_id,
               MIN(r.created_at) AS opened_at,
               MAX(COALESCE(r.completed_at, r.created_at)) AS closed_at
        FROM conversation_requests r
        WHERE r.client_session_id IN (SELECT s.client_session_id FROM sessions s)
        GROUP BY r.context_id, r.client_session_id
    ), executions AS (
        SELECT o.context_id,
               COUNT(*)::bigint AS executed,
               COUNT(*) FILTER (WHERE o.execution_status IN ('failed', 'timeout'))::bigint AS failed,
               COUNT(*) FILTER (WHERE o.artifact_kind IS NOT NULL)::bigint AS artifacts,
               COUNT(*) FILTER (WHERE o.artifact_kind = 'file')::bigint AS artifact_files,
               COUNT(*) FILTER (WHERE o.artifact_kind IN ('ui', 'card', 'body'))::bigint AS artifact_cards
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                        THEN t.execution_context_id ELSE w.context_id END AS context_id,
                   t.execution_status, t.artifact_kind
            FROM tool_activity t
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                  AND x.client_session_id = t.execution_trace_id
                ORDER BY (COALESCE(t.executed_at, t.occurred_at) BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM COALESCE(t.executed_at, t.occurred_at) - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE t.mcp_execution_id IS NOT NULL
              AND (EXISTS (SELECT 1 FROM agg a WHERE a.context_id = t.execution_context_id)
                   OR w.context_id IS NOT NULL)
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), governance AS (
        SELECT o.context_id,
               COUNT(*) FILTER (WHERE o.decision = 'allow')::bigint AS allow,
               COUNT(*) FILTER (WHERE o.decision = 'warn')::bigint AS warn,
               COUNT(*) FILTER (WHERE o.decision = 'deny')::bigint AS deny
        FROM (
            SELECT CASE WHEN EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                        THEN g.context_id ELSE w.context_id END AS context_id, g.decision
            FROM governance_decisions g
            LEFT JOIN LATERAL (
                SELECT x.context_id FROM windows x
                WHERE NOT EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
                  AND x.client_session_id = g.session_id
                ORDER BY (g.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                         abs(EXTRACT(EPOCH FROM g.created_at - x.opened_at)), x.opened_at, x.context_id
                LIMIT 1) w ON TRUE
            WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = g.context_id)
               OR w.context_id IS NOT NULL
        ) o
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = o.context_id)
        GROUP BY o.context_id
    ), hook_events AS (
        SELECT w.context_id, e.event_type, e.id
        FROM plugin_usage_events e
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = e.session_id
            ORDER BY (e.created_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM e.created_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), hooks AS (
        SELECT h.context_id,
               COUNT(*)::bigint AS hook_events,
               COUNT(*) FILTER (WHERE h.event_type = 'UserPromptSubmit')::bigint AS prompts
        FROM hook_events h
        GROUP BY h.context_id
    ), skill_events AS (
        SELECT w.context_id, s.id, s.plugin_id, s.skill, s.invoked_at
        FROM analysis_skill_events s
        JOIN LATERAL (
            SELECT x.context_id FROM windows x
            WHERE x.client_session_id = s.session_id
            ORDER BY (s.invoked_at BETWEEN x.opened_at AND x.closed_at) DESC,
                     abs(EXTRACT(EPOCH FROM s.invoked_at - x.opened_at)), x.opened_at, x.context_id
            LIMIT 1) w ON TRUE
        WHERE s.skill IS NOT NULL AND EXISTS (SELECT 1 FROM agg a WHERE a.context_id = w.context_id)
    ), skills AS (
        SELECT k.context_id, COUNT(*)::bigint AS invocations,
               ARRAY(SELECT y.skill FROM (
                   SELECT q.skill, MIN(q.invoked_at) AS first_at FROM skill_events q
                   WHERE q.context_id = k.context_id GROUP BY q.skill) y
                   ORDER BY y.first_at, y.skill) AS skills
        FROM skill_events k
        GROUP BY k.context_id
    ), skill_upsert AS (
        INSERT INTO conversation_skill_facts AS t (
            context_id, plugin_id, skill, user_id, marketplace_id, marketplace_hash,
            invocations, failures, first_invoked_at, last_invoked_at, refreshed_at)
        SELECT k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id,
               marketplace_version_at(o.marketplace_id, MIN(k.invoked_at)),
               COUNT(*)::bigint,
               COUNT(*) FILTER (WHERE p.event_type = 'PostToolUseFailure')::bigint,
               MIN(k.invoked_at), MAX(k.invoked_at), stamp
        FROM skill_events k
        JOIN agg a ON a.context_id = k.context_id
        JOIN plugin_usage_events p ON p.id = k.id
        LEFT JOIN service_owned_ids o ON o.kind = 'plugin' AND o.id = k.plugin_id
        WHERE k.plugin_id IS NOT NULL
        GROUP BY k.context_id, k.plugin_id, k.skill, a.user_id, o.marketplace_id
        ON CONFLICT (context_id, plugin_id, skill) DO UPDATE SET
            user_id = EXCLUDED.user_id, marketplace_id = EXCLUDED.marketplace_id,
            marketplace_hash = EXCLUDED.marketplace_hash, invocations = EXCLUDED.invocations,
            failures = EXCLUDED.failures, first_invoked_at = EXCLUDED.first_invoked_at,
            last_invoked_at = EXCLUDED.last_invoked_at, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1
    ), upsert AS (
        INSERT INTO conversation_facts AS f (
            context_id, user_id, session_id, client_session_id, group_id, project_id,
            client_kind, client_attestation, wire_protocol, model, provider, models, providers,
            request_count, turn_count, side_call_count, side_call_cost_microdollars,
            error_count, rejected_count, streaming_count,
            input_tokens, output_tokens, cache_read_tokens, cache_creation_tokens, reasoning_tokens,
            cost_microdollars, p50_latency_ms, p95_latency_ms, max_latency_ms, active_ms,
            tool_calls_intended, tool_calls_executed, tool_calls_failed, artifact_count,
            artifact_files, artifact_cards,
            safety_findings, safety_blocked, gov_allow, gov_warn, gov_deny,
            prompt_count, hook_event_count, hook_status, skill_invocations, skills,
            first_at, last_at, duration_seconds, refreshed_at)
        SELECT a.context_id, a.user_id, a.session_id, a.client_session_id, rs.group_id, rs.project_id,
               COALESCE(a.client_kind, 'unknown'), COALESCE(a.client_attestation, 'unknown'),
               COALESCE(a.wire_protocol, 'unknown'), a.model, a.provider,
               COALESCE(u.models, '{}'), COALESCE(u.providers, '{}'),
               a.request_count, a.turn_count, a.side_call_count, a.side_call_cost,
               a.error_count, a.rejected_count, a.streaming_count,
               a.input_tokens, a.output_tokens, a.cache_read_tokens, a.cache_creation_tokens, a.reasoning_tokens,
               a.cost_microdollars, a.p50::int, a.p95::int, a.max_latency_ms, a.active_ms,
               COALESCE(i.tool_calls_intended, 0), COALESCE(x.executed, 0), COALESCE(x.failed, 0), COALESCE(x.artifacts, 0),
               COALESCE(x.artifact_files, 0), COALESCE(x.artifact_cards, 0),
               COALESCE(s.findings, 0), COALESCE(s.blocked, 0),
               COALESCE(g.allow, 0), COALESCE(g.warn, 0), COALESCE(g.deny, 0),
               COALESCE(h.prompts, 0), COALESCE(h.hook_events, 0), ps.status,
               COALESCE(k.invocations, 0), COALESCE(k.skills, '{}'),
               a.first_at, GREATEST(a.last_at, c.updated_at),
               GREATEST(EXTRACT(EPOCH FROM GREATEST(a.last_at, c.updated_at) - a.first_at)::bigint, 0),
               clock_timestamp()
        FROM agg a
        LEFT JOIN used u ON u.context_id = a.context_id
        LEFT JOIN intents i ON i.context_id = a.context_id
        LEFT JOIN safety s ON s.context_id = a.context_id
        LEFT JOIN executions x ON x.context_id = a.context_id
        LEFT JOIN governance g ON g.context_id = a.context_id
        LEFT JOIN hooks h ON h.context_id = a.context_id
        LEFT JOIN skills k ON k.context_id = a.context_id
        LEFT JOIN ai_request_scopes rs ON rs.request_id = a.last_request_id
        LEFT JOIN user_contexts c ON c.context_id = a.context_id
        LEFT JOIN plugin_session_summaries ps ON ps.session_id = a.client_session_id
        ON CONFLICT (context_id) DO UPDATE SET
            user_id = EXCLUDED.user_id, session_id = EXCLUDED.session_id,
            client_session_id = EXCLUDED.client_session_id,
            group_id = EXCLUDED.group_id, project_id = EXCLUDED.project_id,
            client_kind = EXCLUDED.client_kind, client_attestation = EXCLUDED.client_attestation,
            wire_protocol = EXCLUDED.wire_protocol, model = EXCLUDED.model, provider = EXCLUDED.provider,
            models = EXCLUDED.models, providers = EXCLUDED.providers,
            request_count = EXCLUDED.request_count, turn_count = EXCLUDED.turn_count,
            side_call_count = EXCLUDED.side_call_count,
            side_call_cost_microdollars = EXCLUDED.side_call_cost_microdollars,
            error_count = EXCLUDED.error_count, rejected_count = EXCLUDED.rejected_count,
            streaming_count = EXCLUDED.streaming_count,
            input_tokens = EXCLUDED.input_tokens, output_tokens = EXCLUDED.output_tokens,
            cache_read_tokens = EXCLUDED.cache_read_tokens,
            cache_creation_tokens = EXCLUDED.cache_creation_tokens,
            reasoning_tokens = EXCLUDED.reasoning_tokens, cost_microdollars = EXCLUDED.cost_microdollars,
            p50_latency_ms = EXCLUDED.p50_latency_ms, p95_latency_ms = EXCLUDED.p95_latency_ms,
            max_latency_ms = EXCLUDED.max_latency_ms, active_ms = EXCLUDED.active_ms,
            tool_calls_intended = EXCLUDED.tool_calls_intended,
            tool_calls_executed = EXCLUDED.tool_calls_executed,
            tool_calls_failed = EXCLUDED.tool_calls_failed, artifact_count = EXCLUDED.artifact_count,
            artifact_files = EXCLUDED.artifact_files, artifact_cards = EXCLUDED.artifact_cards,
            safety_findings = EXCLUDED.safety_findings, safety_blocked = EXCLUDED.safety_blocked,
            gov_allow = EXCLUDED.gov_allow, gov_warn = EXCLUDED.gov_warn, gov_deny = EXCLUDED.gov_deny,
            prompt_count = EXCLUDED.prompt_count, hook_event_count = EXCLUDED.hook_event_count,
            hook_status = EXCLUDED.hook_status, skill_invocations = EXCLUDED.skill_invocations,
            skills = EXCLUDED.skills, first_at = EXCLUDED.first_at, last_at = EXCLUDED.last_at,
            duration_seconds = EXCLUDED.duration_seconds, refreshed_at = EXCLUDED.refreshed_at
        RETURNING 1)
    SELECT COUNT(*) INTO written FROM upsert;
    -- Why: a skill row this run did not rewrite belongs to a skill the
    -- conversation no longer shows. Only conversations rebuilt here are
    -- judged, so a conversation whose raw events expired keeps its rows.
    DELETE FROM conversation_skill_facts s
    USING conversation_facts f
    WHERE f.context_id = s.context_id AND f.refreshed_at >= stamp AND s.refreshed_at < stamp;
    IF context_ids IS NOT NULL THEN
        DELETE FROM conversation_facts f WHERE f.context_id = ANY(context_ids)
            AND NOT EXISTS (SELECT 1 FROM conversation_requests r WHERE r.context_id = f.context_id);
    END IF;
    RETURN COALESCE(written, 0);
END $$;

-- Every context touched in [from_at, to_at) on any plane that feeds the row: a
-- request settled, a hook event, a governance decision, a tool execution or
-- an artifact.
CREATE OR REPLACE FUNCTION refresh_conversation_facts_between(from_at TIMESTAMPTZ, to_at TIMESTAMPTZ)
RETURNS BIGINT LANGUAGE sql AS $$
SELECT refresh_conversation_facts(ARRAY(
    SELECT DISTINCT context_id FROM (
        SELECT r.context_id FROM ai_requests r
        WHERE (r.updated_at >= from_at AND r.updated_at < to_at)
           OR (r.created_at >= from_at AND r.created_at < to_at)
        UNION ALL
        SELECT r.context_id FROM ai_requests r
        WHERE r.client_session_id IN (
            SELECT e.session_id FROM plugin_usage_events e
            WHERE e.created_at >= from_at AND e.created_at < to_at
            UNION ALL
            SELECT d.session_id FROM governance_decisions d
            WHERE d.created_at >= from_at AND d.created_at < to_at
            UNION ALL
            SELECT x.trace_id FROM mcp_tool_executions x
            WHERE x.created_at >= from_at AND x.created_at < to_at AND x.trace_id IS NOT NULL)
        UNION ALL
        SELECT x.context_id FROM mcp_tool_executions x
        WHERE x.created_at >= from_at AND x.created_at < to_at AND x.context_id IS NOT NULL
        UNION ALL
        SELECT m.context_id FROM mcp_artifacts m
        WHERE m.created_at >= from_at AND m.created_at < to_at AND m.context_id IS NOT NULL
        UNION ALL
        SELECT d.context_id FROM governance_decisions d
        WHERE d.created_at >= from_at AND d.created_at < to_at
    ) touched
    WHERE context_id IS NOT NULL AND context_id <> '00000000-0000-0000-0000-4c4547414359'))
$$;

-- One rollup tick: every context touched between the stored watermark and a
-- new high-water mark, then the mark moves. Consecutive ticks cover disjoint
-- windows, so a conversation is rebuilt once per change, not once per tick.
--
-- Why the mark trails the clock: a row carries the `now()` of the transaction
-- that wrote it, so a transaction still open can commit a row stamped before
-- this tick. The mark stops at the oldest open transaction's start, less a
-- margin for timestamps the application stamps before it begins one; the
-- next tick picks up whatever that transaction commits. A transaction open
-- longer than ten minutes stops holding the mark back, so one stuck session
-- cannot freeze the rollup. The state row is held FOR UPDATE, so two ticks
-- never overlap.
CREATE OR REPLACE FUNCTION refresh_conversation_facts_pending()
RETURNS BIGINT LANGUAGE plpgsql AS $$
DECLARE from_at TIMESTAMPTZ; to_at TIMESTAMPTZ; written BIGINT;
BEGIN
    INSERT INTO conversation_rollup_state(id, watermark)
    VALUES (TRUE, now() - interval '3 minutes')
    ON CONFLICT (id) DO NOTHING;
    SELECT s.watermark INTO from_at FROM conversation_rollup_state s WHERE s.id FOR UPDATE;
    SELECT LEAST(now(), GREATEST(COALESCE(MIN(a.xact_start), now()), now() - interval '10 minutes'))
           - interval '30 seconds' INTO to_at
    FROM pg_stat_activity a
    WHERE a.datname = current_database() AND a.pid <> pg_backend_pid() AND a.xact_start IS NOT NULL;
    IF to_at <= from_at THEN
        RETURN 0;
    END IF;
    written := refresh_conversation_facts_between(from_at, to_at);
    UPDATE conversation_rollup_state SET watermark = to_at, updated_at = clock_timestamp() WHERE id;
    RETURN written;
END $$;
