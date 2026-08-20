-- AI-authored lines of code, measured at hook ingest from Edit/Write tool
-- inputs before sanitize_metadata truncates them. Declarative files
-- 05_plugin_usage.sql / 07_analytics.sql carry the same columns for fresh
-- installs; this migration converges established databases.

ALTER TABLE plugin_usage_events
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;

ALTER TABLE plugin_usage_daily
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;

ALTER TABLE plugin_session_summaries
    ADD COLUMN IF NOT EXISTS loc_added BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS loc_removed BIGINT NOT NULL DEFAULT 0;
