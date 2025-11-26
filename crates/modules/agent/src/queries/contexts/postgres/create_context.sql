-- Only insert session_id if it exists in user_sessions, otherwise set to NULL
-- This prevents FK violations when session_id is invalid
INSERT INTO user_contexts (context_id, user_id, session_id, name, created_at, updated_at)
VALUES (
  $1,
  $2,
  CASE
    WHEN $3 IS NOT NULL AND EXISTS (SELECT 1 FROM user_sessions WHERE session_id = $3)
    THEN $3
    ELSE NULL
  END,
  $4,
  CURRENT_TIMESTAMP,
  CURRENT_TIMESTAMP
)
