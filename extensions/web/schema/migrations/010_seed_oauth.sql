-- Seed: OAuth client configuration for admin dashboard.
--
-- Schema-aware: as of systemprompt-core 0.11 the core migration
-- 004_oauth_client_owner added owner_user_id NOT NULL to oauth_clients.
-- This seed predates that change. On a fresh DB the new column is present
-- and this seed is fully superseded by 015_reseed_oauth_client_owner.sql
-- (which creates a synthetic 'system' user as the owner before inserting
-- the client). On a legacy DB the new column is absent and this seed must
-- run as-originally-written to keep the upgrade path intact.
--
-- The DO block detects the schema and skips when 015 will do the work.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'oauth_clients'
          AND column_name = 'owner_user_id'
          AND is_nullable = 'NO'
    ) THEN
        -- New core schema: defer to 015_reseed_oauth_client_owner.sql
        RETURN;
    END IF;

    -- Legacy core schema: original seed.
    INSERT INTO oauth_clients (client_id, client_secret_hash, client_name, token_endpoint_auth_method, is_active)
    VALUES ('marketplace-admin', NULL, 'Marketplace Admin Dashboard', 'none', true)
    ON CONFLICT (client_id) DO NOTHING;

    INSERT INTO oauth_client_grant_types (client_id, grant_type)
    VALUES ('marketplace-admin', 'authorization_code')
    ON CONFLICT (client_id, grant_type) DO NOTHING;

    INSERT INTO oauth_client_grant_types (client_id, grant_type)
    VALUES ('marketplace-admin', 'refresh_token')
    ON CONFLICT (client_id, grant_type) DO NOTHING;

    INSERT INTO oauth_client_response_types (client_id, response_type)
    VALUES ('marketplace-admin', 'code')
    ON CONFLICT (client_id, response_type) DO NOTHING;

    INSERT INTO oauth_client_scopes (client_id, scope)
    VALUES ('marketplace-admin', 'admin')
    ON CONFLICT (client_id, scope) DO NOTHING;

    INSERT INTO oauth_client_scopes (client_id, scope)
    VALUES ('marketplace-admin', 'user')
    ON CONFLICT (client_id, scope) DO NOTHING;

    INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
    VALUES ('marketplace-admin', '/admin/login', true)
    ON CONFLICT (client_id, redirect_uri) DO NOTHING;

    INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
    VALUES ('marketplace-admin', 'http://localhost:8080/admin/login', false)
    ON CONFLICT (client_id, redirect_uri) DO NOTHING;

    INSERT INTO oauth_client_redirect_uris (client_id, redirect_uri, is_primary)
    VALUES ('marketplace-admin', 'https://f7ae798f9c2a.systemprompt.io/admin/login', false)
    ON CONFLICT (client_id, redirect_uri) DO NOTHING;
END $$;
