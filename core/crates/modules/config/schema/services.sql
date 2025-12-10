-- Service registry schema for SystemPrompt OS
-- LEAN: Only runtime state, config comes from services.yaml
CREATE TABLE IF NOT EXISTS services (
    name TEXT PRIMARY KEY,
    module_name VARCHAR(255) NOT NULL CHECK (module_name IN ('agent', 'mcp', 'api')),
    -- Runtime state only
    status VARCHAR(255) NOT NULL CHECK (status IN ('starting', 'running', 'stopped', 'error')) DEFAULT 'stopped',
    pid INTEGER DEFAULT NULL,
    port INTEGER NOT NULL,
    -- Binary version tracking for automatic restarts on rebuild
    binary_mtime BIGINT DEFAULT NULL,
    -- Minimal timestamps
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_services_status ON services(status);
CREATE INDEX IF NOT EXISTS idx_services_module ON services(module_name);
CREATE INDEX IF NOT EXISTS idx_services_pid ON services(pid);
