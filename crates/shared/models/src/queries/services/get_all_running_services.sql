SELECT name, module_name, status, pid, port, created_at, updated_at FROM services WHERE status = 'running' ORDER BY name
