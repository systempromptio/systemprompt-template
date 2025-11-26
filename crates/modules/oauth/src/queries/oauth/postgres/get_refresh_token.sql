SELECT user_id, scope, expires_at FROM oauth_refresh_tokens WHERE token_id = $1 AND client_id = $2
