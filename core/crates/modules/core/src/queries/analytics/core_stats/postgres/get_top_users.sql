SELECT
    s.user_id as label,
    SUM(s.request_count) as value,
    ROUND(AVG(s.success_rate) * 100, 1) || '% success' as badge,
    ROW_NUMBER() OVER (ORDER BY SUM(s.request_count) DESC) as rank
FROM user_sessions s
WHERE s.last_activity_at >= datetime('now', '-' || $1 || ' days')
    AND s.user_id IS NOT NULL
GROUP BY s.user_id
ORDER BY value DESC
LIMIT $2;
