UPDATE services
SET status = 'error'
WHERE name = $1 AND module_name = 'agent'