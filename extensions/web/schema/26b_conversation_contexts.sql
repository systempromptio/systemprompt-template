-- Conversation metadata the agents member keeps in `user_contexts`: who owns
-- a context, the session that opened it, the name a user or agent gave it and
-- when it was created and last changed. Installed only when the web extension
-- is built with `agents`; 26b_conversation_contexts_unlinked declares the
-- same columns empty for every other build, so the conversation views, the
-- refresh functions and the console read one relation in every flavour.

CREATE OR REPLACE VIEW conversation_contexts AS
SELECT context_id, user_id, session_id, name, created_at, updated_at
FROM user_contexts;
