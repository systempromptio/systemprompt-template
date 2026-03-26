-- Migration 031: Org-level marketplaces (collections of plugins)

CREATE TABLE IF NOT EXISTS org_marketplaces (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS org_marketplace_plugins (
    marketplace_id TEXT NOT NULL REFERENCES org_marketplaces(id) ON DELETE CASCADE,
    plugin_id TEXT NOT NULL,
    position INT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (marketplace_id, plugin_id)
);

CREATE INDEX IF NOT EXISTS idx_omp_marketplace ON org_marketplace_plugins(marketplace_id);
CREATE INDEX IF NOT EXISTS idx_omp_plugin ON org_marketplace_plugins(plugin_id);

-- Add 'marketplace' to access_control_rules entity_type CHECK
ALTER TABLE access_control_rules DROP CONSTRAINT IF EXISTS access_control_rules_entity_type_check;
ALTER TABLE access_control_rules ADD CONSTRAINT access_control_rules_entity_type_check
    CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'marketplace'));
