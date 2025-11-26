SELECT
    id,
    name,
    status,
    pid,
    restart_count,
    startup_time_ms,
    port
FROM services
WHERE name = $1 AND protocol = 'mcp'