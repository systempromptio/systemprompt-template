SELECT id, name, host, port, status
FROM services
WHERE status = 'running' AND protocol = 'mcp'