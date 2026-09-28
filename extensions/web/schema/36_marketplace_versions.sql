-- A marketplace version is its content hash: sha256 over the marketplace's
-- own config, every plugin it includes and every skill those plugins ship,
-- hashed the way a bundle is hashed. Recorded at every boot and every
-- inventory sync from the composed services root, so a base marketplace and
-- a bundled one carry the same kind of identity. One row per (marketplace,
-- hash); the row with `effective_until IS NULL` is the version being served.
-- `manifest` holds per-plugin and per-skill digests so two versions can be
-- diffed without re-reading either tree. The `legacy_source_hash` origin is
-- reserved for versions reconstructed from facts that predate this table and
-- carry the coarser source hash in place of a marketplace hash.
--
-- Declarative twin of migration 085.

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
