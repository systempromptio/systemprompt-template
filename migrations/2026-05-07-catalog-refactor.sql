-- Forward-only, idempotent. Safe to re-run.
-- Catalog refactor (2026-05-07): drops marketplace ratings/visibility/installation
-- tables, narrows access_control_rules.entity_type to the YAML-defined catalog
-- domain (adds 'skill', removes 'marketplace'), and adds per-user
-- share_token_version for revocable shareable manifest links.
-- Apply to systemprompt-prod / FlyIO before next template release.

DROP TABLE IF EXISTS plugin_ratings, plugin_visibility_rules, plugin_installations, plugin_installation_history CASCADE;

DELETE FROM access_control_rules WHERE entity_type = 'marketplace';

ALTER TABLE access_control_rules DROP CONSTRAINT IF EXISTS access_control_rules_entity_type_check;
ALTER TABLE access_control_rules ADD CONSTRAINT access_control_rules_entity_type_check CHECK (entity_type IN ('plugin', 'agent', 'mcp_server', 'skill'));

ALTER TABLE users ADD COLUMN IF NOT EXISTS share_token_version INT NOT NULL DEFAULT 1;
