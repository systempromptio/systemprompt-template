-- Find first active admin user by creation date
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
WHERE roles LIKE '%admin%'
  AND status = 'active'
ORDER BY created_at
LIMIT 1
