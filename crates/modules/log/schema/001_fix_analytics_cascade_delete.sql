-- Migration: Fix analytics_events CASCADE DELETE to preserve historical data
-- This prevents session deletion from destroying analytics event history
-- Analytics events are historical facts that must persist even after sessions expire
BEGIN;

-- Drop the existing foreign key constraint with CASCADE DELETE
ALTER TABLE analytics_events
DROP CONSTRAINT IF EXISTS analytics_events_session_id_fkey;

-- Recreate the foreign key constraint with SET NULL instead of CASCADE
-- This allows analytics events to persist even after their session is deleted
ALTER TABLE analytics_events
ADD CONSTRAINT analytics_events_session_id_fkey
    FOREIGN KEY (session_id)
    REFERENCES user_sessions(session_id)
    ON DELETE SET NULL;

COMMIT;
