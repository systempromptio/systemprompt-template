INSERT INTO services (name, module_name, status, port, binary_mtime)
VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (name) DO UPDATE SET
  module_name = EXCLUDED.module_name,
  status = EXCLUDED.status,
  port = EXCLUDED.port,
  binary_mtime = EXCLUDED.binary_mtime,
  updated_at = CURRENT_TIMESTAMP
