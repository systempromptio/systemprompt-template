use systemprompt::extension::prelude::SchemaDefinition;

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
pub const SCHEMA_AUDIT_EVENT_NOTIFY: &str = include_str!("../schema/11_audit_event_notify.sql");
pub const SCHEMA_MANAGEMENT: &str = include_str!("../schema/12_management.sql");
pub const SCHEMA_DEPARTMENTS_DEFAULT: &str = include_str!("../schema/13_departments_default.sql");

pub const SEED_OAUTH: &str = include_str!("../schema/seed_01_oauth.sql");
pub const SEED_DASHBOARD: &str = include_str!("../schema/seed_02_dashboard.sql");
pub const SEED_GAMIFICATION: &str = include_str!("../schema/seed_03_gamification.sql");

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::new("", SCHEMA_USERS),
        SchemaDefinition::new("", SCHEMA_CONTENT),
        SchemaDefinition::new("", SCHEMA_CAMPAIGNS),
        SchemaDefinition::new("", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::new("", SCHEMA_USER_ENTITIES),
        SchemaDefinition::new("", SCHEMA_MARKETPLACE),
        SchemaDefinition::new("", SCHEMA_ANALYTICS),
        SchemaDefinition::new("", SCHEMA_GAMIFICATION),
        SchemaDefinition::new("", SCHEMA_SECRETS),
        SchemaDefinition::new("", SCHEMA_ADMIN_DASHBOARD),
        SchemaDefinition::new("", SCHEMA_AUDIT_EVENT_NOTIFY),
        SchemaDefinition::new("", SCHEMA_MANAGEMENT),
        SchemaDefinition::new("", SCHEMA_DEPARTMENTS_DEFAULT),
        SchemaDefinition::new("", SEED_OAUTH),
        SchemaDefinition::new("", SEED_DASHBOARD),
        SchemaDefinition::new("", SEED_GAMIFICATION),
    ]
}
