INSERT INTO users (
    id, name, email, full_name, display_name,
    status, email_verified, roles,
    created_at, updated_at
) VALUES (
    $1, $2, $3, $4, COALESCE($5, $6),
    'active', false, ARRAY['user']::TEXT[], $7, $8
)
RETURNING
    id as uuid, name, email, full_name, display_name,
    status, email_verified, roles, avatar_url,
    created_at, updated_at