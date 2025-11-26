UPDATE services
SET status = 'stopped', pid = NULL
WHERE name = $1 AND module_name = 'agent'