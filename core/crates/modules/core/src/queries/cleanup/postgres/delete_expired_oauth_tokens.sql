DELETE FROM oauth_refresh_tokens
WHERE created_at < NOW() - INTERVAL '90 days'
