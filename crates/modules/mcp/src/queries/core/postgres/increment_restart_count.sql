UPDATE services
SET restart_count = restart_count + 1
WHERE name = $1 AND protocol = 'mcp'