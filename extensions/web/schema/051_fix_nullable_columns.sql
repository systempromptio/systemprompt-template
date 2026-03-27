-- Migration 051: Drop NOT NULL on nullable columns in plugin_usage_events
-- prompt_preview, description, and cwd are passed as Option<&str> from Rust
-- and are legitimately NULL for many event types.
-- Migration 048 attempted to fix this with ADD COLUMN IF NOT EXISTS (nullable),
-- but that was a no-op since the columns already existed from 046 (NOT NULL).

ALTER TABLE plugin_usage_events ALTER COLUMN prompt_preview DROP NOT NULL;
ALTER TABLE plugin_usage_events ALTER COLUMN description DROP NOT NULL;
ALTER TABLE plugin_usage_events ALTER COLUMN cwd DROP NOT NULL;
