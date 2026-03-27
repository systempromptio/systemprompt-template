-- Migration 052: Fix unique index on plugin_usage_daily to match ON CONFLICT clause
-- The upsert_daily_aggregation function uses:
--   ON CONFLICT (date, user_id, event_type, COALESCE(tool_name, ''))
-- But the existing unique index includes COALESCE(plugin_id, '') which the code
-- no longer references (plugin_id is not part of the insert).
-- Drop the old index and create one matching the code.

DROP INDEX IF EXISTS idx_usage_daily_unique;

CREATE UNIQUE INDEX idx_usage_daily_unique
    ON plugin_usage_daily(date, user_id, event_type, COALESCE(tool_name, ''));
