-- Index on users.department. The column itself is added by migration
-- 004_users_columns.sql since `users` is core-owned.
CREATE INDEX IF NOT EXISTS idx_users_department ON users(department);
