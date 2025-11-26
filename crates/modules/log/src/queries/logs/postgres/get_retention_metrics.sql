SELECT
    COUNT(*) as total_logs,
    CAST(COUNT(*) * 100.0 / MAX(COUNT(*)) OVER() AS INTEGER) as usage_percentage,
    MIN(created_at) as oldest_log,
    MAX(created_at) as newest_log,
    COUNT(CASE WHEN level = 'ERROR' THEN 1 END) as error_count,
    COUNT(CASE WHEN level = 'WARN' THEN 1 END) as warn_count
FROM logs
