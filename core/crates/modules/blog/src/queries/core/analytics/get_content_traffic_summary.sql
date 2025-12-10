SELECT
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '1 day'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as traffic_1d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '7 days'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as traffic_7d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '30 days'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as traffic_30d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '2 days'
            AND ae.timestamp < CURRENT_TIMESTAMP - INTERVAL '1 day'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as prev_traffic_1d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '14 days'
            AND ae.timestamp < CURRENT_TIMESTAMP - INTERVAL '7 days'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as prev_traffic_7d,
    COUNT(DISTINCT CASE
        WHEN ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '60 days'
            AND ae.timestamp < CURRENT_TIMESTAMP - INTERVAL '30 days'
            AND u.is_bot = FALSE
            AND u.is_scanner = FALSE
        THEN ae.user_id
    END) as prev_traffic_30d
FROM analytics_events ae
LEFT JOIN users u ON ae.user_id = u.id
WHERE ae.event_type = 'page_view'
  AND ae.event_category = 'content'
  AND ae.timestamp >= CURRENT_TIMESTAMP - INTERVAL '60 days'
