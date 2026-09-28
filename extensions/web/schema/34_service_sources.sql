-- Which services source shipped each plugin, marketplace and skill, and that
-- source's content hash — written at every boot from the active composition
-- (the base tree and every pinned bundle) so the feedback projection can
-- stamp an invocation with the published tree its skill ran from.
--
-- Declarative twin of migration 084. `service_sources` is one row per source
-- (`base`, `bundle:<name>`); `service_owned_ids` is one row per owned id,
-- decided by composition: a bundle owns what its manifest says, the base
-- owns everything else.

CREATE TABLE IF NOT EXISTS service_sources (
    name TEXT PRIMARY KEY,
    kind TEXT NOT NULL CHECK (kind IN ('base', 'bundle')),
    content_hash TEXT,
    digest TEXT,
    version TEXT,
    provenance TEXT NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS service_owned_ids (
    kind TEXT NOT NULL CHECK (kind IN ('marketplace', 'plugin', 'skill')),
    id TEXT NOT NULL,
    source TEXT NOT NULL REFERENCES service_sources(name) ON DELETE CASCADE,
    marketplace_id TEXT,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (kind, id)
);

CREATE INDEX IF NOT EXISTS idx_service_owned_ids_source ON service_owned_ids(source);
