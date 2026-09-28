-- Service sources (base + pinned bundles, with content hashes) and the ids
-- each one owns, written at boot. Twin of schema/34_service_sources.sql.

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
