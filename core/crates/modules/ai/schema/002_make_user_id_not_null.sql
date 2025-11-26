-- Migration: Make user_id NOT NULL on ai_requests
-- AI requests must be attributed to users for cost tracking and analytics
BEGIN;

ALTER TABLE ai_requests
ALTER COLUMN user_id SET NOT NULL;

COMMIT;
