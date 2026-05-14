-- Management section: departments + desktop app device linkage.
--
-- Departments back the `users.department` field with a first-class table.
-- Skill assignment via access_control_rules is enabled by widening the
-- entity_type check (see migrations/008_management.sql for the constraint
-- swap on pre-existing installs). Backfill of legacy free-text departments
-- and the constraint update live in the same migration.

CREATE TABLE IF NOT EXISTS departments (
    id TEXT PRIMARY KEY DEFAULT gen_random_uuid()::TEXT,
    name TEXT NOT NULL UNIQUE,
    description TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

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
