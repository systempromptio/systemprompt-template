-- The planes another member owns, as every composition sees them.
--
-- The console's conversation record, the last-seen roster and raw-evidence
-- expiry read three planes whose tables belong to optional members: the
-- tool ledger and the artifact store (mcp), named contexts (agents) and
-- marketplace session bindings (marketplace). Each plane is one view or
-- function here, declared over nothing, so a composition without the
-- member installs and reads an empty plane. The member's own file
-- (50_mcp_planes.sql, 51_agent_planes.sql) replaces it with the real
-- definition; the columns are identical, so every view and function built
-- on a plane keeps working when the member is linked.

-- Every tool call with the artifact rule applied (46_tool_artifacts.sql).
CREATE OR REPLACE VIEW tool_activity AS
SELECT NULL::character varying(255) AS ai_tool_call_id,
       NULL::text AS intent_id,
       NULL::character varying(255) AS request_id,
       NULL::text AS mcp_execution_id,
       NULL::character varying(255) AS artifact_id,
       NULL::character varying(255) AS user_id,
       NULL::character varying(255) AS session_id,
       NULL::character varying(255) AS context_id,
       NULL::character varying(255) AS trace_id,
       NULL::text AS client_kind,
       NULL::character varying(255) AS tool_name,
       NULL::character varying(255) AS server_name,
       NULL::timestamp with time zone AS intended_at,
       NULL::timestamp with time zone AS executed_at,
       NULL::timestamp with time zone AS completed_at,
       NULL::integer AS execution_time_ms,
       NULL::character varying(255) AS execution_status,
       NULL::text AS error_message,
       NULL::character varying(32) AS source,
       NULL::character varying(16) AS correlation,
       NULL::character varying(100) AS artifact_type,
       NULL::character varying(500) AS artifact_title,
       NULL::boolean AS is_structured,
       NULL::boolean AS has_ui_resource,
       NULL::boolean AS is_error,
       NULL::integer AS payload_bytes,
       NULL::integer AS secret_redactions,
       NULL::text AS state,
       NULL::timestamp with time zone AS occurred_at,
       NULL::character varying(255) AS execution_context_id,
       NULL::character varying(255) AS execution_trace_id,
       NULL::character(64) AS payload_sha256,
       NULL::text AS artifact_kind,
       NULL::text AS input_summary,
       NULL::boolean AS is_builtin
WHERE false;

-- Each tool execution and artifact as one touch: who, which context, the
-- harness trace it ran under, when it was written and, for an execution,
-- when it started.
CREATE OR REPLACE VIEW tool_plane_touches AS
SELECT NULL::text AS user_id, NULL::text AS context_id, NULL::text AS trace_id,
       NULL::timestamptz AS touched_at, NULL::timestamptz AS seen_at
WHERE false;

-- A conversation context as the agent member records it: its owner, the
-- session it was opened in, its name and when it was created and changed.
CREATE OR REPLACE VIEW conversation_contexts AS
SELECT NULL::text AS context_id, NULL::text AS user_id, NULL::text AS session_id,
       NULL::text AS name, NULL::timestamptz AS created_at, NULL::timestamptz AS updated_at
WHERE false;

-- The member-owned rows raw-evidence expiry removes (32_raw_retention.sql).
CREATE OR REPLACE FUNCTION expire_member_raw_evidence(cutoff TIMESTAMPTZ)
RETURNS void LANGUAGE plpgsql AS $$
BEGIN
END $$;
