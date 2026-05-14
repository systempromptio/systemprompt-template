-- Widen subagent_spawns to BIGINT on installs that pre-date the declarative
-- schema using BIGINT directly.
ALTER TABLE plugin_session_summaries ALTER COLUMN subagent_spawns TYPE BIGINT;
