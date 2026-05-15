use systemprompt::extension::prelude::{Migration, SchemaDefinition};

pub const SCHEMA_CONTENT: &str = include_str!("../schema/01_content.sql");
pub const SCHEMA_CAMPAIGNS: &str = include_str!("../schema/02_campaigns.sql");
pub const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/05_plugin_usage.sql");
pub const SCHEMA_MARKETPLACE: &str = include_str!("../schema/06_marketplace.sql");
pub const SCHEMA_ANALYTICS: &str = include_str!("../schema/07_analytics.sql");
pub const SCHEMA_GAMIFICATION: &str = include_str!("../schema/08_gamification.sql");
pub const SCHEMA_SECRETS: &str = include_str!("../schema/09_secrets.sql");
pub const SCHEMA_ADMIN_DASHBOARD: &str = include_str!("../schema/10_admin_dashboard.sql");
pub const SCHEMA_AUDIT_EVENT_NOTIFY: &str = include_str!("../schema/11_audit_event_notify.sql");
pub const SCHEMA_MANAGEMENT: &str = include_str!("../schema/12_management.sql");

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::new("", SCHEMA_CONTENT),
        SchemaDefinition::new("", SCHEMA_CAMPAIGNS),
        SchemaDefinition::new("", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::new("", SCHEMA_MARKETPLACE),
        SchemaDefinition::new("", SCHEMA_ANALYTICS),
        SchemaDefinition::new("", SCHEMA_GAMIFICATION),
        SchemaDefinition::new("", SCHEMA_SECRETS),
        SchemaDefinition::new("", SCHEMA_ADMIN_DASHBOARD),
        SchemaDefinition::new("", SCHEMA_AUDIT_EVENT_NOTIFY),
        SchemaDefinition::new("", SCHEMA_MANAGEMENT),
    ]
}

pub fn migrations() -> Vec<Migration> {
    vec![
        Migration::new(
            1,
            "markdown_content_columns",
            include_str!("../schema/migrations/001_markdown_content_columns.sql"),
        ),
        Migration::new(
            2,
            "plugin_usage_columns",
            include_str!("../schema/migrations/002_plugin_usage_columns.sql"),
        ),
        Migration::new(
            3,
            "session_summary_widen",
            include_str!("../schema/migrations/003_session_summary_widen.sql"),
        ),
        Migration::new(
            4,
            "users_columns",
            include_str!("../schema/migrations/004_users_columns.sql"),
        ),
        Migration::new(
            5,
            "drop_legacy_user_entities",
            include_str!("../schema/migrations/005_drop_legacy_user_entities.sql"),
        ),
        Migration::new(
            6,
            "drop_legacy_marketplace",
            include_str!("../schema/migrations/006_drop_legacy_marketplace.sql"),
        ),
        Migration::new(
            7,
            "admin_dashboard_seeds",
            include_str!("../schema/migrations/007_admin_dashboard_seeds.sql"),
        ),
        Migration::new(
            8,
            "management",
            include_str!("../schema/migrations/008_management.sql"),
        ),
        Migration::new(
            9,
            "departments_default",
            include_str!("../schema/migrations/009_departments_default.sql"),
        ),
        Migration::new(
            10,
            "seed_oauth",
            include_str!("../schema/migrations/010_seed_oauth.sql"),
        ),
        Migration::new(
            11,
            "seed_dashboard",
            include_str!("../schema/migrations/011_seed_dashboard.sql"),
        ),
        Migration::new(
            12,
            "seed_gamification",
            include_str!("../schema/migrations/012_seed_gamification.sql"),
        ),
    ]
}
