-- Shared classification: request labels take precedence over thread shape.
CREATE OR REPLACE FUNCTION conversation_request_kind(kind text, has_tools boolean, thread_size bigint)
RETURNS text LANGUAGE sql IMMUTABLE PARALLEL SAFE AS $function$
SELECT CASE WHEN kind = 'probe' THEN 'probe' WHEN kind = 'utility' THEN 'utility'
            WHEN NOT has_tools AND thread_size = 1 THEN 'utility' ELSE 'turn' END
$function$;

-- One row per gateway request, classified for the conversation pages.
--
-- Rule: a side call is a probe, or a single-request thread that offered no
-- tools. Everything else is a turn. `effective_kind` is one of
-- 'turn', 'probe', 'utility'.
--
-- A thread is one `gateway_conversation_id` inside a context (sub-agents and
-- post-compaction histories get their own). The legacy sentinel context that
-- pools every context-less request is excluded here so no page has to.
CREATE OR REPLACE VIEW conversation_requests AS
WITH threads AS (
    SELECT context_id, gateway_conversation_id, COUNT(*)::bigint AS thread_requests
    FROM ai_requests
    GROUP BY context_id, gateway_conversation_id
)
SELECT r.id, r.user_id, r.session_id, r.client_session_id, r.context_id,
       r.gateway_conversation_id, r.trace_id, r.provider, r.model, r.status,
       r.max_tokens, r.input_tokens, r.output_tokens, r.cost_microdollars,
       r.latency_ms, r.created_at, r.completed_at,
       conversation_request_kind(r.request_kind, p.offered_tools IS NOT NULL, t.thread_requests) AS effective_kind
FROM ai_requests r
LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
JOIN threads t ON t.context_id = r.context_id
   AND t.gateway_conversation_id IS NOT DISTINCT FROM r.gateway_conversation_id
WHERE r.context_id <> '00000000-0000-0000-0000-4c4547414359';

-- Numeric summaries have no prompt/message dependency. An explicit context set
-- lets detail and scoped list callers aggregate complete conversations without
-- walking unrelated history. NULL means all contexts; an empty array means none.
CREATE OR REPLACE FUNCTION conversation_metrics_for(context_ids text[])
RETURNS TABLE (
    context_id text, user_id text, display_name text, session_id text,
    client_session_id text, group_name text, project_name text, model text,
    turn_count bigint, side_call_count bigint, side_call_cost_microdollars bigint,
    tool_call_count bigint, error_count bigint, total_input_tokens bigint,
    total_output_tokens bigint, total_cost_microdollars bigint,
    first_at timestamptz, last_at timestamptz, status text
) LANGUAGE sql STABLE AS $function$
WITH scoped AS (
    SELECT ar.id, ar.context_id, ar.gateway_conversation_id, ar.user_id,
           ar.session_id, ar.client_session_id, ar.model, ar.request_kind,
           ar.status, ar.input_tokens, ar.output_tokens, ar.cost_microdollars, ar.created_at,
           COUNT(*) OVER (PARTITION BY ar.context_id, ar.gateway_conversation_id) AS thread_size
    FROM ai_requests ar
    WHERE ar.context_id <> '00000000-0000-0000-0000-4c4547414359'
      AND (context_ids IS NULL OR ar.context_id IN (SELECT unnest(context_ids)))
), requests AS MATERIALIZED (
    SELECT r.*, conversation_request_kind(r.request_kind, p.offered_tools IS NOT NULL, r.thread_size) AS effective_kind
    FROM scoped r LEFT JOIN ai_request_payloads p ON p.ai_request_id = r.id
), agg AS (
    SELECT r.context_id,
        COUNT(*) FILTER (WHERE effective_kind = 'turn')::bigint AS turn_count,
        COUNT(*) FILTER (WHERE effective_kind <> 'turn')::bigint AS side_call_count,
        COALESCE(SUM(cost_microdollars) FILTER (WHERE effective_kind <> 'turn'), 0)::bigint AS side_call_cost_microdollars,
        COUNT(*) FILTER (WHERE status = 'failed')::bigint AS error_count,
        COALESCE(SUM(input_tokens), 0)::bigint AS total_input_tokens,
        COALESCE(SUM(output_tokens), 0)::bigint AS total_output_tokens,
        COALESCE(SUM(cost_microdollars), 0)::bigint AS total_cost_microdollars,
        MIN(created_at) AS first_at, MAX(created_at) AS last_request_at,
        (ARRAY_AGG(user_id ORDER BY created_at DESC, id DESC))[1] AS user_id,
        (ARRAY_AGG(session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE session_id IS NOT NULL))[1] AS session_id,
        (ARRAY_AGG(client_session_id ORDER BY created_at DESC, id DESC)
            FILTER (WHERE client_session_id IS NOT NULL))[1] AS client_session_id,
        (ARRAY_AGG(model ORDER BY created_at DESC, id DESC)
            FILTER (WHERE effective_kind = 'turn' AND model IS NOT NULL))[1] AS model
    FROM requests r GROUP BY r.context_id
), tools AS (
    SELECT r.context_id, COUNT(*)::bigint AS tool_call_count
    FROM requests r JOIN ai_request_tool_calls t ON t.request_id = r.id
    WHERE r.effective_kind = 'turn' GROUP BY r.context_id
)
SELECT a.context_id, a.user_id, u.display_name, a.session_id, a.client_session_id,
       g.name, p.name, a.model, a.turn_count, a.side_call_count,
       a.side_call_cost_microdollars, COALESCE(t.tool_call_count, 0), a.error_count,
       a.total_input_tokens, a.total_output_tokens, a.total_cost_microdollars,
       a.first_at, GREATEST(a.last_request_at, c.updated_at), s.status
