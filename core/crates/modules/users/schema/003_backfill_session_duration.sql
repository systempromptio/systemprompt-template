-- Backfill NULL duration_seconds with calculated values based on session timestamps
-- This migration fixes sessions that were ended but never had duration calculated

BEGIN;

-- Update sessions with ended_at but NULL duration_seconds
-- Use the ended_at timestamp to calculate duration
UPDATE user_sessions
SET duration_seconds = EXTRACT(EPOCH FROM (ended_at - started_at))::INTEGER
WHERE ended_at IS NOT NULL
  AND duration_seconds IS NULL
  AND ended_at >= started_at;

-- Update active sessions (ended_at IS NULL)
-- Use last_activity_at as the session end point for duration calculation
UPDATE user_sessions
SET duration_seconds = EXTRACT(EPOCH FROM (last_activity_at - started_at))::INTEGER
WHERE ended_at IS NULL
  AND duration_seconds IS NULL
  AND last_activity_at >= started_at;

-- Handle any remaining edge cases where duration might be negative (timestamps invalid)
-- Set to 0 for invalid durations to prevent breaking analytics queries
UPDATE user_sessions
SET duration_seconds = 0
WHERE duration_seconds IS NULL
  OR duration_seconds < 0;

COMMIT;
