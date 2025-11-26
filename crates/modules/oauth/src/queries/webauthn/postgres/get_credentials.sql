SELECT id, user_id, credential_id, public_key, counter,
       display_name, device_type, created_at, last_used_at
FROM webauthn_credentials
WHERE user_id = $1
ORDER BY created_at DESC
