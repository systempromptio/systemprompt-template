use systemprompt::extension::prelude::SchemaDefinition;

pub const SCHEMA_MARKDOWN_CONTENT: &str = include_str!("../schema/001_markdown_content.sql");
pub const SCHEMA_MARKDOWN_CATEGORIES: &str = include_str!("../schema/002_markdown_categories.sql");
pub const SCHEMA_CAMPAIGN_LINKS: &str = include_str!("../schema/003_campaign_links.sql");
pub const SCHEMA_LINK_CLICKS: &str = include_str!("../schema/004_link_clicks.sql");
pub const SCHEMA_LINK_ANALYTICS_VIEWS: &str =
    include_str!("../schema/005_link_analytics_views.sql");
pub const SCHEMA_CONTENT_PERFORMANCE_METRICS: &str =
    include_str!("../schema/006_content_performance_metrics.sql");
pub const SCHEMA_MARKDOWN_FTS: &str = include_str!("../schema/007_markdown_fts.sql");
pub const SCHEMA_CONTENT_RELATED_METADATA: &str =
    include_str!("../schema/009_content_related_metadata.sql");
pub const SCHEMA_CONTENT_RELATED_DOCS: &str =
    include_str!("../schema/010_content_related_docs.sql");
pub const SCHEMA_CONTENT_CATEGORY: &str = include_str!("../schema/011_content_category.sql");
pub const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/012_plugin_usage.sql");
pub const SCHEMA_USER_SKILLS: &str = include_str!("../schema/013_user_skills.sql");
pub const SCHEMA_USER_GOVERNANCE: &str = include_str!("../schema/014_user_governance.sql");
pub const SCHEMA_GAMIFICATION: &str = include_str!("../schema/016_gamification.sql");
pub const SCHEMA_ADMIN_OAUTH_CLIENT: &str = include_str!("../schema/015_admin_oauth_client.sql");
pub const SCHEMA_SEED_DASHBOARD: &str = include_str!("../schema/015_seed_dashboard.sql");
pub const SCHEMA_SEED_GAMIFICATION: &str = include_str!("../schema/017_seed_gamification.sql");
pub const SCHEMA_SKILL_FILES: &str = include_str!("../schema/018_skill_files.sql");
pub const SCHEMA_PLUGIN_ENV_VARS: &str = include_str!("../schema/019_plugin_env_vars.sql");
pub const SCHEMA_USER_SKILLS_BASE: &str = include_str!("../schema/020_user_skills_base.sql");
pub const SCHEMA_MARKETPLACE: &str = include_str!("../schema/021_marketplace.sql");
pub const SCHEMA_MARKETPLACE_VERSIONS: &str =
    include_str!("../schema/022_marketplace_versions.sql");
pub const SCHEMA_ADMIN_OAUTH_USER_SCOPE: &str =
    include_str!("../schema/023_admin_oauth_user_scope.sql");
pub const SCHEMA_FIX_USER_ROLES: &str = include_str!("../schema/024_fix_user_roles.sql");
pub const SCHEMA_USER_ACTIVITY: &str = include_str!("../schema/025_user_activity.sql");
pub const SCHEMA_USAGE_AGGREGATIONS: &str = include_str!("../schema/026_usage_aggregations.sql");
pub const SCHEMA_USER_AGENTS: &str = include_str!("../schema/027_user_agents.sql");
pub const SCHEMA_HOOK_OVERRIDES: &str = include_str!("../schema/028_hook_overrides.sql");
pub const SCHEMA_ACCESS_CONTROL: &str = include_str!("../schema/029_access_control.sql");
pub const SCHEMA_USER_ENTITIES: &str = include_str!("../schema/030_user_entities.sql");
pub const SCHEMA_ORG_MARKETPLACES: &str = include_str!("../schema/031_marketplaces.sql");
pub const SCHEMA_HOOK_CATALOG: &str = include_str!("../schema/032_hook_catalog.sql");
pub const SCHEMA_USER_ENCRYPTION_KEYS: &str =
    include_str!("../schema/033_user_encryption_keys.sql");
