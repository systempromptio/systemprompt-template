-- Groups, projects, their AD-group mappings and manual role grants.
--
-- Declarative twin of migration 041: a fresh install creates these tables
-- here, an existing one is migrated there. Both must stay identical, so every
-- statement is IF NOT EXISTS / CREATE OR REPLACE.
--
-- Membership and manual role grants carry a validity window (migration 088):
-- `valid_from` / `valid_until` bound when the row counts, `revoked_at` is
-- stamped by the hourly expiry sweep so the row stays as the audit trail. A
-- row is live when
--     revoked_at IS NULL AND valid_from <= now()
--     AND (valid_until IS NULL OR valid_until > now())
-- and every resolver reads through that predicate — the `user_groups` and
-- `user_projects` views below carry it so consumers inherit it.

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
    source TEXT NOT NULL CHECK (source IN ('adfs','manual')),
    source_ad_group TEXT,
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    PRIMARY KEY (group_id, user_id, source)
);
CREATE INDEX IF NOT EXISTS idx_group_members_valid_until ON group_members(valid_until)
    WHERE valid_until IS NOT NULL AND revoked_at IS NULL;
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
    source TEXT NOT NULL CHECK (source IN ('adfs','manual')),
    source_ad_group TEXT,
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMPTZ,
    revoked_at TIMESTAMPTZ,
    PRIMARY KEY (project_id, user_id, source)
);
CREATE INDEX IF NOT EXISTS idx_project_members_valid_until ON project_members(valid_until)
    WHERE valid_until IS NOT NULL AND revoked_at IS NULL;
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
    role TEXT NOT NULL CHECK (role IN ('platform_admin','admin','developer','user','project_manager','knowledge_worker','super_admin')),
    granted_by TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    valid_until TIMESTAMPTZ,
    PRIMARY KEY (user_id, role)
);
CREATE INDEX IF NOT EXISTS idx_user_manual_roles_valid_until ON user_manual_roles(valid_until)
    WHERE valid_until IS NOT NULL;

CREATE OR REPLACE VIEW user_groups AS
SELECT DISTINCT gm.user_id, gm.group_id FROM group_members gm
WHERE gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
  AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP)
UNION ALL
SELECT u.id AS user_id, 'unassigned' AS group_id
FROM users u
WHERE NOT ('anonymous' = ANY(u.roles))
  AND NOT EXISTS (
      SELECT 1 FROM group_members gm WHERE gm.user_id = u.id
        AND gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
        AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP));

CREATE OR REPLACE VIEW user_projects AS
SELECT DISTINCT pm.user_id, pm.project_id FROM project_members pm
WHERE pm.revoked_at IS NULL AND pm.valid_from <= CURRENT_TIMESTAMP
  AND (pm.valid_until IS NULL OR pm.valid_until > CURRENT_TIMESTAMP);
