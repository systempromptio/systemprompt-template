//! Schema and migration inventory for the web extension.
//!
//! DDL lives in `schema/*.sql` and is embedded with `include_str!`; migrations
//! are discovered from `schema/migrations/` by `build.rs`. Nothing here is SQL
//! text.

use systemprompt::extension::prelude::{Migration, SchemaDefinition, extension_migrations};

pub(crate) const SCHEMA_PLUGIN_USAGE: &str = include_str!("../schema/05_plugin_usage.sql");
pub(crate) const SCHEMA_ANALYTICS: &str = include_str!("../schema/07_analytics.sql");
pub(crate) const SCHEMA_SECRETS: &str = include_str!("../schema/09_secrets.sql");
pub(crate) const SCHEMA_ADMIN_DASHBOARD: &str = include_str!("../schema/10_admin_dashboard.sql");
pub(crate) const SCHEMA_MANAGEMENT: &str = include_str!("../schema/12_management.sql");
pub(crate) const SCHEMA_WEB_SIDE_TABLES: &str = include_str!("../schema/13_web_side_tables.sql");
pub(crate) const SCHEMA_AUDIT_EVENT_NOTIFY: &str =
    include_str!("../schema/14_audit_event_notify.sql");
pub(crate) const SCHEMA_USAGE_METRICS: &str = include_str!("../schema/17_usage_metrics.sql");
pub(crate) const SCHEMA_SALESFORCE_IDENTITY: &str =
    include_str!("../schema/21_salesforce_identity.sql");
pub(crate) const SCHEMA_DEV_LOGIN_CODES: &str = include_str!("../schema/22_dev_login_codes.sql");
pub(crate) const SCHEMA_GROUPS_PROJECTS: &str = include_str!("../schema/23_groups_projects.sql");
pub(crate) const SCHEMA_SCOPE_DEFAULTS: &str = include_str!("../schema/24_scope_defaults.sql");

pub(crate) const SCHEMA_CONNECTOR_CREDENTIALS: &str =
    include_str!("../schema/25_connector_credentials.sql");

#[doc(hidden)]
pub fn schema_definitions() -> Vec<SchemaDefinition> {
    vec![
        SchemaDefinition::new("", SCHEMA_PLUGIN_USAGE),
        SchemaDefinition::new("", SCHEMA_ANALYTICS),
        SchemaDefinition::new("", SCHEMA_SECRETS),
        SchemaDefinition::new("", SCHEMA_ADMIN_DASHBOARD),
        SchemaDefinition::new("", SCHEMA_MANAGEMENT),
        SchemaDefinition::new("", SCHEMA_WEB_SIDE_TABLES),
        SchemaDefinition::new("", SCHEMA_AUDIT_EVENT_NOTIFY),
        SchemaDefinition::new("", SCHEMA_USAGE_METRICS),
        SchemaDefinition::new("", SCHEMA_SALESFORCE_IDENTITY),
        SchemaDefinition::new("", SCHEMA_DEV_LOGIN_CODES),
        SchemaDefinition::new("", SCHEMA_GROUPS_PROJECTS),
        SchemaDefinition::new("", SCHEMA_SCOPE_DEFAULTS),
        SchemaDefinition::new("", SCHEMA_CONNECTOR_CREDENTIALS),
        SchemaDefinition::new("", include_str!("../schema/26_connector_accounts.sql")),
        SchemaDefinition::new("", include_str!("../schema/27_conversation_requests.sql")),
        SchemaDefinition::new("", include_str!("../schema/28_ingestion_integrity.sql")),
        SchemaDefinition::new("", include_str!("../schema/29_skill_version_impact.sql")),
    ]
}

// Why: not `const` — with migrations present, `extension_migrations!()`
// expands to a `vec![…]` of embedded files, which cannot be built in const
// context (it could while the migrations directory was empty).
#[doc(hidden)]
pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
