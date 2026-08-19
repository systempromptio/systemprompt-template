-- Management section: desktop app device linkage.
--
-- Departments used to live here; they are declared in 16_organizations.sql
-- now, because they carry a foreign key onto `organizations` and a declarative
-- schema may not ALTER an earlier table.

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
