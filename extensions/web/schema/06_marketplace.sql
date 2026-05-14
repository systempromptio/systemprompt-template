-- Access control rules table.
--
-- 2026-05-07 catalog refactor: removed plugin_ratings, plugin_visibility_rules,
-- plugin_installations, plugin_installation_history, marketplace_versions,
-- marketplace_changelog, org_marketplaces, and org_marketplace_plugins.
-- Marketplaces are now defined as YAML at services/marketplaces/<id>/config.yaml
-- and ingested by core into ServicesConfig.marketplaces at boot. RBAC entity_type
-- allows 'skill' and 'gateway_route'; 'marketplace' is gone.
-- Drops of legacy tables live in migrations/006_drop_legacy_marketplace.sql.

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
