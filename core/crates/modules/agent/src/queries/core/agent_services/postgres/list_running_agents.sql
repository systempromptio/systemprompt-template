SELECT
    name
FROM services
WHERE module_name = 'agent'
  AND pid IS NOT NULL
  AND status = 'running'