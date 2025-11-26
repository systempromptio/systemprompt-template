SELECT
    id,
    name,
    status,
    pid,
    restart_count,
    startup_time_ms,
    port
FROM services
WHERE protocol = 'mcp' AND status = 'running'