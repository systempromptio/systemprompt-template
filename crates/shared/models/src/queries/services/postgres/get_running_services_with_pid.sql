SELECT name, module_name, status, pid, port, binary_mtime FROM services WHERE status = 'running' AND pid IS NOT NULL
