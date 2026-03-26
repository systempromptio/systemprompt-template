-- Migration 043: Seed default enterprise demo marketplace
-- Accessible to all users by default (no access control rules)

INSERT INTO org_marketplaces (id, name, description, enabled)
VALUES (
    'enterprise-demo',
    'Enterprise Demo',
    'Default marketplace with HTTP hooks, security review, compliance checks, and developer tools. Synced from github.com/systempromptio/systemprompt-enterprise-demo-marketplace.',
    true
) ON CONFLICT (id) DO NOTHING;
