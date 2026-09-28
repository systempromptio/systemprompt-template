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
pub(crate) const SCHEMA_ORGANIZATIONS: &str = include_str!("../schema/16_organizations.sql");
pub(crate) const SCHEMA_USAGE_METRICS: &str = include_str!("../schema/17_usage_metrics.sql");
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
        SchemaDefinition::new("", SCHEMA_ORGANIZATIONS),
        SchemaDefinition::new("", SCHEMA_USAGE_METRICS),
        SchemaDefinition::new("", SCHEMA_DEV_LOGIN_CODES),
        SchemaDefinition::new("", SCHEMA_GROUPS_PROJECTS),
        SchemaDefinition::new("", SCHEMA_SCOPE_DEFAULTS),
        SchemaDefinition::new("", SCHEMA_CONNECTOR_CREDENTIALS),
        SchemaDefinition::new("", include_str!("../schema/26_connector_accounts.sql")),
        SchemaDefinition::new("", include_str!("../schema/27_conversation_requests.sql")),
        SchemaDefinition::new("", include_str!("../schema/28_ingestion_integrity.sql")),
        SchemaDefinition::new("", include_str!("../schema/29_skill_version_impact.sql")),
        SchemaDefinition::new("", include_str!("../schema/32_raw_retention.sql")),
        SchemaDefinition::new("", include_str!("../schema/33_sync_state.sql")),
        SchemaDefinition::new("", include_str!("../schema/34_service_sources.sql")),
        SchemaDefinition::new("", include_str!("../schema/36_marketplace_versions.sql")),
        SchemaDefinition::new("", include_str!("../schema/37_conversation_analyses.sql")),
        SchemaDefinition::new("", include_str!("../schema/40_request_scopes.sql")),
        SchemaDefinition::new("", include_str!("../schema/41_time_bound_access.sql")),
        SchemaDefinition::new("", include_str!("../schema/42_gateway_routes.sql")),
        // Why: 46 before 45 — conversation_facts' refresh reads the
        // `tool_activity` view that 46 defines.
        SchemaDefinition::new("", include_str!("../schema/46_tool_artifacts.sql")),
        SchemaDefinition::new("", include_str!("../schema/45_conversation_facts.sql")),
        SchemaDefinition::new("", include_str!("../schema/47_user_last_seen.sql")),
        SchemaDefinition::new("", include_str!("../schema/48_retention.sql")),
    ]
}

// Why: not `const` — with migrations present, `extension_migrations!()`
// expands to a `vec![…]` of embedded files, which cannot be built in const
// context (it could while the migrations directory was empty).
#[doc(hidden)]
pub fn migrations() -> Vec<Migration> {
    extension_migrations!()
}
