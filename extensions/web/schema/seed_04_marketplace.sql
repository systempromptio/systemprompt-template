-- Seed: Default enterprise demo marketplace

INSERT INTO org_marketplaces (id, name, description, enabled, github_repo_url)
VALUES (
    'enterprise-demo',
    'Enterprise Demo',
    'Enterprise governance demo — a dangerous secret skill blocked by policy hooks, and a simple web search skill demonstrating tool governance.',
    true,
    'https://github.com/systempromptio/systemprompt-enterprise-demo-marketplace'
) ON CONFLICT (id) DO NOTHING;

INSERT INTO org_marketplace_plugins (marketplace_id, plugin_id, position)
VALUES ('enterprise-demo', 'enterprise-demo', 0)
ON CONFLICT DO NOTHING;
