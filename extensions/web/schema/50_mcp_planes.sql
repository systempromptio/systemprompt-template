-- The mcp and marketplace members' planes (declared empty in
-- 20_member_planes.sql), installed only when those members are linked.

-- Every tool call with the artifact rule applied. The execution's own
-- context and trace ids are exposed beside the ledger's coalesced ones
-- because the conversation record joins hook-sourced executions through the
-- harness session (`trace_id`), which the ledger overwrites with the
-- request's trace when an intent row exists.
CREATE OR REPLACE VIEW tool_activity AS
SELECT l.ai_tool_call_id, l.intent_id, l.request_id, l.mcp_execution_id, l.artifact_id,
       l.user_id, l.session_id, l.context_id, l.trace_id, l.client_kind,
       l.tool_name, l.server_name, l.intended_at, l.executed_at, l.completed_at,
       l.execution_time_ms, l.execution_status, l.error_message, l.source, l.correlation,
       l.artifact_type, l.artifact_title, l.is_structured, l.has_ui_resource, l.is_error,
       l.payload_bytes, l.secret_redactions, l.state, l.occurred_at,
       x.context_id AS execution_context_id,
       x.trace_id AS execution_trace_id,
       COALESCE(a.payload_sha256, x.payload_sha256) AS payload_sha256,
       artifact_kind(l.tool_name, l.artifact_type, l.has_ui_resource, l.is_structured,
                     COALESCE(a.payload_sha256, x.payload_sha256), x.input, b.is_builtin) AS artifact_kind,
       tool_input_summary(x.input) AS input_summary,
       b.is_builtin
FROM tool_call_ledger l
LEFT JOIN mcp_tool_executions x ON x.mcp_execution_id = l.mcp_execution_id
LEFT JOIN mcp_artifacts a ON a.artifact_id = l.artifact_id
CROSS JOIN LATERAL (SELECT (l.server_name IS NOT NULL AND l.server_name = l.source
                            AND COALESCE(l.tool_name, '') NOT LIKE 'mcp\_\_%') AS is_builtin) b;

CREATE OR REPLACE VIEW tool_plane_touches AS
SELECT x.user_id::text AS user_id, x.context_id::text AS context_id, x.trace_id::text AS trace_id,
       x.created_at AS touched_at, x.started_at AS seen_at
FROM mcp_tool_executions x
UNION ALL
SELECT m.user_id::text, m.context_id::text, NULL::text, m.created_at, NULL::timestamptz
FROM mcp_artifacts m;

-- A binding is kept while its session still has hook events newer than the
-- cutoff.
CREATE OR REPLACE FUNCTION expire_member_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS void LANGUAGE plpgsql AS $$
BEGIN
    DELETE FROM managed_consumer_session_bindings b WHERE bound_at<cutoff
        AND NOT EXISTS(SELECT 1 FROM plugin_usage_events e WHERE e.user_id=b.consumer_id AND e.session_id=b.native_session_id AND e.created_at>=cutoff);
END $$;
