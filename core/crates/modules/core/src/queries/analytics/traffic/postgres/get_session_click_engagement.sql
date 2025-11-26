WITH session_clicks AS (
    SELECT
        us.session_id,
        COUNT(lc.id) as click_count
    FROM user_sessions us
    LEFT JOIN link_clicks lc ON us.session_id = lc.session_id
    WHERE us.started_at >= CURRENT_TIMESTAMP - ($1 || ' days')::INTERVAL
      AND us.is_bot = FALSE
      AND us.is_scanner = FALSE
    GROUP BY us.session_id
)
SELECT
    COUNT(*)::INTEGER as total_sessions,
    SUM(CASE WHEN click_count > 0 THEN 1 ELSE 0 END)::INTEGER as sessions_with_clicks,
    CAST(ROUND(
        100.0 * SUM(CASE WHEN click_count > 0 THEN 1 ELSE 0 END) / NULLIF(COUNT(*), 0),
        2
    ) AS FLOAT) as click_engagement_rate,
    CAST(ROUND(AVG(click_count), 2) AS FLOAT) as avg_clicks_per_session,
    SUM(CASE WHEN click_count >= 3 THEN 1 ELSE 0 END)::INTEGER as high_intent_sessions
FROM session_clicks
