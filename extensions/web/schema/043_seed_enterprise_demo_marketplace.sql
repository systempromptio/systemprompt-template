-- Migration 043: Seed default enterprise demo marketplace
-- Accessible to all users by default (no access control rules)

INSERT INTO org_marketplaces (id, name, description, enabled)
VALUES (
    'enterprise-demo',
    'Enterprise Demo',
    'Enterprise governance demo — a dangerous secret skill blocked by policy hooks, and a simple web search skill demonstrating tool governance.',
    true
) ON CONFLICT (id) DO NOTHING;

-- Add the enterprise-demo plugin to the marketplace
INSERT INTO org_marketplace_plugins (marketplace_id, plugin_id, position)
VALUES ('enterprise-demo', 'enterprise-demo', 0)
ON CONFLICT DO NOTHING;