pub const SCHEMA_SECRET_ENCRYPTION: &str = include_str!("../schema/034_secret_encryption.sql");
pub const SCHEMA_SECRET_RESOLUTION_TOKENS: &str =
    include_str!("../schema/035_secret_resolution_tokens.sql");
pub const SCHEMA_SKILL_SECRETS: &str = include_str!("../schema/036_skill_secrets.sql");
pub const SCHEMA_MAGIC_LINK_TOKENS: &str = include_str!("../schema/037_magic_link_tokens.sql");
pub const SCHEMA_USER_PLUGIN_SELECTIONS: &str =
    include_str!("../schema/038_user_plugin_selections.sql");
pub const SCHEMA_DEDUP_KEY: &str = include_str!("../schema/039_dedup_key.sql");
pub const SCHEMA_SESSION_TRANSCRIPTS: &str = include_str!("../schema/040_session_transcripts.sql");
pub const SCHEMA_TRANSCRIPT_TOKEN_TRACKING: &str =
    include_str!("../schema/041_transcript_token_tracking.sql");
pub const SCHEMA_PLUGIN_INSTALLATIONS: &str =
    include_str!("../schema/042_plugin_installations.sql");
pub const SCHEMA_SEED_ENTERPRISE_DEMO: &str =
    include_str!("../schema/043_seed_enterprise_demo_marketplace.sql");
pub const SCHEMA_MARKETPLACE_GITHUB: &str =
    include_str!("../schema/044_marketplace_github.sql");
pub const SCHEMA_GOVERNANCE_DECISIONS: &str =
    include_str!("../schema/045_governance_decisions.sql");
pub const SCHEMA_MCP_ACCESS_TRACKING: &str =
    include_str!("../schema/045_mcp_access_tracking.sql");
pub const SCHEMA_ADMIN_DASHBOARD_TABLES: &str =
    include_str!("../schema/046_admin_dashboard_tables.sql");
pub const SCHEMA_ADMIN_DASHBOARD_VIEWS: &str =
    include_str!("../schema/047_admin_dashboard_views.sql");
pub const SCHEMA_ADMIN_DASHBOARD_COLUMNS: &str =
    include_str!("../schema/048_admin_dashboard_columns.sql");
