-- Per-user entity tables (skills, agents, plugins, MCP servers, hooks) were
-- removed: those are now defined in services/*.yaml only. Drop any stragglers
-- from older deployments.

DROP TABLE IF EXISTS user_plugin_mcp_servers CASCADE;
DROP TABLE IF EXISTS user_plugin_agents CASCADE;
DROP TABLE IF EXISTS user_plugin_skills CASCADE;
DROP TABLE IF EXISTS user_plugin_hooks CASCADE;
DROP TABLE IF EXISTS user_mcp_servers CASCADE;
DROP TABLE IF EXISTS user_plugins CASCADE;
DROP TABLE IF EXISTS user_agents CASCADE;
DROP TABLE IF EXISTS user_hooks CASCADE;
DROP TABLE IF EXISTS user_skills CASCADE;
DROP TABLE IF EXISTS hook_overrides CASCADE;
DROP TABLE IF EXISTS skill_files CASCADE;
DROP TABLE IF EXISTS marketplace_sync_status CASCADE;
DROP TABLE IF EXISTS user_selected_org_plugins CASCADE;
DROP FUNCTION IF EXISTS mark_marketplace_dirty() CASCADE;
