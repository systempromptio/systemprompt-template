UPDATE services
SET status = 'error'
WHERE module_name = 'mcp' AND status = 'running' AND pid IS NULL