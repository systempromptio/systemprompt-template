SELECT * FROM user_sessions
WHERE fingerprint_hash = $1
ORDER BY started_at DESC
LIMIT 1
