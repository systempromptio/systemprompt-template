SELECT
    client_id,
    COUNT(*) as error_count,
    COUNT(DISTINCT session_id) as affected_sessions,
    MAX(error_message) as last_error
FROM oauth_errors
GROUP BY client_id
ORDER BY error_count DESC
