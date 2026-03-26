CREATE TABLE IF NOT EXISTS hook_overrides (
    hook_id TEXT PRIMARY KEY,
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS user_hooks (
    id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL,
    hook_id TEXT NOT NULL,
    name TEXT NOT NULL DEFAULT '',
    description TEXT NOT NULL DEFAULT '',
    event TEXT NOT NULL,
    matcher TEXT NOT NULL DEFAULT '*',
    command TEXT NOT NULL,
    is_async BOOLEAN NOT NULL DEFAULT false,
    enabled BOOLEAN NOT NULL DEFAULT true,
    base_hook_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, hook_id)
);
CREATE INDEX IF NOT EXISTS idx_user_hooks_user ON user_hooks(user_id);
CREATE INDEX IF NOT EXISTS idx_user_hooks_base ON user_hooks(base_hook_id);
