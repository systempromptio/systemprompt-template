DELETE FROM logs l
WHERE l.session_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM user_sessions us
    WHERE us.session_id = l.session_id
  )
