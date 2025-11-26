-- Migration: Make user_id NOT NULL on historical tables
-- Users are the primary entity for attribution, not sessions
-- This enforces that all analytics must be attributed to a user (even anonymous)
BEGIN;

-- Make user_id required on analytics_events
-- All analytics events must be attributed to a user
ALTER TABLE analytics_events
ALTER COLUMN user_id SET NOT NULL;

-- Note: user_contexts.user_id is already NOT NULL ✅
-- Note: ai_requests will be handled in its own migration

COMMIT;
