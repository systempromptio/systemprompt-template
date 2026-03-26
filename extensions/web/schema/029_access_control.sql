-- Migration 029: Unified access control rules for plugins, agents, and MCP servers

CREATE TABLE IF NOT EXISTS access_control_rules (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    entity_type TEXT NOT NULL CHECK (entity_type IN ('plugin', 'agent', 'mcp_server')),
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

-- Seed from existing plugin_visibility_rules
INSERT INTO access_control_rules (id, entity_type, entity_id, rule_type, rule_value, access, created_at)
SELECT id, 'plugin', plugin_id, rule_type, rule_value, access, created_at
FROM plugin_visibility_rules
ON CONFLICT DO NOTHING;
