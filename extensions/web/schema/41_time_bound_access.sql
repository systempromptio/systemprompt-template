-- Validity windows for the two core-owned access rows this extension bounds.
--
-- `access_control_rules` (core `systemprompt-security`) and
-- `user_device_certs` (core `systemprompt-users`) are core tables, and the
-- declarative-schema gate forbids an `ALTER TABLE` here, so their expiry
-- lives in a side table keyed by the row id and cascades away with it. The
-- extension-owned membership tables carry `valid_from` / `valid_until`
-- directly (23_groups_projects.sql).
--
-- Core's resolver reads `access_control_rules` and core's device gate reads
-- `user_device_certs.revoked_at`; neither consults these tables. The hourly
-- `access_expiry` job is what makes a window bind: it deletes a rule whose
-- window has closed and stamps `revoked_at` on an expired certificate. The
-- console reads the tables directly to show what is due to expire.

CREATE TABLE IF NOT EXISTS access_control_rule_validity (
    rule_id TEXT PRIMARY KEY REFERENCES access_control_rules(id) ON DELETE CASCADE,
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_access_control_rule_validity_until
    ON access_control_rule_validity(valid_until);

CREATE TABLE IF NOT EXISTS user_device_cert_validity (
    device_id TEXT PRIMARY KEY REFERENCES user_device_certs(id) ON DELETE CASCADE,
    valid_until TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX IF NOT EXISTS idx_user_device_cert_validity_until
    ON user_device_cert_validity(valid_until);
