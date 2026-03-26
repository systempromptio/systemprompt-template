ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS dedup_key TEXT;
CREATE UNIQUE INDEX IF NOT EXISTS idx_plugin_usage_dedup ON plugin_usage_events(dedup_key) WHERE dedup_key IS NOT NULL;
