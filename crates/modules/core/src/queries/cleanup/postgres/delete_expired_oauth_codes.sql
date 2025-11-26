DELETE FROM oauth_auth_codes
WHERE created_at < NOW() - INTERVAL '30 days'
