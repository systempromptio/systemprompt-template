-- Conversation metadata for a build without the agents member: no member
-- keeps named contexts, so the relation is empty. Same columns as
-- 26b_conversation_contexts.sql, which replaces it when agents is linked.

CREATE OR REPLACE VIEW conversation_contexts AS
SELECT NULL::text AS context_id, NULL::text AS user_id, NULL::text AS session_id,
       NULL::text AS name, NULL::timestamptz AS created_at, NULL::timestamptz AS updated_at
WHERE false;
