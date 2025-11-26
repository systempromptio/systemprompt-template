SELECT id, pid, status FROM services
WHERE name = $1 AND module_name = 'mcp'