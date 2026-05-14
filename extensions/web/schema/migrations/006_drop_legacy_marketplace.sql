-- 2026-05-07 catalog refactor: marketplaces moved to YAML-defined services.
-- These tables no longer exist in the declarative schema; drop leftovers
-- from pre-refactor deployments.

DROP TABLE IF EXISTS marketplace_changelog CASCADE;
DROP TABLE IF EXISTS marketplace_versions CASCADE;
DROP TABLE IF EXISTS org_marketplace_plugins CASCADE;
DROP TABLE IF EXISTS org_marketplaces CASCADE;
DROP TABLE IF EXISTS plugin_ratings CASCADE;
DROP TABLE IF EXISTS plugin_visibility_rules CASCADE;
DROP TABLE IF EXISTS plugin_installations CASCADE;
DROP TABLE IF EXISTS plugin_installation_history CASCADE;
DROP TABLE IF EXISTS org_marketplace_sync_logs CASCADE;
