-- Replace the synthetic 'system' owner with the real bootstrap 'admin' user.
--
-- Core 0.11 introduced `systemprompt admin bootstrap` (idempotent) as the
-- canonical seed path for the platform admin. The scheduler resolves every
-- job's `owner:` against `users.name`, so the admin row must exist and be
-- active before services start.
--
-- This migration creates that row eagerly so the marketplace-admin OAuth
-- client FK (added by 015) points at the same admin the scheduler resolves.
-- `admin bootstrap` at boot will find this row, verify status, and ensure
-- the 'admin' role is present.

BEGIN;

INSERT INTO users (
    id, name, email, full_name, display_name,
    status, email_verified, roles, is_bot, is_scanner
)
SELECT
    gen_random_uuid()::text, 'admin', 'admin@localhost',
    'Platform Admin', 'Platform Admin',
    'active', true, ARRAY['admin', 'user']::TEXT[], false, false
WHERE NOT EXISTS (SELECT 1 FROM users WHERE name = 'admin');

UPDATE oauth_clients
   SET owner_user_id = (SELECT id FROM users WHERE name = 'admin')
 WHERE client_id = 'marketplace-admin';

DELETE FROM users
 WHERE name = 'system'
   AND NOT EXISTS (
       SELECT 1 FROM oauth_clients WHERE owner_user_id = users.id
   );

COMMIT;
