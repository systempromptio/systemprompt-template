-- 2026-06-30 schema bloat cleanup: drop dead/unreferenced tables.
--
-- Audit of the 99-table schema found these tables had no reachable write path
-- and no live read consumers, so they stayed empty on every deployment:
--   * Gamification (08_gamification.sql): the employee_* family was only ever
--     populated by a one-shot seed (removed 012_seed_gamification.sql); no
--     application code touched it. Its re-platformed successors user_ranks /
--     user_achievements got read-only scaffolding that was never called.
--   * Hooks catalog (10_admin_dashboard.sql): hook_catalog / hook_plugins /
--     hook_files had a schema but no Rust or template ever referenced them.
--   * skill_secrets (09_secrets.sql): only a user-deletion cascade stub existed;
--     never written or read. The plugin_env_vars / secret_* cluster covers secrets.
--
-- Declarative CREATEs were removed from the schema/*.sql files in the same change.
-- CASCADE clears the FKs between hook_catalog and its children. No view depends
-- on any of these (verified against pg_depend before authoring this migration).

DROP TABLE IF EXISTS employee_xp_ledger CASCADE;
DROP TABLE IF EXISTS employee_achievements CASCADE;
DROP TABLE IF EXISTS employee_daily_usage CASCADE;
DROP TABLE IF EXISTS employee_ranks CASCADE;
DROP TABLE IF EXISTS user_achievements CASCADE;
DROP TABLE IF EXISTS user_ranks CASCADE;
DROP TABLE IF EXISTS hook_files CASCADE;
DROP TABLE IF EXISTS hook_plugins CASCADE;
DROP TABLE IF EXISTS hook_catalog CASCADE;
DROP TABLE IF EXISTS skill_secrets CASCADE;
