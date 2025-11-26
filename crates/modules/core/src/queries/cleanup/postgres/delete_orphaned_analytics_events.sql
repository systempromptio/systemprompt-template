DELETE FROM analytics_events ae
WHERE ae.session_id IS NOT NULL
  AND NOT EXISTS (
    SELECT 1 FROM user_sessions us
    WHERE us.session_id = ae.session_id
  )
