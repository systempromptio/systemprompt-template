-- Drop the manager concept from departments and ensure the catch-all
-- "Default" department exists.
--
-- Idempotent across fresh installs (no-ops the DROP) and existing DBs.

ALTER TABLE departments DROP COLUMN IF EXISTS manager_user_id;
DROP INDEX IF EXISTS idx_departments_manager;

INSERT INTO departments (name, description)
VALUES ('Default', 'Default department; contains every user without an explicit assignment.')
ON CONFLICT (name) DO NOTHING;
