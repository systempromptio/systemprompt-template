-- The agent member's plane (declared empty in 20_member_planes.sql),
-- installed only when the agent member is linked.

CREATE OR REPLACE VIEW conversation_contexts AS
SELECT c.context_id::text AS context_id, c.name::text AS name, c.updated_at
FROM user_contexts c;
