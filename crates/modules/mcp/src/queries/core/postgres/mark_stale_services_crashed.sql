UPDATE services
SET status = 'crashed', pid = NULL, last_stopped_at = CURRENT_TIMESTAMP
WHERE module_name = 'mcp' AND status = 'running'