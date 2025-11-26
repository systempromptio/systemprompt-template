-- List users with optional search filter
--
-- Parameters:
--   $1: search_term (TEXT, optional) - Searches across name, email, and full_name
--
-- Returns: All non-deleted users matching the search criteria with session data
--
-- Examples:
--   - No filter: Pass NULL for $1 to get all users
--   - With filter: Pass "john" to find users with "john" in any searchable field
--
-- Note: ILIKE is case-insensitive in PostgreSQL
SELECT
    u.id as uuid,
    u.name,
    u.email,
    u.full_name,
    u.display_name,
    u.status,
    u.email_verified,
    u.roles,
    u.avatar_url,
    u.created_at,
    u.updated_at,
    COALESCE(COUNT(DISTINCT s.session_id), 0) AS total_sessions,
    MAX(s.last_activity_at) AS last_active
FROM users u
LEFT JOIN user_sessions s ON u.id = s.user_id
WHERE u.status NOT IN ('deleted', 'temporary')
  AND (
    $1::TEXT IS NULL
    OR u.name ILIKE '%' || $1::TEXT || '%'
    OR u.email ILIKE '%' || $1::TEXT || '%'
    OR u.full_name ILIKE '%' || $1::TEXT || '%'
  )
GROUP BY u.id, u.name, u.email, u.full_name, u.display_name,
         u.status, u.email_verified, u.roles, u.avatar_url,
         u.created_at, u.updated_at
ORDER BY u.created_at DESC