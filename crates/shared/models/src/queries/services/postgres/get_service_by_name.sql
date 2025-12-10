SELECT name, module_name, status, pid, port, binary_mtime, created_at, updated_at
FROM services
WHERE name = $1
