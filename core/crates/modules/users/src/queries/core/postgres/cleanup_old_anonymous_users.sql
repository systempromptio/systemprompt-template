DELETE FROM users WHERE status = $1 AND created_at < NOW() - INTERVAL '30 days'
