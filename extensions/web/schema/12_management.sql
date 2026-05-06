-- Consolidated schema: Management section (departments + desktop app status)
--
-- Departments become a first-class CRUD entity backing the existing free-text
-- users.department field. Skill assignment via access_control_rules is enabled
-- by widening the entity_type check. Desktop app linking is captured per device.

CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    manager_user_id TEXT REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_departments_manager ON departments(manager_user_id);

-- Backfill from any existing free-text departments on users.
INSERT INTO departments (name)
SELECT DISTINCT department
FROM users
WHERE department IS NOT NULL AND department <> ''
ON CONFLICT (name) DO NOTHING;

-- Allow access_control_rules to govern skills as well (was: plugin/agent/mcp_server/marketplace).
ALTER TABLE access_control_rules DROP CONSTRAINT IF EXISTS access_control_rules_entity_type_check;
ALTER TABLE access_control_rules ADD CONSTRAINT access_control_rules_entity_type_check
    CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'marketplace', 'skill', 'gateway_route'));

-- Desktop app linkage. device_id matches the cowork api_key id or device_cert id
-- depending on enrolment mode; both are TEXT, so we keep this loose intentionally.
CREATE TABLE IF NOT EXISTS device_app_links (
    device_id TEXT PRIMARY KEY,
    user_id TEXT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    app_platform TEXT NOT NULL CHECK (app_platform IN ('macos', 'windows', 'linux')),
    app_version TEXT NOT NULL DEFAULT '',
    hostname TEXT NOT NULL DEFAULT '',
    last_seen_at TIMESTAMPTZ,
    enrolled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_device_app_links_user ON device_app_links(user_id);
CREATE INDEX IF NOT EXISTS idx_device_app_links_last_seen ON device_app_links(last_seen_at DESC NULLS LAST);
