-- ============================================================================
-- Migration: Fix link_clicks session_id foreign key constraint
-- ============================================================================
-- Problem: Foreign key constraint requires session to exist before recording
-- clicks, but redirect requests may not have sessions created yet.
--
-- Solution: Drop the strict FK constraint and recreate without CASCADE,
-- allowing clicks to be recorded even if session doesn't exist yet.
-- ============================================================================

BEGIN;

-- Drop the existing foreign key constraint
ALTER TABLE link_clicks
DROP CONSTRAINT IF EXISTS link_clicks_session_id_fkey;

-- Add back as optional (no FK constraint)
-- This allows clicks to be recorded without requiring session to exist
-- Sessions will be created later, and we can still join on session_id for analytics

COMMIT;
