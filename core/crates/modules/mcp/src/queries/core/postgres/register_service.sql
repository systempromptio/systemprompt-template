INSERT INTO services (
    id, name, module_name, display_name, description, version, host, port,
    protocol, status, pid, startup_time_ms, auth, created_at, updated_at
)
VALUES (
    $1, $2, 'mcp', $3, '', '1.0.0', '127.0.0.1', $4,
    'mcp', 'running', $5, $6, $7, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP
)
ON CONFLICT (id) DO UPDATE SET
    name = $2,
    display_name = $3,
    port = $4,
    status = 'running',
    pid = $5,
    startup_time_ms = $6,
    auth = $7,
    updated_at = CURRENT_TIMESTAMP