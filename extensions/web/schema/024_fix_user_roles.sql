UPDATE users
SET department = COALESCE(NULLIF(department, ''), roles[1]),
    roles = ARRAY['user']
WHERE NOT (roles @> ARRAY['anonymous']::TEXT[])
  AND NOT (roles @> ARRAY['admin']::TEXT[])
  AND array_length(roles, 1) = 1
  AND roles[1] NOT IN ('user', 'admin', 'anonymous', 'a2a', 'mcp', 'service');

UPDATE users
SET roles = array_remove(roles, r.bad_role),
    department = COALESCE(NULLIF(department, ''), r.bad_role)
FROM (
    SELECT u.id, unnest(u.roles) AS bad_role
    FROM users u
) r
WHERE users.id = r.id
  AND r.bad_role NOT IN ('user', 'admin', 'anonymous', 'a2a', 'mcp', 'service');

UPDATE users
SET roles = CASE
    WHEN NOT (roles @> ARRAY['user']::TEXT[])
    THEN array_append(roles, 'user')
    ELSE roles
END
WHERE NOT (roles @> ARRAY['anonymous']::TEXT[])
  AND array_length(roles, 1) > 0;