pub const SCHEMA_PROFILE_REPORTS_HOOKS_FIX: &str =
    include_str!("../schema/049_profile_reports_and_hooks_fix.sql");

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::inline("markdown_content", SCHEMA_MARKDOWN_CONTENT),
        SchemaDefinition::inline("markdown_categories", SCHEMA_MARKDOWN_CATEGORIES),
        SchemaDefinition::inline("campaign_links", SCHEMA_CAMPAIGN_LINKS),
        SchemaDefinition::inline("link_clicks", SCHEMA_LINK_CLICKS),
        SchemaDefinition::inline("link_analytics_views", SCHEMA_LINK_ANALYTICS_VIEWS),
        SchemaDefinition::inline(
            "content_performance_metrics",
            SCHEMA_CONTENT_PERFORMANCE_METRICS,
        ),
        SchemaDefinition::inline("markdown_fts", SCHEMA_MARKDOWN_FTS),
        SchemaDefinition::inline("content_related_metadata", SCHEMA_CONTENT_RELATED_METADATA),
        SchemaDefinition::inline("content_related_docs", SCHEMA_CONTENT_RELATED_DOCS),
        SchemaDefinition::inline("content_category", SCHEMA_CONTENT_CATEGORY),
        SchemaDefinition::inline("plugin_usage", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::inline("user_skills", SCHEMA_USER_SKILLS),
        SchemaDefinition::inline("user_governance", SCHEMA_USER_GOVERNANCE),
        SchemaDefinition::inline("gamification", SCHEMA_GAMIFICATION),
        SchemaDefinition::inline("admin_oauth_client", SCHEMA_ADMIN_OAUTH_CLIENT),
        SchemaDefinition::inline("seed_dashboard", SCHEMA_SEED_DASHBOARD),
        SchemaDefinition::inline("seed_gamification", SCHEMA_SEED_GAMIFICATION),
        SchemaDefinition::inline("skill_files", SCHEMA_SKILL_FILES),
        SchemaDefinition::inline("plugin_env_vars", SCHEMA_PLUGIN_ENV_VARS),
        SchemaDefinition::inline("user_skills_base", SCHEMA_USER_SKILLS_BASE),
        SchemaDefinition::inline("marketplace", SCHEMA_MARKETPLACE),
        SchemaDefinition::inline("marketplace_versions", SCHEMA_MARKETPLACE_VERSIONS),
        SchemaDefinition::inline("admin_oauth_user_scope", SCHEMA_ADMIN_OAUTH_USER_SCOPE),
        SchemaDefinition::inline("fix_user_roles", SCHEMA_FIX_USER_ROLES),
        SchemaDefinition::inline("user_activity", SCHEMA_USER_ACTIVITY),
        SchemaDefinition::inline("usage_aggregations", SCHEMA_USAGE_AGGREGATIONS),
        SchemaDefinition::inline("user_agents", SCHEMA_USER_AGENTS),
        SchemaDefinition::inline("hook_overrides", SCHEMA_HOOK_OVERRIDES),
        SchemaDefinition::inline("access_control", SCHEMA_ACCESS_CONTROL),
        SchemaDefinition::inline("user_entities", SCHEMA_USER_ENTITIES),
        SchemaDefinition::inline("org_marketplaces", SCHEMA_ORG_MARKETPLACES),
        SchemaDefinition::inline("hook_catalog", SCHEMA_HOOK_CATALOG),
        SchemaDefinition::inline("user_encryption_keys", SCHEMA_USER_ENCRYPTION_KEYS),
        SchemaDefinition::inline("secret_encryption", SCHEMA_SECRET_ENCRYPTION),
        SchemaDefinition::inline("secret_resolution_tokens", SCHEMA_SECRET_RESOLUTION_TOKENS),
        SchemaDefinition::inline("skill_secrets", SCHEMA_SKILL_SECRETS),
        SchemaDefinition::inline("magic_link_tokens", SCHEMA_MAGIC_LINK_TOKENS),
        SchemaDefinition::inline("user_plugin_selections", SCHEMA_USER_PLUGIN_SELECTIONS),
        SchemaDefinition::inline("dedup_key", SCHEMA_DEDUP_KEY),
        SchemaDefinition::inline("session_transcripts", SCHEMA_SESSION_TRANSCRIPTS),
        SchemaDefinition::inline(
            "transcript_token_tracking",
            SCHEMA_TRANSCRIPT_TOKEN_TRACKING,
        ),
        SchemaDefinition::inline("plugin_installations", SCHEMA_PLUGIN_INSTALLATIONS),
        SchemaDefinition::inline("seed_enterprise_demo", SCHEMA_SEED_ENTERPRISE_DEMO),
        SchemaDefinition::inline("marketplace_github", SCHEMA_MARKETPLACE_GITHUB),
        SchemaDefinition::inline("governance_decisions", SCHEMA_GOVERNANCE_DECISIONS),
        SchemaDefinition::inline("mcp_access_tracking", SCHEMA_MCP_ACCESS_TRACKING),
        SchemaDefinition::inline("admin_dashboard_tables", SCHEMA_ADMIN_DASHBOARD_TABLES),
        SchemaDefinition::inline("admin_dashboard_views", SCHEMA_ADMIN_DASHBOARD_VIEWS),
        SchemaDefinition::inline("admin_dashboard_columns", SCHEMA_ADMIN_DASHBOARD_COLUMNS),
        SchemaDefinition::inline("profile_reports_hooks_fix", SCHEMA_PROFILE_REPORTS_HOOKS_FIX),
    ]
}
