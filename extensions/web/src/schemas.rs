use systemprompt::extension::prelude::{extension_migrations, Migration, SchemaDefinition};

pub const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/05_plugin_usage.sql");
pub const SCHEMA_ANALYTICS: &str = include_str!("../schema/07_analytics.sql");
pub const SCHEMA_SECRETS: &str = include_str!("../schema/09_secrets.sql");
pub const SCHEMA_ADMIN_DASHBOARD: &str = include_str!("../schema/10_admin_dashboard.sql");
pub const SCHEMA_MANAGEMENT: &str = include_str!("../schema/12_management.sql");
pub const SCHEMA_WEB_SIDE_TABLES: &str = include_str!("../schema/13_web_side_tables.sql");
pub const SCHEMA_AUDIT_EVENT_NOTIFY: &str = include_str!("../schema/14_audit_event_notify.sql");

pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::new("", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::new("", SCHEMA_ANALYTICS),
        SchemaDefinition::new("", SCHEMA_SECRETS),
        SchemaDefinition::new("", SCHEMA_ADMIN_DASHBOARD),
        SchemaDefinition::new("", SCHEMA_MANAGEMENT),
        SchemaDefinition::new("", SCHEMA_WEB_SIDE_TABLES),
        SchemaDefinition::new("", SCHEMA_AUDIT_EVENT_NOTIFY),
    ]
}

pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
