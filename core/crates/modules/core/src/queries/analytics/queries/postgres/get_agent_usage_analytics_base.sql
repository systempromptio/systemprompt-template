SELECT
    json_extract(t.metadata, '$.agent_id') as agent_id,
    COUNT(*) as total_tasks,
    COUNT(DISTINCT t.user_id) as unique_users,
    COUNT(DISTINCT t.session_id) as unique_sessions,
    SUM(json_extract(t.metadata, '$.message_count')) as total_messages,
    AVG(JULIANDAY(t.updated_at) - JULIANDAY(t.created_at)) * 86400 as avg_completion_time_seconds,
    SUM(CASE WHEN t.status = 'failed' THEN 1 ELSE 0 END) as failed_tasks,
    SUM(CASE WHEN t.status = 'completed' THEN 1 ELSE 0 END) as completed_tasks
FROM agent_tasks t
WHERE t.created_at >= datetime('now', '-' || $1 || ' days')