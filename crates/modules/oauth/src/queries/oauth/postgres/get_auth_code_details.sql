SELECT user_id, scope, expires_at, redirect_uri, used_at,
       code_challenge, code_challenge_method
FROM oauth_auth_codes
WHERE code = $1 AND client_id = $2
