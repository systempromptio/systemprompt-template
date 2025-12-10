-- System configuration variables schema for SystemPrompt OS
-- This table stores all system-wide configuration variables
CREATE TABLE IF NOT EXISTS variables (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,           -- 'database_url', 'admin_port', 'log_level', 'admin_host'
    value TEXT,                          -- Variable value (stored as string, cast as needed)
    type VARCHAR(255) NOT NULL,                  -- 'string', 'integer', 'boolean', 'json'
    description TEXT,                    -- Human-readable description
    category TEXT DEFAULT 'system',     -- 'system', 'database', 'web', 'security', 'logging'
    is_secret BOOLEAN DEFAULT FALSE,    -- Whether this is sensitive data (passwords, keys)
    is_required BOOLEAN DEFAULT TRUE,   -- Whether this variable is required for system operation
    default_value TEXT,                 -- Default value if not set
    -- Lifecycle tracking
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    CONSTRAINT type_check CHECK (type IN ('string', 'integer', 'boolean', 'json')),
    CONSTRAINT category_check CHECK (category IN ('system', 'database', 'web', 'security', 'logging', 'module'))
);
-- Indexes for fast lookups
CREATE INDEX IF NOT EXISTS idx_variables_name ON variables(name);
CREATE INDEX IF NOT EXISTS idx_variables_category ON variables(category);
CREATE INDEX IF NOT EXISTS idx_variables_required ON variables(is_required);
-- Timestamp updates handled at application level