-- Consolidated schema: access control rules.
--
-- 2026-05-07 catalog refactor: removed plugin_ratings, plugin_visibility_rules,
-- plugin_installations, and plugin_installation_history tables. The marketplace
-- concept is gone — only the YAML-defined catalog (skills/plugins/MCP servers)
-- remains. RBAC entity_type now allows 'skill' and no longer allows 'marketplace'.
-- See migrations/2026-05-07-catalog-refactor.sql for the forward-only DDL applied
-- to existing databases.

-- marketplace_versions / marketplace_changelog: removed.
-- Marketplaces are now defined as YAML at services/marketplaces/<id>/config.yaml
-- and ingested by core into ServicesConfig.marketplaces at boot. There is no
-- runtime authoring surface, so version snapshots and per-user changelogs are
-- no longer needed. Drop any pre-existing tables left over from older deployments.
DROP TABLE IF EXISTS marketplace_changelog CASCADE;
DROP TABLE IF EXISTS marketplace_versions CASCADE;

CREATE TABLE IF NOT EXISTS access_control_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'skill', 'gateway_route')),
    entity_id TEXT NOT NULL,
    rule_type TEXT NOT NULL CHECK (rule_type IN ('role', 'department')),
    rule_value TEXT NOT NULL,
    access TEXT NOT NULL DEFAULT 'allow' CHECK (access IN ('allow', 'deny')),
    default_included BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(entity_type, entity_id, rule_type, rule_value)
);
CREATE INDEX IF NOT EXISTS idx_acl_entity ON access_control_rules(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_acl_rule ON access_control_rules(rule_type, rule_value);
CREATE INDEX IF NOT EXISTS idx_acl_default ON access_control_rules(default_included) WHERE default_included = true;

-- org_marketplaces / org_marketplace_plugins: removed.
-- Marketplaces are YAML-defined in services/marketplaces/. The DB no longer
-- owns marketplace definitions. See core's ServicesConfig.marketplaces.
DROP TABLE IF EXISTS org_marketplace_plugins CASCADE;
DROP TABLE IF EXISTS org_marketplaces CASCADE;

-- plugin_ratings / plugin_visibility_rules / plugin_installations /
-- plugin_installation_history: removed in catalog refactor (2026-05-07).
-- See header comment above.
DROP TABLE IF EXISTS plugin_ratings CASCADE;
DROP TABLE IF EXISTS plugin_visibility_rules CASCADE;
DROP TABLE IF EXISTS plugin_installations CASCADE;
DROP TABLE IF EXISTS plugin_installation_history CASCADE;
