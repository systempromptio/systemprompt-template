CREATE TABLE IF NOT EXISTS roles (
    name VARCHAR(50) PRIMARY KEY,
    description TEXT,
    is_default BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_roles_is_default ON roles(is_default);

INSERT INTO roles (name, description, is_default) VALUES
    ('user', 'Standard user role with basic permissions', true),
    ('admin', 'Administrator role with full permissions', false),
    ('anonymous', 'Anonymous user role for unauthenticated users', false)
ON CONFLICT (name) DO NOTHING;
