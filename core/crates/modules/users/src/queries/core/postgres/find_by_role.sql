-- Find first active user with specified role
SELECT
    id as uuid,
    name,
    email,
    full_name,
    display_name,
    status,
    email_verified,
    roles,
    avatar_url,
    created_at,
    updated_at
FROM users
WHERE $1 = ANY(roles)
  AND status = 'active'
ORDER BY created_at
LIMIT 1
