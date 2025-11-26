DELETE FROM oauth_refresh_tokens WHERE expires_at < $1
