SELECT client_id FROM oauth_auth_codes WHERE code = $1 AND expires_at > $2
