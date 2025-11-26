UPDATE user_sessions
SET endpoints_accessed = CASE
    WHEN endpoints_accessed = '[]' THEN jsonb_build_array($1)::text
    WHEN endpoints_accessed::jsonb ? $2 THEN endpoints_accessed
    ELSE (endpoints_accessed::jsonb || jsonb_build_array($3))::text
END
WHERE session_id = $4