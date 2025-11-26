SELECT * FROM user_sessions
WHERE fingerprint_hash = $1
AND user_id IS NULL
AND last_activity_at >= datetime('now', '-' || $2 || ' seconds')
ORDER BY last_activity_at DESC
LIMIT 1
