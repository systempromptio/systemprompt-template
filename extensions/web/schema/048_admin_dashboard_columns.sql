-- Migration 048: Add missing columns referenced by admin Rust code
-- Uses ADD COLUMN IF NOT EXISTS for idempotency.

-- ============================================================
-- 1. user_hooks — columns expected by Rust UserHook struct and CRUD queries
--    Original table (028) has: id, user_id, hook_id, name, description, event,
--    matcher, command, is_async, enabled, base_hook_id.
--    Rust code references: hook_name, event_type, hook_type, url, headers,
--    timeout, is_default, plugin_id.
-- ============================================================

-- hook_name (String in Rust, NOT NULL)
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS hook_name TEXT NOT NULL DEFAULT '';

-- event_type (String in Rust, NOT NULL) — replaces old "event" column semantically
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS event_type TEXT NOT NULL DEFAULT '';

-- hook_type (String in Rust, NOT NULL, default 'http')
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS hook_type TEXT NOT NULL DEFAULT 'http';

-- url (String in Rust, NOT NULL)
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS url TEXT NOT NULL DEFAULT '';

-- headers (serde_json::Value in Rust = JSONB, NOT NULL)
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS headers JSONB NOT NULL DEFAULT '{}';

-- timeout (i32 in Rust, NOT NULL)
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS timeout INT NOT NULL DEFAULT 10;

-- is_default (bool in Rust, NOT NULL)
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS is_default BOOLEAN NOT NULL DEFAULT false;

-- plugin_id (Option<String> in Rust = nullable TEXT)
-- Already added by 046, but ensure it exists.
ALTER TABLE user_hooks ADD COLUMN IF NOT EXISTS plugin_id TEXT;

-- Backfill hook_name from old "name" column and event_type from old "event" column
-- (only where the new columns are still empty defaults)
UPDATE user_hooks SET hook_name = name WHERE hook_name = '' AND name IS NOT NULL AND name != '';
UPDATE user_hooks SET event_type = event WHERE event_type = '' AND event IS NOT NULL AND event != '';

-- Index for querying hooks by event_type (used by ensure_default_hooks)
CREATE INDEX IF NOT EXISTS idx_user_hooks_event_type ON user_hooks(event_type);
CREATE INDEX IF NOT EXISTS idx_user_hooks_default ON user_hooks(user_id, is_default) WHERE is_default = true;

-- ============================================================
-- 2. plugin_session_summaries — columns from 046 re-stated for
--    environments where 046 has not yet been applied.
--    All use ADD COLUMN IF NOT EXISTS for idempotency.
-- ============================================================
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS status TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS unique_files_touched INT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT NOT NULL DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT NOT NULL DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_title TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_summary TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_tags TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS ai_description TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS apm REAL;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS eapm REAL;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS peak_concurrent INT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS permission_mode TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS client_source TEXT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS subagent_spawns BIGINT NOT NULL DEFAULT 0;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS user_prompts INT;
ALTER TABLE plugin_session_summaries ADD COLUMN IF NOT EXISTS automated_actions INT;

-- ============================================================
-- 3. plugin_usage_events — columns from 046 re-stated
-- ============================================================
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS description TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS prompt_preview TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS cwd TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS dedup_key TEXT;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT NOT NULL DEFAULT 0;
ALTER TABLE plugin_usage_events ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT NOT NULL DEFAULT 0;

CREATE UNIQUE INDEX IF NOT EXISTS idx_pue_dedup ON plugin_usage_events(dedup_key) WHERE dedup_key IS NOT NULL;

-- ============================================================
-- 4. plugin_usage_daily — columns from 046 re-stated
-- ============================================================
ALTER TABLE plugin_usage_daily ADD COLUMN IF NOT EXISTS content_input_bytes BIGINT NOT NULL DEFAULT 0;
ALTER TABLE plugin_usage_daily ADD COLUMN IF NOT EXISTS content_output_bytes BIGINT NOT NULL DEFAULT 0;
