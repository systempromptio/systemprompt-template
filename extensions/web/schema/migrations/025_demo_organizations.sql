-- Demo customers, so the enterprise console has a portfolio to show.
--
-- A cross-customer view with one row proves nothing: the seat bars, the budget
-- bands, and the margin column only mean anything next to each other. These
-- three tenants are deliberately different shapes — one large enterprise well
-- inside its cap, one standard customer approaching it, and one suspended
-- customer whose inference cost has overrun what its licence pays.
--
-- Everything here is keyed to `demo-` ids and guarded by ON CONFLICT, so this
-- migration is safe to re-run and safe to leave applied on a real instance:
-- it adds rows, it never edits anyone else's.
--
-- The organizations themselves are also declared in
-- services/access-control/plans.yaml. They are inserted here as well because
-- the bootstrap loader runs as a startup job, after migrations, and this file
-- needs the rows to exist now. The loader then owns them: it will correct the
-- name, plan, domains, and status on the next boot, and project the plan's
-- grants into access_control_rules, which is why none are written here.

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
    ('demo-nw-1', 'ava.stone@northwind.example', 'ava.stone@northwind.example', 'Ava Stone', 'active', TRUE, ARRAY['user', 'admin']),
    ('demo-nw-2', 'ben.parr@northwind.example', 'ben.parr@northwind.example', 'Ben Parr', 'active', TRUE, ARRAY['user']),
    ('demo-nw-3', 'cara.dunn@northwind.example', 'cara.dunn@northwind.example', 'Cara Dunn', 'active', TRUE, ARRAY['user']),
    ('demo-nw-4', 'dev.rao@northwind.example', 'dev.rao@northwind.example', 'Dev Rao', 'active', TRUE, ARRAY['user']),
    ('demo-nw-5', 'eli.frost@northwind.example', 'eli.frost@northwind.example', 'Eli Frost', 'inactive', TRUE, ARRAY['user']),
    ('demo-co-1', 'fay.oduya@contoso.example', 'fay.oduya@contoso.example', 'Fay Oduya', 'active', TRUE, ARRAY['user', 'admin']),
    ('demo-co-2', 'gil.mercer@contoso.example', 'gil.mercer@contoso.example', 'Gil Mercer', 'active', TRUE, ARRAY['user']),
    ('demo-co-3', 'hana.vos@contoso.example', 'hana.vos@contoso.example', 'Hana Vos', 'active', TRUE, ARRAY['user']),
    ('demo-it-1', 'ira.blum@initech.example', 'ira.blum@initech.example', 'Ira Blum', 'active', TRUE, ARRAY['user', 'admin']),
    ('demo-it-2', 'jo.kern@initech.example', 'jo.kern@initech.example', 'Jo Kern', 'active', TRUE, ARRAY['user'])
ON CONFLICT (id) DO NOTHING;

INSERT INTO user_profile_ext (user_id, department)
VALUES
    ('demo-nw-1', 'Sales'),
    ('demo-nw-2', 'Sales'),
    ('demo-nw-3', 'Support'),
    ('demo-nw-4', 'Engineering'),
    ('demo-nw-5', 'Engineering'),
    ('demo-co-1', 'Sales'),
    ('demo-co-2', 'Operations'),
    ('demo-co-3', 'Operations'),
    ('demo-it-1', 'Engineering'),
    ('demo-it-2', 'Finance')
ON CONFLICT (user_id) DO NOTHING;

INSERT INTO organization_members (user_id, org_id, org_role)
VALUES
    ('demo-nw-1', 'demo-northwind', 'owner'),
    ('demo-nw-2', 'demo-northwind', 'member'),
    ('demo-nw-3', 'demo-northwind', 'member'),
    ('demo-nw-4', 'demo-northwind', 'member'),
    ('demo-nw-5', 'demo-northwind', 'member'),
    ('demo-co-1', 'demo-contoso', 'owner'),
    ('demo-co-2', 'demo-contoso', 'member'),
    ('demo-co-3', 'demo-contoso', 'member'),
    ('demo-it-1', 'demo-initech', 'owner'),
    ('demo-it-2', 'demo-initech', 'member')
ON CONFLICT (user_id) DO NOTHING;

-- Gateway traffic, spread over the last 30 days so both the rolling window and
-- the month-to-date figure are populated. Cost per request differs per tenant
-- so the three land in three different budget bands against the plan caps in
-- plans.yaml (standard $2500, enterprise $50000), because a dashboard whose
-- health pill only ever shows one colour has not been seen working:
--
--   Northwind  ~$15k of a $50k cap   — healthy, and profitable at $75k/mo
--   Contoso    ~$2.1k of a $2.5k cap — at risk, still profitable at $4k/mo
--   Initech    ~$6k of a $2.5k cap   — over cap and past its licence fee, so
--                                      its margin is negative. Suspended,
--                                      which is what stopped the overrun.
INSERT INTO ai_requests (
    id, request_id, user_id, context_id, provider, model, input_tokens, output_tokens,
    tokens_used, cost_microdollars, latency_ms, status, actor_kind, actor_id,
    created_at, updated_at, completed_at
)
SELECT
    'demo-req-' || seed.user_id || '-' || g,
    'demo-req-' || seed.user_id || '-' || g,
    seed.user_id,
    md5(seed.user_id)::uuid,
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
        ('demo-nw-1', 'claude-opus-4-5-20251101', 420000, 90000, 60000000),
        ('demo-nw-2', 'claude-sonnet-4-5-20250929', 180000, 42000, 25000000),
        ('demo-nw-3', 'claude-haiku-4-5-20251001', 90000, 21000, 8000000),
        ('demo-nw-4', 'claude-sonnet-4-5-20250929', 260000, 74000, 32000000),
        ('demo-co-1', 'claude-sonnet-4-5-20250929', 74000, 17000, 9000000),
        ('demo-co-2', 'claude-haiku-4-5-20251001', 60000, 15000, 2900000),
        ('demo-co-3', 'claude-sonnet-4-5-20250929', 48000, 12000, 6000000),
        ('demo-it-1', 'claude-opus-4-5-20251101', 210000, 48000, 34000000),
        ('demo-it-2', 'claude-sonnet-4-5-20250929', 130000, 33000, 16000000)
) AS seed(user_id, model, input_tokens, output_tokens, cost)
CROSS JOIN generate_series(1, 120) AS g
ON CONFLICT (id) DO NOTHING;