FROM agg a
LEFT JOIN tools t ON t.context_id = a.context_id
LEFT JOIN user_contexts c ON c.context_id = a.context_id
LEFT JOIN plugin_session_summaries s ON s.session_id = a.client_session_id
LEFT JOIN users u ON u.id = a.user_id
LEFT JOIN user_scope_defaults d ON d.user_id = a.user_id
LEFT JOIN groups g ON g.id = d.primary_group_id
LEFT JOIN projects p ON p.id = d.primary_project_id
$function$;

-- Parse only the first matching message, never every prompt in history.
CREATE OR REPLACE FUNCTION conversation_opening_prompt(context_key text, max_characters integer)
RETURNS text LANGUAGE sql STABLE AS $function$
SELECT LEFT(NULLIF(BTRIM(regexp_replace(regexp_replace(regexp_replace(
    split_part(m.content, '=== ASSISTANT ANSWER ===', 1),
    '=== USER PROMPT ===', '', 'g'), '<system-reminder>.*?</system-reminder>', '', 'g'),
    '\s+', ' ', 'g')), ''), max_characters)
FROM (
    SELECT m.content FROM conversation_requests cr
    JOIN ai_request_messages m ON m.request_id = cr.id
    WHERE cr.context_id = context_key AND cr.effective_kind = 'turn' AND m.role = 'user'
    ORDER BY cr.created_at, m.sequence_number, cr.id LIMIT 1
) m
$function$;

-- COALESCE avoids reading messages when a generated title/name is available.
CREATE OR REPLACE FUNCTION conversation_title(context_key text, client_session_key text)
RETURNS text LANGUAGE sql STABLE AS $function$
SELECT COALESCE(
    (SELECT s.ai_title FROM plugin_session_summaries s WHERE s.session_id = client_session_key),
    (SELECT NULLIF(c.name, 'Gateway conversation') FROM user_contexts c WHERE c.context_id = context_key),
    conversation_opening_prompt(context_key, 160),
    'Conversation ' || LEFT(context_key, 12) || '…')
$function$;

-- Compatibility view for existing callers. New page queries select metrics
-- first and call conversation_title only for search candidates/displayed rows.
CREATE OR REPLACE VIEW conversation_rollups AS
SELECT r.context_id::varchar(255) AS context_id, conversation_title(r.context_id, r.client_session_id) AS title,
       r.user_id::varchar AS user_id, r.display_name::varchar(255) AS display_name,
       r.session_id::varchar AS session_id, r.client_session_id,
       r.group_name, r.project_name, r.model, r.turn_count, r.side_call_count,
       r.side_call_cost_microdollars, r.tool_call_count, r.error_count,
       r.total_input_tokens, r.total_output_tokens, r.total_cost_microdollars,
       r.first_at, r.last_at, r.status
FROM conversation_metrics_for(NULL) r;
