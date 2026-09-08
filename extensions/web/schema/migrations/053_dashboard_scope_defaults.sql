-- One primary group and one primary project per user: the attribution key
-- that stops a person in several groups being counted in full in each.
--
-- Declarative twin of migration 042. `source` records who decided: 'auto' is
-- the recomputation job's answer and may be overwritten by it, 'manual' is an
-- operator's and never is.

CREATE TABLE IF NOT EXISTS user_scope_defaults (
    user_id TEXT PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    primary_group_id TEXT REFERENCES groups(id) ON DELETE SET NULL,
    primary_project_id TEXT REFERENCES projects(id) ON DELETE SET NULL,
    source TEXT NOT NULL DEFAULT 'auto' CHECK (source IN ('auto', 'manual')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX IF NOT EXISTS idx_user_scope_defaults_group
    ON user_scope_defaults(primary_group_id);
CREATE INDEX IF NOT EXISTS idx_user_scope_defaults_project
    ON user_scope_defaults(primary_project_id);
