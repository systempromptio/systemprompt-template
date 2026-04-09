use systemprompt::extension::prelude::SchemaDefinition;

// Consolidated schema files
pub const SCHEMA_CONTENT: &str = include_str!("../schema/01_content.sql");
pub const SCHEMA_CAMPAIGNS: &str = include_str!("../schema/02_campaigns.sql");
pub const SCHEMA_USERS: &str = include_str!("../schema/03_users.sql");
pub const SCHEMA_USER_ENTITIES: &str = include_str!("../schema/04_user_entities.sql");
pub const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/05_plugin_usage.sql");
pub const SCHEMA_MARKETPLACE: &str = include_str!("../schema/06_marketplace.sql");
pub const SCHEMA_ANALYTICS: &str = include_str!("../schema/07_analytics.sql");
pub const SCHEMA_GAMIFICATION: &str = include_str!("../schema/08_gamification.sql");
pub const SCHEMA_SECRETS: &str = include_str!("../schema/09_secrets.sql");
pub const SCHEMA_ADMIN_DASHBOARD: &str = include_str!("../schema/10_admin_dashboard.sql");

// Seed data files
pub const SEED_OAUTH: &str = include_str!("../schema/seed_01_oauth.sql");
pub const SEED_DASHBOARD: &str = include_str!("../schema/seed_02_dashboard.sql");
pub const SEED_GAMIFICATION: &str = include_str!("../schema/seed_03_gamification.sql");
pub const SEED_MARKETPLACE: &str = include_str!("../schema/seed_04_marketplace.sql");

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        // Core table amendments (always run, idempotent)
        SchemaDefinition::inline("", SCHEMA_USERS),
        // Schema files (always run, all use IF NOT EXISTS)
        SchemaDefinition::inline("", SCHEMA_CONTENT),
        SchemaDefinition::inline("", SCHEMA_CAMPAIGNS),
        SchemaDefinition::inline("", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::inline("", SCHEMA_USER_ENTITIES),
        SchemaDefinition::inline("", SCHEMA_MARKETPLACE),
        SchemaDefinition::inline("", SCHEMA_ANALYTICS),
        SchemaDefinition::inline("", SCHEMA_GAMIFICATION),
        SchemaDefinition::inline("", SCHEMA_SECRETS),
        SchemaDefinition::inline("", SCHEMA_ADMIN_DASHBOARD),
        // Seed data (always run, idempotent via ON CONFLICT)
        SchemaDefinition::inline("", SEED_OAUTH),
        SchemaDefinition::inline("", SEED_DASHBOARD),
        SchemaDefinition::inline("", SEED_GAMIFICATION),
        SchemaDefinition::inline("", SEED_MARKETPLACE),
    ]
}
