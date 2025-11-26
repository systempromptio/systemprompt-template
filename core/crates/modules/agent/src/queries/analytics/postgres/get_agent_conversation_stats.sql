SELECT
    COUNT(*) AS total_count,
    AVG(COALESCE(EXTRACT(EPOCH FROM (completed_at - created_at)), 0)) AS avg_duration_seconds,
    SUM(CASE WHEN status = 'completed' THEN 1 ELSE 0 END) * 100.0 / COUNT(*) AS success_rate
FROM agent_tasks
WHERE created_at > CURRENT_TIMESTAMP - INTERVAL '7 days'
