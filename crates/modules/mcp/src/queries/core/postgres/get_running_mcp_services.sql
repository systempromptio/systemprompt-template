SELECT name, pid FROM services
WHERE module_name = 'mcp' AND status = 'running' AND pid IS NOT NULL