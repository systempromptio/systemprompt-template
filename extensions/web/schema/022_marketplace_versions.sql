-- Migration 022: Marketplace version history and changelog

CREATE TABLE IF NOT EXISTS marketplace_versions (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    version_type TEXT NOT NULL DEFAULT 'upload' CHECK (version_type IN ('upload', 'restore')),
    snapshot_path TEXT NOT NULL,
    skills_snapshot JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, version_number)
);

CREATE INDEX IF NOT EXISTS idx_marketplace_versions_user
    ON marketplace_versions(user_id, version_number DESC);

CREATE TABLE IF NOT EXISTS marketplace_changelog (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    version_id TEXT REFERENCES marketplace_versions(id) ON DELETE SET NULL,
    action TEXT NOT NULL CHECK (action IN ('added', 'updated', 'deleted', 'restored')),
    skill_id TEXT NOT NULL,
    skill_name TEXT NOT NULL DEFAULT '',
    detail TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_marketplace_changelog_user
    ON marketplace_changelog(user_id, created_at DESC);

-- Add version_type column if table already exists (idempotent)
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'marketplace_versions' AND column_name = 'version_type'
    ) THEN
        ALTER TABLE marketplace_versions
            ADD COLUMN version_type TEXT NOT NULL DEFAULT 'upload'
            CHECK (version_type IN ('upload', 'restore'));
    END IF;
END $$;
