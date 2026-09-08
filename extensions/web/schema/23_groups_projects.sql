-- Groups, projects, their AD-group mappings and manual role grants.
--
-- Declarative twin of migration 041: a fresh install creates these tables
-- here, an existing one is migrated there. Both must stay identical, so every
-- statement is IF NOT EXISTS / CREATE OR REPLACE.

CREATE TABLE IF NOT EXISTS groups (
    id TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9][a-z0-9_-]{0,63}$'),
    name TEXT NOT NULL,
    description TEXT,
    is_system BOOLEAN NOT NULL DEFAULT false,
    source TEXT NOT NULL DEFAULT 'dashboard' CHECK (source IN ('yaml','dashboard','system')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE OR REPLACE FUNCTION groups_protect_system() RETURNS trigger AS $$
BEGIN
    IF OLD.is_system THEN
        RAISE EXCEPTION 'group % is a system group and cannot be deleted', OLD.id;
    END IF;
    RETURN OLD;
END
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_groups_protect_system ON groups;
CREATE TRIGGER trg_groups_protect_system BEFORE DELETE ON groups
    FOR EACH ROW EXECUTE FUNCTION groups_protect_system();

CREATE TABLE IF NOT EXISTS group_members (
    group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK (source IN ('adfs','manual','odoo')),
    source_ad_group TEXT,
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (group_id, user_id, source)
);
CREATE INDEX IF NOT EXISTS idx_group_members_user ON group_members(user_id);

CREATE TABLE IF NOT EXISTS group_ad_mappings (
    ad_group TEXT NOT NULL,
    group_id TEXT NOT NULL REFERENCES groups(id) ON DELETE CASCADE,
    source TEXT NOT NULL DEFAULT 'dashboard' CHECK (source IN ('yaml','dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ad_group, group_id)
);

CREATE TABLE IF NOT EXISTS projects (
    id TEXT PRIMARY KEY CHECK (id ~ '^[a-z0-9][a-z0-9_-]{0,63}$'),
    name TEXT NOT NULL,
    description TEXT,
    source TEXT NOT NULL DEFAULT 'dashboard' CHECK (source IN ('yaml','dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS project_members (
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    source TEXT NOT NULL CHECK (source IN ('adfs','manual','odoo')),
    source_ad_group TEXT,
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (project_id, user_id, source)
);
CREATE INDEX IF NOT EXISTS idx_project_members_user ON project_members(user_id);

CREATE TABLE IF NOT EXISTS project_ad_mappings (
    ad_group TEXT NOT NULL,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    source TEXT NOT NULL DEFAULT 'dashboard' CHECK (source IN ('yaml','dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (ad_group, project_id)
);

CREATE TABLE IF NOT EXISTS user_manual_roles (
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    role TEXT NOT NULL,
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (user_id, role)
);

CREATE OR REPLACE VIEW user_groups AS
SELECT DISTINCT gm.user_id, gm.group_id FROM group_members gm
UNION ALL
SELECT u.id AS user_id, 'unassigned' AS group_id
FROM users u
WHERE NOT ('anonymous' = ANY(u.roles))
  AND NOT EXISTS (SELECT 1 FROM group_members gm WHERE gm.user_id = u.id);

