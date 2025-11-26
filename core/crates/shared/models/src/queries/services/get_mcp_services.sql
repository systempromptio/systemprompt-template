SELECT name, module_name, status, pid, port, created_at, updated_at
FROM services
WHERE module_name = 'mcp'
ORDER BY name
