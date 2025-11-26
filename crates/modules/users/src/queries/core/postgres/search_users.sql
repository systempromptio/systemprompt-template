-- Search users with pagination
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
WHERE 
    (name LIKE '%' || $1 || '%' OR 
     email LIKE '%' || $1 || '%' OR 
     full_name LIKE '%' || $1 || '%')
    AND status != 'deleted'
ORDER BY name ASC
LIMIT $2 OFFSET $3