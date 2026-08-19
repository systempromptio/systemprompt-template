-- Demo customers, so the enterprise console has a portfolio to show.
--
-- Run by demo/02-seed-demo-tenants.sh only. This is demo-flow data: it is
-- never installed by boot, migrations, or seeds, and a clean install has none
-- of it.
--
-- Everything is keyed to demo- ids and fixed de000000-… UUIDs and guarded by
-- ON CONFLICT, so this file is safe to re-run and never edits anyone else's
-- rows. User ids are fixed UUID literals, not gen_random_uuid(): they have to
-- be stable across every database so the console, the demos and the reports
-- can be reasoned about, and users.id is parsed as a UUID on every auth path.
-- Members carry only the `user` role: org-level ownership is expressed by
-- organization_members.org_role, and platform admin is a much larger grant
-- these fixtures must never hold.

INSERT INTO plans (id, name, description)
VALUES
    ('standard', 'Standard', 'Placeholder until plans.yaml is loaded.'),
    ('enterprise', 'Enterprise', 'Placeholder until plans.yaml is loaded.')
ON CONFLICT (id) DO NOTHING;

INSERT INTO organizations (id, slug, name, plan_id, status, email_domains, contract_start)
VALUES
    ('demo-northwind', 'northwind', 'Northwind Trading', 'enterprise', 'active',
     ARRAY['northwind.example'], NOW()::DATE - 400),
    ('demo-contoso', 'contoso', 'Contoso Retail', 'standard', 'active',
     ARRAY['contoso.example'], NOW()::DATE - 180),
    ('demo-initech', 'initech', 'Initech Systems', 'standard', 'suspended',
     ARRAY['initech.example'], NOW()::DATE - 90)
ON CONFLICT (slug) DO NOTHING;

INSERT INTO departments (id, name, description, org_id)
VALUES
    ('demo-nw-sales', 'Sales', 'Pipeline and account management', 'demo-northwind'),
    ('demo-nw-support', 'Support', 'Customer service desk', 'demo-northwind'),
    ('demo-nw-eng', 'Engineering', 'Platform and integrations', 'demo-northwind'),
    ('demo-co-sales', 'Sales', 'Retail accounts', 'demo-contoso'),
    ('demo-co-ops', 'Operations', 'Store operations', 'demo-contoso'),
    ('demo-it-eng', 'Engineering', 'Product engineering', 'demo-initech'),
    ('demo-it-finance', 'Finance', 'Billing and reporting', 'demo-initech')
ON CONFLICT DO NOTHING;

