ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS plugin_id TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT DEFAULT 0;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT DEFAULT 0;
