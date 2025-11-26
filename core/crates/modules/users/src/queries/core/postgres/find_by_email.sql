SELECT
    id as uuid, name, email, full_name, display_name,
    status, email_verified, roles, avatar_url,
    created_at, updated_at
FROM users 
WHERE email = $1 AND status != 'deleted'