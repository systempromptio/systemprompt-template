-- Twin of schema/42_gateway_routes.sql: the gateway routing table (with its
-- human-facing name and description) and the staged governance chain, both
-- seeded from their file by the governance bootstrap on the next boot (empty
-- table = seed; otherwise drift is reported on /admin/sync and nothing is
-- written).

CREATE TABLE IF NOT EXISTS gateway_routes (
    id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    name TEXT,
    description TEXT,
    model_pattern TEXT NOT NULL,
    provider TEXT NOT NULL,
    upstream_model TEXT,
    extra_headers JSONB NOT NULL DEFAULT '{}'::jsonb,
    pricing JSONB,
    when_match JSONB,
    requires JSONB,
    fallback_provider TEXT,
    fallback_upstream_model TEXT,
    explicit_id BOOLEAN NOT NULL DEFAULT FALSE,
    source TEXT NOT NULL DEFAULT 'code' CHECK (source IN ('code', 'dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_gateway_routes_position ON gateway_routes(position);

CREATE TABLE IF NOT EXISTS governance_chain (
    policy_id TEXT PRIMARY KEY,
    position INTEGER NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    mode TEXT NOT NULL CHECK (mode IN ('enforce', 'warn')),
    entry JSONB NOT NULL,
    source TEXT NOT NULL DEFAULT 'code' CHECK (source IN ('code', 'dashboard')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- The chain's own switch and inherited mode (`governance.enabled`,
-- `governance.mode`), one row.
CREATE TABLE IF NOT EXISTS governance_chain_settings (
    singleton BOOLEAN PRIMARY KEY DEFAULT TRUE CHECK (singleton),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    mode TEXT NOT NULL CHECK (mode IN ('enforce', 'warn')),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
