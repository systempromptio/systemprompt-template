-- Module Management Schema for SystemPrompt OS
-- Tracks installed modules and their versions
CREATE TABLE IF NOT EXISTS modules (
    id TEXT PRIMARY KEY,
    name VARCHAR(255) NOT NULL UNIQUE,
    version TEXT NOT NULL,
    display_name VARCHAR(255) NOT NULL,
    description TEXT,
    weight INTEGER DEFAULT 100,
    -- Module metadata from module.yaml (stored as JSON)
    schemas TEXT,
    seeds TEXT,
    permissions TEXT,
    -- Control flags
    enabled BOOLEAN DEFAULT TRUE,
    -- Timestamps
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);
-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_modules_name ON modules(name);
CREATE INDEX IF NOT EXISTS idx_modules_enabled ON modules(enabled);
CREATE INDEX IF NOT EXISTS idx_modules_weight ON modules(weight);
-- Trigger for automatic timestamp updates (handled at application level)
