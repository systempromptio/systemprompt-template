-- Conversation metadata the agents member keeps in `user_contexts`: the name a
-- user or agent gave a context and when it last changed. Installed only when
-- the web extension is built with `agents`; 26b_conversation_contexts_unlinked
-- declares the same columns empty for every other build, so the conversation
-- views and refresh functions read one relation in every flavour.

CREATE OR REPLACE VIEW conversation_contexts AS
SELECT context_id, name, updated_at
FROM user_contexts;
