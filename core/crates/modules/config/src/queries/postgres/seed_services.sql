-- Seed data for services table
-- Note: Only referencing existing modules
INSERT OR IGNORE INTO services (id, name, module_name, display_name, description, version, host, port, protocol, status) VALUES
(
    '550e8400-e29b-41d4-a716-446655440000',
    'mcp',
    'mcp',
    'MCP Service Manager',
    'Model Context Protocol server management service',
    '0.1.0',
    '127.0.0.1',
    3001,
    'mcp',
    'stopped'
);