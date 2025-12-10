SELECT
    client_id,
    COUNT(*) as error_count,
    COUNT(DISTINCT session_id) as affected_sessions,
    MAX(error_message) as last_error
FROM oauth_errors
WHERE client_id = $1
GROUP BY client_id
