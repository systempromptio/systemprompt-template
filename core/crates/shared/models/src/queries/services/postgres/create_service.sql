INSERT INTO services (name, module_name, status, port)
VALUES ($1, $2, $3, $4)
ON CONFLICT (name) DO UPDATE SET
  module_name = EXCLUDED.module_name,
  status = EXCLUDED.status,
  port = EXCLUDED.port,
  updated_at = CURRENT_TIMESTAMP
