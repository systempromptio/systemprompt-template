UPDATE webauthn_credentials
SET counter = $1, last_used_at = $2
WHERE credential_id = $3
