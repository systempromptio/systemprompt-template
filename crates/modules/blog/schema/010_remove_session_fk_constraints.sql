-- ============================================================================
-- Migration: Remove session_id foreign key constraints
-- ============================================================================
-- Problem: Bot sessions (bot_*) and untracked sessions are not inserted into
-- user_sessions table, causing FK violations when:
-- 1. Bots click links -> link_clicks FK violation
-- 2. Analytics events are logged -> analytics_events FK violation
--
-- Solution: Remove FK constraints from both tables to allow tracking without
-- requiring sessions to exist. Sessions can still be joined for analytics.
-- ============================================================================

BEGIN;

-- Remove FK constraint from link_clicks
ALTER TABLE link_clicks
DROP CONSTRAINT IF EXISTS link_clicks_session_id_fkey;

-- Remove FK constraint from analytics_events
ALTER TABLE analytics_events
DROP CONSTRAINT IF EXISTS analytics_events_session_id_fkey;

-- Note: We keep the session_id columns and indexes for analytics joins
-- The data can still be queried by joining on session_id where sessions exist

COMMIT;
