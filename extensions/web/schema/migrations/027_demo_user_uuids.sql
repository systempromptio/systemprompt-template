-- Give the demo fixture users real UUIDs, and stop them being platform admins.
--
-- 025 seeded them with readable string ids (`demo-nw-1`, `demo-co-1`, …).
-- `users.id` is TEXT so the database accepted that happily, but the auth layer
-- does not: core parses the id as a UUID on every path that matters —
--
--   domain/oauth/services/providers.rs      "user_id {:?} is not a valid UUID"
--   domain/oauth/services/jwt/authentication.rs   Uuid::parse_str -> 401
--   domain/oauth/services/jwt/authorization.rs    Uuid::parse_str -> 401
--   entry/api/routes/oauth/endpoints/callback.rs
--
-- so a string-id user can never authenticate, and anything minting for one
-- fails. That was not theoretical: three of them carried ARRAY['user','admin'],
-- which put them in the admin block of scripts/select-user.sh, so on a fresh
-- clone `demo/00-preflight.sh` picked `demo-nw-1` and died on
-- `issue-plugin-token` — "User id 'demo-nw-1' is not a valid UUID" — taking the
-- whole demo suite down before it started. A warm working copy happened to pick
-- a real admin, which is why it never showed up locally.
--
-- The platform `admin` role goes too. These are display fixtures for the
-- cross-customer console; being an owner of their own org is already expressed
-- by organization_members.org_role, and platform admin is a different and much
-- larger grant that they never needed.
--
-- Ids are remapped by delete-and-reinsert rather than UPDATE: every foreign key
-- into users.id is NO ACTION (21 of them), so there is no ordering that lets an
-- id change in place. These rows are pure fixtures in the `demo-` namespace and
-- carry no real activity, so recreating them loses nothing. Organization and
-- department ids are deliberately left alone — nothing parses those as UUIDs.
--
-- Re-runnable: the deletes are scoped to the ten known fixture ids and the
-- inserts are guarded, so applying this twice is a no-op.

DELETE FROM ai_requests WHERE id LIKE 'demo-req-%';

DELETE FROM organization_members WHERE user_id IN (
    'demo-nw-1','demo-nw-2','demo-nw-3','demo-nw-4','demo-nw-5',
    'demo-co-1','demo-co-2','demo-co-3','demo-it-1','demo-it-2');

DELETE FROM user_profile_ext WHERE user_id IN (
    'demo-nw-1','demo-nw-2','demo-nw-3','demo-nw-4','demo-nw-5',
    'demo-co-1','demo-co-2','demo-co-3','demo-it-1','demo-it-2');

DELETE FROM users WHERE id IN (
    'demo-nw-1','demo-nw-2','demo-nw-3','demo-nw-4','demo-nw-5',
    'demo-co-1','demo-co-2','demo-co-3','demo-it-1','demo-it-2');

-- Fixed literals, not gen_random_uuid(): the ids have to be stable across every
-- database so the console, the demos and the reports can be reasoned about. The
-- `de…` prefix and the sequential tail make them obviously synthetic on sight.
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

-- Same traffic shape as 025, rebuilt against the new ids: the three tenants
-- have to keep landing in three different budget bands or the console's health
-- pill only ever shows one colour.
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
