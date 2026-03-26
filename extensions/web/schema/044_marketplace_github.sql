-- Migration 044: GitHub integration for org marketplaces

ALTER TABLE org_marketplaces ADD COLUMN IF NOT EXISTS github_repo_url TEXT;

-- Backfill the existing enterprise-demo marketplace
UPDATE org_marketplaces
SET github_repo_url = 'https://github.com/systempromptio/systemprompt-enterprise-demo-marketplace'
WHERE id = 'enterprise-demo' AND github_repo_url IS NULL;

-- Sync/publish audit log
CREATE TABLE IF NOT EXISTS github_marketplace_sync_log (
    id BIGSERIAL PRIMARY KEY,
    marketplace_id TEXT NOT NULL REFERENCES org_marketplaces(id) ON DELETE CASCADE,
    action TEXT NOT NULL CHECK (action IN ('sync', 'publish')),
    status TEXT NOT NULL CHECK (status IN ('started', 'success', 'error')),
    commit_hash TEXT,
    plugin_count INT NOT NULL DEFAULT 0,
    error_count INT NOT NULL DEFAULT 0,
    error_message TEXT,
    triggered_by TEXT,
    duration_ms BIGINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_gms_marketplace ON github_marketplace_sync_log(marketplace_id);
CREATE INDEX IF NOT EXISTS idx_gms_created ON github_marketplace_sync_log(created_at DESC);
