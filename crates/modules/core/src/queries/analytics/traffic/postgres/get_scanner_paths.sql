WITH scanner_endpoints AS (
    SELECT
        session_id,
        json_array_elements_text(endpoints_accessed::json) AS endpoint
    FROM user_sessions
    WHERE started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
      AND is_scanner = TRUE
      AND endpoints_accessed IS NOT NULL
      AND endpoints_accessed != '[]'
      AND endpoints_accessed != ''
)
SELECT
    endpoint AS path,
    COUNT(DISTINCT session_id) AS session_count,
    COUNT(*) AS hit_count
FROM scanner_endpoints
GROUP BY endpoint
ORDER BY session_count DESC, hit_count DESC
LIMIT 20
