INSERT INTO users (id, name, email, full_name, display_name, status, email_verified, roles)
VALUES ($1, $2, $3, $4, $5, $6, false, $7::TEXT[])
ON CONFLICT (id) DO NOTHING
