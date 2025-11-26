SELECT name, module_name, status, pid, port FROM services WHERE status = 'running' AND pid IS NOT NULL
