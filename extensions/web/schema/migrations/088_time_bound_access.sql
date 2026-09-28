-- Twin of schema/23_groups_projects.sql (validity columns, windowed views)
-- and schema/41_time_bound_access.sql (side tables for the core-owned rows).
--
-- Every membership and manual role grant gains a validity window. Existing
-- rows are open-ended: `valid_from` backfills to when the row was granted and
-- `valid_until` stays NULL, so nothing already held changes hands here. The
-- `user_groups` view is replaced with the windowed definition and a
-- `user_projects` twin is added so every resolver inherits the predicate.

ALTER TABLE group_members
    ADD COLUMN IF NOT EXISTS valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ADD COLUMN IF NOT EXISTS valid_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;
UPDATE group_members SET valid_from = created_at WHERE valid_from > created_at;
CREATE INDEX IF NOT EXISTS idx_group_members_valid_until ON group_members(valid_until)
    WHERE valid_until IS NOT NULL AND revoked_at IS NULL;

ALTER TABLE project_members
    ADD COLUMN IF NOT EXISTS valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ADD COLUMN IF NOT EXISTS valid_until TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS revoked_at TIMESTAMPTZ;
UPDATE project_members SET valid_from = created_at WHERE valid_from > created_at;
CREATE INDEX IF NOT EXISTS idx_project_members_valid_until ON project_members(valid_until)
    WHERE valid_until IS NOT NULL AND revoked_at IS NULL;

ALTER TABLE user_manual_roles
    ADD COLUMN IF NOT EXISTS valid_from TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ADD COLUMN IF NOT EXISTS valid_until TIMESTAMPTZ;
UPDATE user_manual_roles SET valid_from = created_at WHERE valid_from > created_at;
CREATE INDEX IF NOT EXISTS idx_user_manual_roles_valid_until ON user_manual_roles(valid_until)
    WHERE valid_until IS NOT NULL;

CREATE OR REPLACE VIEW user_groups AS
SELECT DISTINCT gm.user_id, gm.group_id FROM group_members gm
WHERE gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
  AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP)
UNION ALL
SELECT u.id AS user_id, 'unassigned' AS group_id
FROM users u
WHERE NOT ('anonymous' = ANY(u.roles))
  AND NOT EXISTS (
      SELECT 1 FROM group_members gm WHERE gm.user_id = u.id
        AND gm.revoked_at IS NULL AND gm.valid_from <= CURRENT_TIMESTAMP
        AND (gm.valid_until IS NULL OR gm.valid_until > CURRENT_TIMESTAMP));

CREATE OR REPLACE VIEW user_projects AS
SELECT DISTINCT pm.user_id, pm.project_id FROM project_members pm
WHERE pm.revoked_at IS NULL AND pm.valid_from <= CURRENT_TIMESTAMP
  AND (pm.valid_until IS NULL OR pm.valid_until > CURRENT_TIMESTAMP);

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
