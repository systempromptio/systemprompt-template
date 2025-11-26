UPDATE services
SET health_status = $1, last_health_check = CURRENT_TIMESTAMP
WHERE name = $2 AND protocol = 'mcp'