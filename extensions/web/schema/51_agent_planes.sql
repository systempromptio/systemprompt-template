-- The agent member's plane (declared empty in 20_member_planes.sql),
-- installed only when the agent member is linked.

CREATE OR REPLACE VIEW conversation_contexts AS
SELECT c.context_id::text AS context_id, c.user_id::text AS user_id, c.session_id::text AS session_id,
       c.name::text AS name, c.created_at, c.updated_at
FROM user_contexts c;
