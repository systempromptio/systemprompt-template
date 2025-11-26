SELECT
    u.id,
    u.name,
    u.email,
    u.roles,
    u.status,
    u.created_at,
    COUNT(DISTINCT s.id) as total_sessions,
    MAX(s.last_activity_at) as last_active
FROM users u
LEFT JOIN user_sessions s ON u.id = s.user_id
GROUP BY u.id, u.name, u.email, u.roles, u.status, u.created_at
ORDER BY u.created_at DESC
LIMIT $1 OFFSET $2
