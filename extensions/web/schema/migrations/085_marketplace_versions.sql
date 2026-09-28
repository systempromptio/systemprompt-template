-- Marketplace content hash as version identity: one row per (marketplace,
-- hash), the open row being the version served, plus the lookup that answers
-- which version was live at an instant. Twin of
-- schema/36_marketplace_versions.sql. Nothing is seeded: this repo has no
-- earlier invocation facts carrying a source hash to reconstruct from.

CREATE TABLE IF NOT EXISTS marketplace_versions (
    marketplace_id TEXT NOT NULL,
    content_hash TEXT NOT NULL,
    source TEXT NOT NULL,
    source_hash TEXT,
    manifest JSONB,
    plugin_count INTEGER NOT NULL DEFAULT 0,
    skill_count INTEGER NOT NULL DEFAULT 0,
    first_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    effective_until TIMESTAMPTZ,
    origin TEXT NOT NULL DEFAULT 'manifest' CHECK (origin IN ('manifest', 'legacy_source_hash')),
    PRIMARY KEY (marketplace_id, content_hash)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_marketplace_versions_current
    ON marketplace_versions(marketplace_id) WHERE effective_until IS NULL;
CREATE INDEX IF NOT EXISTS idx_marketplace_versions_interval
    ON marketplace_versions(marketplace_id, first_seen_at, effective_until);

-- The version a marketplace was serving at an instant: the row whose
-- interval contains it, else the newest row that had started by then, so a
-- fact projected before the table existed still resolves.
CREATE OR REPLACE FUNCTION marketplace_version_at(mid TEXT, at TIMESTAMPTZ)
RETURNS TEXT LANGUAGE sql STABLE AS $$
SELECT content_hash FROM marketplace_versions
WHERE marketplace_id = mid AND first_seen_at <= at
ORDER BY (effective_until IS NULL OR at < effective_until) DESC, first_seen_at DESC
LIMIT 1
$$;
