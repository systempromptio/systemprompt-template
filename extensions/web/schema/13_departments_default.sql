-- Drop the manager concept from departments and ensure every user lives in
-- a department. The "Default" department is the catch-all for any user whose
-- `users.department` was empty.
--
-- Idempotent: safe to run on fresh installs (no-ops the DROP) and on existing
-- DBs (drops the column + index, backfills users, seeds the default).

ALTER TABLE departments DROP COLUMN IF EXISTS manager_user_id;
DROP INDEX IF EXISTS idx_departments_manager;

INSERT INTO departments (name, description)
VALUES ('Default', 'Default department; contains every user without an explicit assignment.')
ON CONFLICT (name) DO NOTHING;

ALTER TABLE users ALTER COLUMN department SET DEFAULT 'Default';

UPDATE users SET department = 'Default'
WHERE department IS NULL OR department = '';