-- Members. `name` carries the email because `users.name` is unique and a
-- display name is not; `display_name` is what the console shows.
INSERT INTO users (id, name, email, display_name, status, email_verified, roles)
VALUES
    ('de000000-0000-4000-8000-000000000001', 'ava.stone@northwind.example', 'ava.stone@northwind.example', 'Ava Stone', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000002', 'ben.parr@northwind.example', 'ben.parr@northwind.example', 'Ben Parr', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000003', 'cara.dunn@northwind.example', 'cara.dunn@northwind.example', 'Cara Dunn', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000004', 'dev.rao@northwind.example', 'dev.rao@northwind.example', 'Dev Rao', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000005', 'eli.frost@northwind.example', 'eli.frost@northwind.example', 'Eli Frost', 'inactive', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000006', 'fay.oduya@contoso.example', 'fay.oduya@contoso.example', 'Fay Oduya', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000007', 'gil.mercer@contoso.example', 'gil.mercer@contoso.example', 'Gil Mercer', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000008', 'hana.vos@contoso.example', 'hana.vos@contoso.example', 'Hana Vos', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000009', 'ira.blum@initech.example', 'ira.blum@initech.example', 'Ira Blum', 'active', TRUE, ARRAY['user']),
    ('de000000-0000-4000-8000-000000000010', 'jo.kern@initech.example', 'jo.kern@initech.example', 'Jo Kern', 'active', TRUE, ARRAY['user'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_profile_ext (user_id, department)
VALUES
    ('de000000-0000-4000-8000-000000000001', 'Sales'),
    ('de000000-0000-4000-8000-000000000002', 'Sales'),
    ('de000000-0000-4000-8000-000000000003', 'Support'),
    ('de000000-0000-4000-8000-000000000004', 'Engineering'),
    ('de000000-0000-4000-8000-000000000005', 'Engineering'),
    ('de000000-0000-4000-8000-000000000006', 'Sales'),
    ('de000000-0000-4000-8000-000000000007', 'Operations'),
    ('de000000-0000-4000-8000-000000000008', 'Operations'),
    ('de000000-0000-4000-8000-000000000009', 'Engineering'),
    ('de000000-0000-4000-8000-000000000010', 'Finance')
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO organization_members (user_id, org_id, org_role)
VALUES
    ('de000000-0000-4000-8000-000000000001', 'demo-northwind', 'owner'),
    ('de000000-0000-4000-8000-000000000002', 'demo-northwind', 'member'),
    ('de000000-0000-4000-8000-000000000003', 'demo-northwind', 'member'),
    ('de000000-0000-4000-8000-000000000004', 'demo-northwind', 'member'),
    ('de000000-0000-4000-8000-000000000005', 'demo-northwind', 'member'),
    ('de000000-0000-4000-8000-000000000006', 'demo-contoso', 'owner'),
    ('de000000-0000-4000-8000-000000000007', 'demo-contoso', 'member'),
    ('de000000-0000-4000-8000-000000000008', 'demo-contoso', 'member'),
    ('de000000-0000-4000-8000-000000000009', 'demo-initech', 'owner'),
    ('de000000-0000-4000-8000-000000000010', 'demo-initech', 'member')
ON CONFLICT (user_id) DO NOTHING;

-- Thirty days of traffic per member, shaped so the three tenants land in
-- three different budget bands — or the console's health pill only ever shows
-- one colour.
INSERT INTO ai_requests (
    id, request_id, user_id, context_id, provider, model, input_tokens, output_tokens,
    tokens_used, cost_microdollars, latency_ms, status, actor_kind, actor_id,
    created_at, updated_at, completed_at
)
SELECT
    'demo-req-' || seed.user_id || '-' || g,
    'demo-req-' || seed.user_id || '-' || g,
    seed.user_id,
    '00000000-0000-0000-0000-4c4547414359',
    'anthropic',
    seed.model,
    seed.input_tokens,
    seed.output_tokens,
    seed.input_tokens + seed.output_tokens,
    seed.cost,
    900 + (g * 37) % 2600,
    'completed',
    'user',
    seed.user_id,
    NOW() - ((g % 30) || ' days')::INTERVAL - ((g * 17) % 1400 || ' minutes')::INTERVAL,
    NOW() - ((g % 30) || ' days')::INTERVAL,
    NOW() - ((g % 30) || ' days')::INTERVAL
FROM (
    VALUES
        ('de000000-0000-4000-8000-000000000001', 'claude-opus-4-5-20251101', 420000, 90000, 60000000),
        ('de000000-0000-4000-8000-000000000002', 'claude-sonnet-4-5-20250929', 180000, 42000, 25000000),
        ('de000000-0000-4000-8000-000000000003', 'claude-haiku-4-5-20251001', 90000, 21000, 8000000),
        ('de000000-0000-4000-8000-000000000004', 'claude-sonnet-4-5-20250929', 260000, 74000, 32000000),
        ('de000000-0000-4000-8000-000000000006', 'claude-sonnet-4-5-20250929', 74000, 17000, 9000000),
        ('de000000-0000-4000-8000-000000000007', 'claude-haiku-4-5-20251001', 60000, 15000, 2900000),
        ('de000000-0000-4000-8000-000000000008', 'claude-sonnet-4-5-20250929', 48000, 12000, 6000000),
        ('de000000-0000-4000-8000-000000000009', 'claude-opus-4-5-20251101', 210000, 48000, 34000000),
        ('de000000-0000-4000-8000-000000000010', 'claude-sonnet-4-5-20250929', 130000, 33000, 16000000)
) AS seed(user_id, model, input_tokens, output_tokens, cost)
CROSS JOIN generate_series(1, 120) AS g
ON CONFLICT (id) DO NOTHING;
