INSERT INTO webauthn_credentials
(id, user_id, credential_id, public_key, counter, display_name, device_type, transports)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
