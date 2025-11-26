INSERT INTO oauth_auth_codes (
    code, client_id, user_id, redirect_uri, scope,
    expires_at, code_challenge, code_challenge_method
) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
